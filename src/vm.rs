use crate::attach_arguments::AttachArguments;
use crate::class::Class;
use crate::error::JniError;
use crate::init_arguments::InitArguments;
use crate::object::Object;
use crate::primitives::ToJniTuple;
use crate::token::*;
use crate::version::JniVersion;
use cfg_if::cfg_if;
use jni_sys;
use std;
use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr;

include!("call_jni_method.rs");
include!("generate_class.rs");

/// A struct for interacting with the Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct JavaVMRef {
    java_vm: *mut jni_sys::JavaVM,
}

/// Make [`JavaVMRef`](struct.JavaVMRef.html) sendable between threads.
/// Guaranteed to be safe by JNI.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
unsafe impl Send for JavaVMRef {}

/// Make [`JavaVMRef`](struct.JavaVMRef.html) shareable by multiple threads.
/// Guaranteed to be safe by JNI.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
unsafe impl Sync for JavaVMRef {}

impl JavaVMRef {
    /// Get the raw Java VM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    ///
    /// [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/invocation.html#invocation-api-functions).
    pub unsafe fn raw_jvm(&self) -> *mut jni_sys::JavaVM {
        self.java_vm
    }

    /// Unsafe because one can pass an invalid `java_vm` pointer.
    pub(crate) unsafe fn from_ptr(java_vm: *mut jni_sys::JavaVM) -> Self {
        Self { java_vm }
    }

    /// Unsafe because:
    /// 1. A user might pass an incorrect pointer.
    /// 2. The current thread might not be attached.
    pub(crate) unsafe fn detach(java_vm: *mut jni_sys::JavaVM) {
        let detach_fn = (**java_vm).DetachCurrentThread.unwrap();
        let error = JniError::from_raw(detach_fn(java_vm));
        // There is no way to recover from detach failure, except leak or fail.
        if error.is_some() {
            panic!(
                "Could not detach the current thread. Status: {:?}",
                error.unwrap()
            );
        }
    }
}

#[cfg(test)]
mod java_vm_ref_tests {
    use super::*;

    #[test]
    fn as_ref() {
        let vm = JavaVMRef {
            java_vm: 0x1234 as *mut jni_sys::JavaVM,
        };

        // Safe because we're not using the pointer.
        assert_eq!(unsafe { vm.raw_jvm() }, vm.java_vm);
    }
}

/// A struct for interacting with the Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
///
/// # Examples
/// ```
/// use rust_jni::{InitArguments, JavaVM, JniVersion, JvmOption, JvmVerboseOption};
/// use std::ptr;
///
/// let options = InitArguments::get_default(JniVersion::V8).unwrap()
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Gc))
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Jni));
///
/// let vm = JavaVM::create(&options).unwrap();
/// unsafe {
///     assert_ne!(vm.raw_jvm(), ptr::null_mut());
/// }
///
/// let vms = JavaVM::list().unwrap();
/// unsafe {
///     assert_eq!(vms[0].raw_jvm(), vm.raw_jvm());
/// }
/// ```
/// `JavaVM` is `Send + Sync`. It means it can be shared between threads.
/// ```
/// use rust_jni::{InitArguments, JavaVM, JniVersion};
/// use std::ptr;
/// use std::sync::Arc;
///
/// let vm =
///     Arc::new(JavaVM::create(&InitArguments::get_default(JniVersion::V8).unwrap()).unwrap());
/// {
///     let vm = vm.clone();
///     ::std::thread::spawn(move || {
///         unsafe {
///             assert_ne!(vm.raw_jvm(), ptr::null_mut());
///         }
///     });
/// }
/// unsafe {
///     assert_ne!(vm.raw_jvm(), ptr::null_mut());
/// }
/// ```
///
/// The main purpose of [`JavaVM`](struct.JavaVM.html) is to attach threads by provisioning
/// [`JniEnv`](struct.JniEnv.html)-s.
#[derive(Debug)]
pub struct JavaVM {
    java_vm: JavaVMRef,
}

impl JavaVM {
    /// Create a Java VM with the specified arguments.
    ///
    /// [Only one](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    /// Java VM per process is supported. When called for the second time will return an error.
    /// This is the case even if the object is dropped.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn create(arguments: &InitArguments) -> Result<Self, JniError> {
        let mut java_vm: *mut jni_sys::JavaVM = ptr::null_mut();
        let mut jni_env: *mut jni_sys::JNIEnv = ptr::null_mut();
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let mut raw_arguments = arguments.to_raw(&mut strings_buffer, &mut options_buffer);
        // Safe because we pass pointers to valid values which we just initialized.
        let error = JniError::from_raw(unsafe {
            JNI_CreateJavaVM(
                (&mut java_vm) as *mut *mut jni_sys::JavaVM,
                (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
                &mut raw_arguments.raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
            )
        });
        match error {
            None => {
                // We want to detach the current thread because we want to only allow attaching
                // a thread once and the `attach` method will panic if the thread is already
                // attached. Detaching here makes this logic easier to implement.
                // Safe because `JNI_CreateJavaVM` returned OK and hence `java_vm`
                // is a valid `jni_sys::JavaVM` pointer and because `JNI_CreateJavaVM` attaches
                // the current thread.
                // [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/invocation.html#detachcurrentthread)
                // says trying to detach a thread that is not attached is a no-op.
                unsafe { JavaVMRef::detach(java_vm) };

                Ok(Self {
                    java_vm: JavaVMRef { java_vm },
                })
            }
            Some(JniError::UnsupportedVersion) => panic!(
                "Got upsupported version error when creating a Java VM. \
                 Should not happen as `InitArguments` are supposed to check \
                 for version support."
            ),
            Some(JniError::ThreadDetached) => {
                panic!("Unexpected `EDETACHED` error when creating a Java VM.")
            }
            Some(error) => Err(error),
        }
    }

    /// Get a list of created Java VMs.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getcreatedjavavms)
    pub fn list() -> Result<Vec<JavaVMRef>, JniError> {
        let mut vms_created: jni_sys::jsize = 0;
        // Safe because arguments are correct.
        let error = JniError::from_raw(unsafe {
            JNI_GetCreatedJavaVMs(
                ::std::ptr::null_mut(),
                0,
                (&mut vms_created) as *mut jni_sys::jsize,
            )
        });
        match error {
            None => {
                let mut java_vms: Vec<*mut jni_sys::JavaVM> = vec![];
                java_vms.resize(vms_created as usize, ::std::ptr::null_mut());
                let mut tmp: jni_sys::jsize = 0;
                // Safe because arguments are valid.
                let error = JniError::from_raw(unsafe {
                    JNI_GetCreatedJavaVMs(
                        (java_vms.as_mut_ptr()) as *mut *mut jni_sys::JavaVM,
                        vms_created,
                        // Technically, a new VM could have been created since the previous call to
                        // `JNI_GetCreatedJavaVMs`. But then we also technically should not return
                        // any new ones, because they weren't there wneh this function was called.
                        (&mut tmp) as *mut jni_sys::jsize,
                    )
                });
                match error {
                    None => Ok(java_vms
                        .iter()
                        .cloned()
                        // Safe as the validity of the pointer is guaranteed by JNI.
                        .map(|java_vm| unsafe { JavaVMRef::from_ptr(java_vm) })
                        .collect()),
                    Some(error) => Err(error),
                }
            }
            Some(error) => Err(error),
        }
    }

    /// Get the raw Java VM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    ///
    /// [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/invocation.html#invocation-api-functions).
    pub unsafe fn raw_jvm(&self) -> *mut jni_sys::JavaVM {
        self.java_vm.raw_jvm()
    }

    /// Attach the current thread to the Java VM and execute code that calls JNI on it.
    ///
    /// Runs a closure passing it a newly attached [`JniEnv`](struct.JniEnv.html) and
    /// a [`NoException`](struct.NoException.html) token. The closure must return the
    /// [`NoException`](struct.NoException.html) token thus guaranteeing that there are no exceptions in flight after
    /// the closure is done executing.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn with_attached<'vm, T>(
        &'vm self,
        arguments: &AttachArguments,
        closure: impl for<'token> FnOnce(&JniEnv, NoException<'token>) -> (T, NoException<'token>),
    ) -> Result<T, JniError> {
        let env = self.attach(arguments)?;
        // Safe because we work with a freshly created JniEnv that doens't have an exception in flight.
        let token = unsafe { NoException::new(&env) };
        let (result, _token) = closure(&env, token);
        Ok(result)
    }

    /// Attach the current thread to the Java VM as a daemon and execute code that calls JNI on it.
    ///
    /// Runs a closure passing it a newly attached [`JniEnv`](struct.JniEnv.html) and
    /// a [`NoException`](struct.NoException.html) token. The closure must return the
    /// [`NoException`](struct.NoException.html) token thus guaranteeing that there are no exceptions in flight after
    /// the closure is done executing.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn with_attached_daemon<'vm, T>(
        &'vm self,
        arguments: &AttachArguments,
        closure: impl for<'token> FnOnce(&JniEnv, NoException<'token>) -> (T, NoException<'token>),
    ) -> Result<T, JniError> {
        let env = self.attach_daemon(arguments)?;
        // Safe because we work with a freshly created JniEnv that doens't have an exception in flight.
        let token = unsafe { NoException::new(&env) };
        let (result, _token) = closure(&env, token);
        Ok(result)
    }

    /// Attach the current thread to the Java VM with.
    /// Returns a [`JniEnv`](struct.JniEnv.html) instance for this thread.
    ///
    /// Exception-safety is based on the [`NoException`](struct.NoException.html) token and guaranteed in run time.
    /// To have compile-time guarantees use `with_attached` instead.
    ///
    /// Use this method only when ownership of the [`JniEnv`](struct.JniEnv.html) is required.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn attach<'vm: 'env, 'env>(
        &'vm self,
        arguments: &AttachArguments,
    ) -> Result<JniEnv<'env>, JniError> {
        // Safe because the argument is ensured to be the correct method.
        unsafe { self.attach_generic(arguments, (**self.raw_jvm()).AttachCurrentThread.unwrap()) }
    }

    /// Attach the current thread to the Java VM as a daemon.
    /// Returns a [`JniEnv`](struct.JniEnv.html) instance for this thread.
    ///
    /// Exception-safety is based on the [`NoException`](struct.NoException.html) token and guaranteed in run time.
    /// To have compile-time guarantees use `with_attached_daemon` instead.
    ///
    /// Use this method only when ownership of the [`JniEnv`](struct.JniEnv.html) is required.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthreadasdaemon)
    pub fn attach_daemon<'vm: 'env, 'env>(
        &'vm self,
        arguments: &AttachArguments,
    ) -> Result<JniEnv<'env>, JniError> {
        // Safe because the argument is ensured to be the correct method.
        unsafe {
            self.attach_generic(
                arguments,
                (**self.raw_jvm()).AttachCurrentThreadAsDaemon.unwrap(),
            )
        }
    }

    /// Unsafe because:
    /// 1. One can pass an invalid `attach_fn`.
    /// 2. The current thread might already be attached.
    unsafe fn attach_generic(
        &self,
        arguments: &AttachArguments,
        attach_fn: unsafe extern "system" fn(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint,
    ) -> Result<JniEnv, JniError> {
        let mut buffer: Vec<u8> = vec![];
        let mut raw_arguments = arguments.to_raw(&mut buffer);
        let mut jni_env: *mut jni_sys::JNIEnv = ::std::ptr::null_mut();
        let get_env_fn = (**self.raw_jvm()).GetEnv.unwrap();
        // Safe, because the arguments are correct.
        let error = JniError::from_raw(get_env_fn(
            self.raw_jvm(),
            (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
            arguments.version().to_raw(),
        ));
        match error {
            Some(JniError::ThreadDetached) => {
                let error = JniError::from_raw(attach_fn(
                    self.raw_jvm(),
                    (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
                    (&mut raw_arguments.raw_arguments) as *mut jni_sys::JavaVMAttachArgs
                        as *mut c_void,
                ));
                match error {
                    None => {
                        let mut env = JniEnv {
                            vm: &self.java_vm,
                            jni_env,
                            has_token: RefCell::new(true),
                            // We don't want to drop `JniEnv` with a pending exception.
                            native_method_call: true,
                        };
                        if env.has_exception() {
                            panic!("Newly attached thread has a pending exception.");
                        }
                        env.native_method_call = false;
                        Ok(env)
                    }
                    Some(JniError::UnsupportedVersion) => panic!(
                        "Got upsupported version error when creating a Java VM. \
                         Should not happen as `InitArguments` are supposed to check \
                         for version support."
                    ),
                    Some(JniError::ThreadDetached) => {
                        panic!("Got `EDETACHED` when trying to attach a thread.")
                    }
                    // TODO: panic on more impossible errors.
                    Some(error) => Err(error),
                }
            }
            None => panic!(
                "This thread is already attached to the JVM. \
                 Attaching a thread twice is not allowed."
            ),
            // According to the
            // [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#getenv),
            // can only returd `OK`, `EDETACHED` and `EVERSION`.
            // Will not return `EVERSION` here, because the version was already checked when
            // creating the Java VM.
            Some(error) => panic!(
                "GetEnv JNI method returned an unexpected error code {:?}",
                error
            ),
        }
    }

    /// Unsafe because:
    /// 1. A user might pass an incorrect pointer.
    /// 2. The current thread might not be attached.
    unsafe fn detach(java_vm: *mut jni_sys::JavaVM) {
        let detach_fn = (**java_vm).DetachCurrentThread.unwrap();
        let error = JniError::from_raw(detach_fn(java_vm));
        // There is no way to recover from detach failure, except leak or fail.
        if error.is_some() {
            panic!(
                "Could not detach the current thread. Status: {:?}",
                error.unwrap()
            )
        }
    }
}

/// Implement [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
/// for [`JavaVM`](struct.JavaVm.html) to cast it to a reference to
/// [`JavaVMRef`](struct.JavaVMRef.html).
///
/// As [`JavaVM`](struct.JavaVm.html) will be destroyed when dropped, references to it's
/// [`JavaVMRef`-s](struct.JavaVMRef.html) should not outlive it.
/// This impl is not very useful and mostly serves as the documentation of this fact.
impl AsRef<JavaVMRef> for JavaVM {
    fn as_ref<'vm>(&'vm self) -> &'vm JavaVMRef {
        &self.java_vm
    }
}

/// Destroy [`JavaVM`](struct.JavaVM.html) when the value is dropped.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#destroyjavavm)
impl Drop for JavaVM {
    fn drop(&mut self) {
        // Safe because JavaVM can't be created from an invalid or non-owned Java VM pointer.
        let error = JniError::from_raw(unsafe {
            let destroy_fn = (**self.raw_jvm()).DestroyJavaVM.unwrap();
            destroy_fn(self.raw_jvm())
        });
        if error.is_some() {
            // Drop is supposed to always succeed. We can't do anything besides panicing in case of failure.
            panic!("Failed destroying the JavaVm. Status: {:?}", error.unwrap());
        }
    }
}

#[cfg(test)]
mod java_vm_tests {
    use super::*;
    use std::mem;

    #[test]
    fn as_ref() {
        let vm_ref = JavaVMRef {
            java_vm: 0x1234 as *mut jni_sys::JavaVM,
        };
        let vm = JavaVM { java_vm: vm_ref };

        assert_eq!(vm.as_ref(), &vm_ref);
        // Do not drop: we didn't mock the destructor.
        mem::forget(vm);
    }
}

#[cfg(test)]
mod java_vm_drop_tests {
    use super::*;
    use crate::testing::empty_raw_java_vm;
    use mockall::*;
    use serial_test_derive::serial;

    #[automock(mod mock;)]
    // We're not using the non-test function.
    #[allow(dead_code)]
    extern "Rust" {
        pub fn destroy_vm(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint;
    }

    unsafe extern "system" fn destroy_vm_impl(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint {
        mock::destroy_vm(java_vm)
    }

    fn raw_java_vm() -> jni_sys::JNIInvokeInterface_ {
        jni_sys::JNIInvokeInterface_ {
            DestroyJavaVM: Some(destroy_vm_impl),
            ..empty_raw_java_vm()
        }
    }

    #[test]
    #[serial]
    fn drop() {
        let raw_java_vm = raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        // Need to pass a number to the closure below as pointers are not Send.
        let raw_java_vm_ptr_usize = raw_java_vm_ptr as usize;
        let destroy_vm_mock = mock::destroy_vm_context();
        {
            let _vm = JavaVM {
                java_vm: JavaVMRef {
                    java_vm: raw_java_vm_ptr,
                },
            };
            // Nothing has happened yet.
            destroy_vm_mock.checkpoint();
            destroy_vm_mock
                .expect()
                .times(1)
                .withf(move |x| *x as usize == raw_java_vm_ptr_usize)
                .return_const(jni_sys::JNI_OK);
        }
        // Expectations are checked after the scope has ended.
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Failed destroying the JavaVm. Status: Unknown(-1)")]
    fn drop_panics() {
        let raw_java_vm = raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let destroy_vm_mock = mock::destroy_vm_context();
        destroy_vm_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR);
        {
            let _vm = JavaVM {
                java_vm: JavaVMRef {
                    java_vm: raw_java_vm_ptr,
                },
            };
        }
        // Nothing has happened.
        destroy_vm_mock.checkpoint();
    }
}

#[cfg(test)]
mod java_vm_create_tests {
    use super::*;
    use crate::testing::empty_raw_java_vm;
    use mockall::*;
    use serial_test_derive::serial;
    use std::mem;

    #[automock(mod mock;)]
    // We're not using the non-test function.
    #[allow(dead_code)]
    extern "Rust" {
        pub fn detach_thread(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint;
    }

    unsafe extern "system" fn detach_thread_impl(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint {
        mock::detach_thread(java_vm)
    }

    fn raw_java_vm() -> jni_sys::JNIInvokeInterface_ {
        jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach_thread_impl),
            ..empty_raw_java_vm()
        }
    }

    #[test]
    #[serial]
    fn create() {
        let raw_java_vm = raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        // Need to pass a number to the closure below as pointers are not Send.
        let raw_java_vm_ptr_usize = raw_java_vm_ptr as usize;
        let mut sequence = Sequence::new();
        let create_vm_mock = ffi::mock::JNI_CreateJavaVM_context();
        create_vm_mock
            .expect()
            .times(1)
            .withf(move |java_vm, _jni_env, arguments| {
                let arguments = *arguments as *mut jni_sys::JavaVMInitArgs;
                // We know that this pointer points to a valid value.
                match unsafe { arguments.as_ref() } {
                    None => false,
                    Some(arguments) => {
                        // We know raw arguments value is valid.
                        let arguments = unsafe { InitArguments::from_raw(arguments) };
                        if arguments != InitArguments::default() {
                            false
                        } else {
                            // Safe because we allocated a valid value on the stack in JavaVM::create().
                            unsafe {
                                **java_vm = raw_java_vm_ptr_usize as *mut jni_sys::JavaVM;
                            }
                            true
                        }
                    }
                }
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock
            .expect()
            .times(1)
            .withf(move |java_vm| *java_vm as usize == raw_java_vm_ptr_usize)
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let vm = JavaVM::create(&InitArguments::default()).unwrap();
        unsafe {
            assert_eq!(vm.raw_jvm(), raw_java_vm_ptr);
        }
        // Do not drop: we didn't mock the destructor.
        mem::forget(vm);
    }

    #[test]
    #[serial]
    fn create_error() {
        let create_vm_mock = ffi::mock::JNI_CreateJavaVM_context();
        create_vm_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR);
        assert_eq!(
            JavaVM::create(&InitArguments::default()).err().unwrap(),
            JniError::Unknown(jni_sys::JNI_ERR)
        );
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    // Result unused because the funtion will panic.
    #[allow(unused_must_use)]
    #[should_panic(expected = "upsupported version")]
    fn create_error_version() {
        let create_vm_mock = ffi::mock::JNI_CreateJavaVM_context();
        create_vm_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EVERSION);
        JavaVM::create(&InitArguments::default());
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    // Result unused because the funtion will panic.
    #[allow(unused_must_use)]
    #[should_panic(expected = "Unexpected `EDETACHED`")]
    fn create_error_detached() {
        let create_vm_mock = ffi::mock::JNI_CreateJavaVM_context();
        create_vm_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED);
        JavaVM::create(&InitArguments::default());
    }
}

#[cfg(test)]
mod java_vm_list_tests {
    use super::*;
    use crate::testing::empty_raw_java_vm;
    use mockall::*;
    use serial_test_derive::serial;

    #[test]
    #[serial]
    fn list() {
        let raw_java_vm_1 = empty_raw_java_vm();
        let raw_java_vm_ptr_1 = &mut (&raw_java_vm_1 as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let raw_java_vm_2 = empty_raw_java_vm();
        let raw_java_vm_ptr_2 = &mut (&raw_java_vm_2 as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        assert_ne!(raw_java_vm_ptr_1, raw_java_vm_ptr_2);

        // Need to pass a number to the closure below as pointers are not Send.
        let raw_java_vm_ptr_1_usize = raw_java_vm_ptr_1 as usize;
        let raw_java_vm_ptr_2_usize = raw_java_vm_ptr_2 as usize;

        let mut sequence = Sequence::new();
        let list_vms_mock = ffi::mock::JNI_GetCreatedJavaVMs_context();
        list_vms_mock
            .expect()
            .times(1)
            .withf(move |java_vms, buffer_size, vms_count| {
                if *java_vms != ptr::null_mut() || *buffer_size != 0 {
                    false
                } else {
                    // Safe because the data is allocated on the stack in `list()`.
                    unsafe {
                        **vms_count = 2 as jni_sys::jint;
                    }
                    true
                }
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        list_vms_mock
            .expect()
            .times(1)
            .withf(move |java_vms, buffer_size, vms_count| {
                if *buffer_size != 2 {
                    false
                } else {
                    unsafe {
                        **java_vms = raw_java_vm_ptr_1_usize as *mut jni_sys::JavaVM;
                        *((*java_vms).offset(1)) = raw_java_vm_ptr_2_usize as *mut jni_sys::JavaVM;
                        **vms_count = 2 as jni_sys::jint;
                    }
                    true
                }
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let vms = JavaVM::list().unwrap();
        unsafe {
            assert_eq!(vms[0].raw_jvm(), raw_java_vm_ptr_1);
            assert_eq!(vms[1].raw_jvm(), raw_java_vm_ptr_2);
        }
    }

    #[test]
    #[serial]
    fn list_error_first_call() {
        let list_vms_mock = ffi::mock::JNI_GetCreatedJavaVMs_context();
        list_vms_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR);
        assert_eq!(JavaVM::list(), Err(JniError::Unknown(jni_sys::JNI_ERR)));
    }

    #[test]
    #[serial]
    fn list_error_second_call() {
        let mut sequence = Sequence::new();
        let list_vms_mock = ffi::mock::JNI_GetCreatedJavaVMs_context();
        list_vms_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        list_vms_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR)
            .in_sequence(&mut sequence);
        assert_eq!(JavaVM::list(), Err(JniError::Unknown(jni_sys::JNI_ERR)));
    }
}

#[cfg(test)]
mod java_vm_tests_legacy {
    use super::*;
    use crate::init_arguments;

    fn default_args() -> InitArguments {
        init_arguments::init_arguments_manipulation_tests::default_args()
    }

    // #[test]
    // fn attach() {
    //     let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
    //         result: jni_sys::JNI_FALSE,
    //     })]);
    //     static mut GET_ENV_CALLS: i32 = 0;
    //     static mut GET_ENV_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
    //     static mut GET_ENV_VERSION_ARGUMENT: jni_sys::jint = 0;
    //     unsafe extern "system" fn get_env(
    //         java_vm: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         version: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         GET_ENV_CALLS += 1;
    //         GET_ENV_VM_ARGUMENT = java_vm;
    //         GET_ENV_VERSION_ARGUMENT = version;
    //         jni_sys::JNI_EDETACHED
    //     }
    //     static mut ATTACH_CALLS: i32 = 0;
    //     static mut ATTACH_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
    //     static mut ATTACH_ENV_ARGUMENT: *mut c_void = ptr::null_mut();
    //     static mut ATTACH_ARGUMENT: *mut c_void = ptr::null_mut();
    //     unsafe extern "system" fn attach(
    //         java_vm: *mut jni_sys::JavaVM,
    //         jni_env: *mut *mut c_void,
    //         argument: *mut c_void,
    //     ) -> jni_sys::jint {
    //         *jni_env = ATTACH_ENV_ARGUMENT;
    //         ATTACH_CALLS += 1;
    //         ATTACH_VM_ARGUMENT = java_vm;
    //         ATTACH_ARGUMENT = argument;
    //         jni_sys::JNI_OK
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     let init_arguments = InitArguments::default().with_version(JniVersion::V8);
    //     unsafe {
    //         ATTACH_ENV_ARGUMENT = calls.env as *mut c_void;
    //     }
    //     let env = vm
    //         .attach(&AttachArguments::named(&init_arguments, "test-name"))
    //         .unwrap();
    //     unsafe {
    //         assert_eq!(GET_ENV_CALLS, 1);
    //         assert_eq!(GET_ENV_VM_ARGUMENT, raw_java_vm_ptr);
    //         assert_eq!(GET_ENV_VERSION_ARGUMENT, JniVersion::V8.to_raw());
    //         assert_eq!(ATTACH_CALLS, 1);
    //         assert_eq!(ATTACH_VM_ARGUMENT, raw_java_vm_ptr);
    //         assert_eq!(
    //             from_java_string(
    //                 CStr::from_ptr((*(ATTACH_ARGUMENT as *mut jni_sys::JavaVMAttachArgs)).name)
    //                     .to_bytes_with_nul()
    //             )
    //             .unwrap(),
    //             "test-name"
    //         );
    //         assert_eq!(env.raw_jvm(), raw_java_vm_ptr);
    //         assert_eq!(env.raw_env(), calls.env);
    //     }
    //     assert_eq!(env.has_token, RefCell::new(true));
    //     assert_eq!(env.native_method_call, false);
    //     // Don't want to drop a manually created `JniEnv`.
    //     mem::forget(env);
    // }

    // #[test]
    // #[should_panic(expected = "already attached")]
    // fn attach_already_attached() {
    //     unsafe extern "system" fn get_env(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_OK
    //     }
    //     unsafe extern "system" fn attach(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: *mut c_void,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_OK
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    // }

    // #[test]
    // #[should_panic(expected = "GetEnv JNI method returned an unexpected error code Unknown(-1)")]
    // fn attach_get_env_error() {
    //     unsafe extern "system" fn get_env(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_ERR
    //     }
    //     unsafe extern "system" fn attach(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: *mut c_void,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_OK
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    // }

    // #[test]
    // #[should_panic(expected = "Got `EDETACHED` when trying to attach a thread")]
    // fn attach_cant_attach() {
    //     unsafe extern "system" fn get_env(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_EDETACHED
    //     }
    //     unsafe extern "system" fn attach(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: *mut c_void,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_EDETACHED
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    // }

    // #[test]
    // #[should_panic(expected = "upsupported version")]
    // fn attach_unsupported_version() {
    //     unsafe extern "system" fn get_env(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_EDETACHED
    //     }
    //     unsafe extern "system" fn attach(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: *mut c_void,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_EVERSION
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    // }

    // #[test]
    // fn attach_attach_error() {
    //     unsafe extern "system" fn get_env(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_EDETACHED
    //     }
    //     unsafe extern "system" fn attach(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: *mut c_void,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_ERR
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     assert_eq!(
    //         vm.attach(&AttachArguments::new(JniVersion::V8))
    //             .unwrap_err(),
    //         JniError::Unknown(jni_sys::JNI_ERR as i32)
    //     );
    // }

    // #[test]
    // #[should_panic(expected = "Newly attached thread has a pending exception")]
    // fn attach_pending_exception() {
    //     let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
    //         result: jni_sys::JNI_TRUE,
    //     })]);
    //     unsafe extern "system" fn get_env(
    //         _: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         _: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         jni_sys::JNI_EDETACHED
    //     }
    //     static mut ATTACH_ENV_ARGUMENT: *mut c_void = ptr::null_mut();
    //     unsafe extern "system" fn attach(
    //         _: *mut jni_sys::JavaVM,
    //         jni_env: *mut *mut c_void,
    //         _: *mut c_void,
    //     ) -> jni_sys::jint {
    //         *jni_env = ATTACH_ENV_ARGUMENT;
    //         jni_sys::JNI_OK
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThread: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     unsafe {
    //         ATTACH_ENV_ARGUMENT = calls.env as *mut c_void;
    //     }
    //     vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    // }

    // #[test]
    // fn attach_daemon() {
    //     let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
    //         result: jni_sys::JNI_FALSE,
    //     })]);
    //     static mut GET_ENV_CALLS: i32 = 0;
    //     static mut GET_ENV_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
    //     static mut GET_ENV_VERSION_ARGUMENT: jni_sys::jint = 0;
    //     unsafe extern "system" fn get_env(
    //         java_vm: *mut jni_sys::JavaVM,
    //         _: *mut *mut c_void,
    //         version: jni_sys::jint,
    //     ) -> jni_sys::jint {
    //         GET_ENV_CALLS += 1;
    //         GET_ENV_VM_ARGUMENT = java_vm;
    //         GET_ENV_VERSION_ARGUMENT = version;
    //         jni_sys::JNI_EDETACHED
    //     }
    //     static mut ATTACH_CALLS: i32 = 0;
    //     static mut ATTACH_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
    //     static mut ATTACH_ENV_ARGUMENT: *mut c_void = ptr::null_mut();
    //     static mut ATTACH_ARGUMENT: *mut c_void = ptr::null_mut();
    //     unsafe extern "system" fn attach(
    //         java_vm: *mut jni_sys::JavaVM,
    //         jni_env: *mut *mut c_void,
    //         argument: *mut c_void,
    //     ) -> jni_sys::jint {
    //         *jni_env = ATTACH_ENV_ARGUMENT;
    //         ATTACH_CALLS += 1;
    //         ATTACH_VM_ARGUMENT = java_vm;
    //         ATTACH_ARGUMENT = argument;
    //         jni_sys::JNI_OK
    //     }
    //     let raw_java_vm = jni_sys::JNIInvokeInterface_ {
    //         GetEnv: Some(get_env),
    //         AttachCurrentThreadAsDaemon: Some(attach),
    //         ..empty_raw_java_vm()
    //     };
    //     let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
    //     let vm = test_vm(raw_java_vm_ptr);
    //     let init_arguments = InitArguments::default().with_version(JniVersion::V8);
    //     unsafe {
    //         ATTACH_ENV_ARGUMENT = calls.env as *mut c_void;
    //     }
    //     let env = vm
    //         .attach_daemon(&AttachArguments::named(&init_arguments, "test-name"))
    //         .unwrap();
    //     unsafe {
    //         assert_eq!(GET_ENV_CALLS, 1);
    //         assert_eq!(GET_ENV_VM_ARGUMENT, raw_java_vm_ptr);
    //         assert_eq!(GET_ENV_VERSION_ARGUMENT, JniVersion::V8.to_raw());
    //         assert_eq!(ATTACH_CALLS, 1);
    //         assert_eq!(ATTACH_VM_ARGUMENT, raw_java_vm_ptr);
    //         assert_eq!(
    //             from_java_string(
    //                 CStr::from_ptr((*(ATTACH_ARGUMENT as *mut jni_sys::JavaVMAttachArgs)).name)
    //                     .to_bytes_with_nul()
    //             )
    //             .unwrap(),
    //             "test-name"
    //         );
    //         assert_eq!(env.raw_jvm(), raw_java_vm_ptr);
    //         assert_eq!(env.raw_env(), calls.env);
    //     }
    //     assert_eq!(env.has_token, RefCell::new(true));
    //     assert_eq!(env.native_method_call, false);
    //     // Don't want to drop a manually created `JniEnv`.
    //     mem::forget(env);
    // }
}

/// The interface for interacting with Java.
/// All calls to Java are performed through this interface.
/// JNI methods can only be called from threads, explicitly attached to the Java VM.
/// [`JniEnv`](struct.JniEnv.html) represents such a thread.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#interface-function-table)
///
/// # Examples
/// ```
/// use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// use std::ptr;
///
/// let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
/// unsafe {
///     assert_ne!(env.raw_env(), ptr::null_mut());
/// }
/// ```
/// [`JniEnv`](struct.JniEnv.html) is
/// [`!Send`](https://doc.rust-lang.org/std/marker/trait.Send.html). It means it can't be passed
/// between threads:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
/// {
///     ::std::thread::spawn(move || {
///         unsafe { env.raw_env() }; // doesn't compile!
///     });
/// }
/// ```
/// Instead, you need to attach each new thread to the VM:
/// ```
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// # use std::ptr;
/// use std::sync::Arc;
///
/// let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// let vm = Arc::new(JavaVM::create(&init_arguments).unwrap());
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
/// {
///     let vm = vm.clone();
///     ::std::thread::spawn(move || {
///         let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
///         unsafe {
///             assert_ne!(env.raw_env(), ptr::null_mut());
///         }
///     });
/// }
/// unsafe {
///     assert_ne!(env.raw_env(), ptr::null_mut());
/// }
/// ```
/// The thread is automatically detached once the [`JniEnv`](struct.JniEnv.html) is dropped.
///
/// [`JniEnv`](struct.JniEnv.html) can't outlive the parent [`JavaVM`](struct.JavaVM.html).
/// This code is not allowed:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// let env = {
///     let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
///     let vm = JavaVM::create(&init_arguments).unwrap();
///     vm.attach(&AttachArguments::new(init_arguments.version())).unwrap() // doesn't compile!
/// };
/// ```
/// [`JniEnv`](struct.JniEnv.html) represents a thread, attached to the Java VM. Thus there
/// can't be two [`JniEnv`](struct.JniEnv.html)-s per thread.
/// [`JavaVM::attach`](struct.JavaVM.html#methods.attach) will panic if you attempt to do so:
/// ```should_panic
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap(); // panics!
/// ```
// TODO: docs about panicing on detach when there's a pending exception.
#[derive(Debug)]
pub struct JniEnv<'this> {
    vm: &'this JavaVMRef,
    jni_env: *mut jni_sys::JNIEnv,
    has_token: RefCell<bool>,
    native_method_call: bool,
}

// [`JniEnv`](struct.JniEnv.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'vm> !Send for JniEnv<'vm> {}
// impl<'vm> !Sync for JniEnv<'vm> {}

impl<'this> JniEnv<'this> {
    /// Get the raw Java VM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    pub unsafe fn raw_jvm(&self) -> *mut jni_sys::JavaVM {
        self.vm.raw_jvm()
    }

    /// Get the raw JNI environment pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    pub unsafe fn raw_env(&self) -> *mut jni_sys::JNIEnv {
        self.jni_env
    }

    /// Get a [`NoException`](struct.NoException.html) token indicating that there is no pending
    /// exception in this thread.
    ///
    /// Read more about tokens in [`NoException`](struct.NoException.html) documentation.
    // TODO(#22): Return a token with the env if possible:
    // https://stackoverflow.com/questions/50891977/can-i-return-a-value-and-a-reference-to-it-from-a-function.
    pub fn token(&self) -> NoException {
        if !*self.has_token.borrow() {
            panic!("Trying to obtain a second `NoException` token from the `JniEnv` value.");
        } else if self.has_exception() {
            panic!("Trying to obtain a `NoException` token when there is a pending exception.");
        } else {
            *self.has_token.borrow_mut() = false;
            // Safe because there's no exception.
            unsafe { NoException::new(self) }
        }
    }

    /// Get JNI versoin.
    ///
    /// [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/functions.html#getversion)
    pub fn version(&self) -> JniVersion {
        JniVersion::from_raw(unsafe { call_jni_method!(self, GetVersion) })
    }

    pub(crate) fn native<'vm: 'env, 'env>(
        vm: &'vm JavaVMRef,
        jni_env: *mut jni_sys::JNIEnv,
    ) -> JniEnv<'env> {
        JniEnv {
            vm,
            jni_env,
            has_token: RefCell::new(true),
            native_method_call: true,
        }
    }

    pub(crate) fn has_exception(&self) -> bool {
        // Safe because the argument is ensured to be the correct by construction.
        let value = unsafe { call_jni_method!(self, ExceptionCheck) };
        // Safe because `bool` conversion is safe internally.
        unsafe { bool::__from_jni(self, value) }
    }
}

/// `Drop` detaches the current thread from the Java VM.
/// [It's not safe](https://developer.android.com/training/articles/perf-jni#exceptions)
/// to do so with an exception pending, so it panics if this happens.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#detachcurrentthread)
impl<'vm> Drop for JniEnv<'vm> {
    fn drop(&mut self) {
        // In native calls `JniEnv` is build from a raw pointer, without attaching the current
        // thread, and thus the thread doesn't need to be detached. A native method can return
        // with a pending exception to propagate it to Java code, so no need to panic on pending
        // exceptions either.
        if self.native_method_call {
            return;
        }

        if self.has_exception() {
            // Safe because the argument is ensured to be the correct by construction.
            unsafe { call_jni_method!(self, ExceptionDescribe) };
            panic!(
                "Dropping `JniEnv` with a pending exception is not allowed. Please clear the \
                 exception by unwrapping the exception token before dropping it."
            );
        }
        // Safe because the current thread is guaranteed to be attached and the argument is correct.
        unsafe { JavaVM::detach(self.raw_jvm()) };
    }
}

#[cfg(test)]
pub(crate) fn test_vm(ptr: *mut jni_sys::JavaVM) -> JavaVMRef {
    JavaVMRef { java_vm: ptr }
}

#[cfg(test)]
pub(crate) fn test_env<'vm>(vm: &'vm JavaVMRef, ptr: *mut jni_sys::JNIEnv) -> JniEnv<'vm> {
    JniEnv {
        vm: &vm,
        jni_env: ptr,
        has_token: RefCell::new(true),
        native_method_call: true,
    }
}

#[cfg(test)]
mod jni_env_tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn raw_jvm() {
        let vm = test_vm(0x1234 as *mut jni_sys::JavaVM);
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(env.raw_jvm(), vm.raw_jvm());
        }
    }

    #[test]
    fn raw_env() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        unsafe {
            assert_eq!(env.raw_env(), jni_env);
        }
    }

    #[test]
    fn version() {
        let calls = test_raw_jni_env!(vec![JniCall::GetVersion(GetVersion {
            result: jni_sys::JNI_VERSION_1_4,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        assert_eq!(env.version(), JniVersion::V4);
    }

    #[test]
    fn drop() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
            result: jni_sys::JNI_FALSE,
        })]);
        static mut DETACH_CALLS: i32 = 0;
        static mut DETACH_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        unsafe extern "system" fn detach(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint {
            DETACH_CALLS += 1;
            DETACH_ARGUMENT = java_vm;
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        {
            let _env = JniEnv {
                vm: &vm,
                jni_env: calls.env,
                has_token: RefCell::new(true),
                native_method_call: false,
            };
            unsafe {
                assert_eq!(DETACH_CALLS, 0);
            }
        }
        unsafe {
            assert_eq!(DETACH_CALLS, 1);
            assert_eq!(DETACH_ARGUMENT, vm.java_vm);
        }
    }

    #[test]
    fn drop_native_method() {
        let vm = test_vm(ptr::null_mut());
        test_env(&vm, ptr::null_mut());
        // This test would fail if any JNI methods were called by the `JniEnv::drop` method.
    }

    #[test]
    #[should_panic(expected = "Dropping `JniEnv` with a pending exception is not allowed")]
    fn drop_exception_pending() {
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::ExceptionDescribe(ExceptionDescribe {}),
        ]);
        unsafe extern "system" fn destroy_vm(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DestroyJavaVM: Some(destroy_vm),
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        JniEnv {
            vm: &vm,
            jni_env: calls.env,
            has_token: RefCell::new(true),
            native_method_call: false,
        };
    }

    #[test]
    #[should_panic(expected = "Could not detach the current thread. Status: Unknown(-1)")]
    fn drop_detach_error() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
            result: jni_sys::JNI_FALSE,
        })]);
        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        JniEnv {
            vm: &vm,
            jni_env: calls.env,
            has_token: RefCell::new(true),
            native_method_call: false,
        };
    }

    #[test]
    fn token() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
            result: jni_sys::JNI_FALSE,
        })]);
        let raw_java_vm_ptr = 0x1234 as *mut jni_sys::JavaVM;
        let vm = test_vm(raw_java_vm_ptr);
        let env = test_env(&vm, calls.env);
        env.token();
        assert_eq!(env.has_token, RefCell::new(false));
    }

    #[test]
    #[should_panic(expected = "Trying to obtain a second `NoException` token from the `JniEnv`")]
    fn token_twice() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
            result: jni_sys::JNI_FALSE,
        })]);
        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        let env = JniEnv {
            vm: &vm,
            jni_env: calls.env,
            has_token: RefCell::new(false),
            native_method_call: true,
        };
        env.token();
    }

    #[test]
    #[should_panic(
        expected = "Trying to obtain a `NoException` token when there is a pending exception"
    )]
    fn token_pending_exception() {
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
        ]);
        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        let env = test_env(&vm, calls.env);
        env.token();
    }
}

/// A trait that represents a JNI type. It's implemented for all JNI primitive types
/// and [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
/// Implements Java method calls and provides the default value for this JNI type.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented for classes by generated code.
#[doc(hidden)]
pub trait JniType {
    fn default() -> Self;

    unsafe fn call_method<In: ToJniTuple>(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: In,
    ) -> Self;

    unsafe fn call_static_method<In: ToJniTuple>(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: In,
    ) -> Self;
}

/// A trait that represents JNI types that can be passed as arguments to JNI functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
#[doc(hidden)]
pub trait JniArgumentType: JniType {}

/// A trait that represents Rust types that are mappable to JNI types.
/// This trait has to be implemented for all types that need to be passed as arguments
/// to or returned from Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
pub trait JavaType {
    /// The corresponding JNI type.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    type __JniType: JniType;

    /// Compute the signature for this Java type.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    fn __signature() -> &'static str;
}

/// A trait for mapping types to their JNI types.
/// This trait has to be implemented for all types that need to be passed as arguments
/// to Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
#[doc(hidden)]
pub trait ToJni: JavaType {
    /// Map the value to a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    unsafe fn __to_jni(&self) -> Self::__JniType;
}

/// A trait for constructing types from their JNI types and [`JniEnv`](struct.JniEnv.html)
/// references. This trait has to be implemented for all types that the user wants to pass
/// return from Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
#[doc(hidden)]
pub trait FromJni<'env>: JavaType {
    /// Construct a value from a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    unsafe fn __from_jni(env: &'env JniEnv<'env>, value: Self::__JniType) -> Self;
}

/// Make references mappable to JNI types of their referenced types.
impl<'a, T> JavaType for &'a T
where
    T: JavaType + ?Sized,
{
    #[doc(hidden)]
    type __JniType = T::__JniType;

    #[doc(hidden)]
    fn __signature() -> &'static str {
        T::__signature()
    }
}

/// Make references mappable from JNI types of their referenced types.
impl<'a, T> ToJni for &'a T
where
    T: ToJni,
{
    unsafe fn __to_jni(&self) -> Self::__JniType {
        T::__to_jni(self)
    }
}

/// A trait that represents Rust function types that are mappable to Java function types.
/// This trait is separate from `JavaType` because this one doesn't need to be exposed
/// in the public crate API.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
// TODO: reimplement it in a way that it returns `&'static str`.
// `concat!` doesn't acceps arbitrary expressions of type `&'static str`, so it can't be
// implemented that way today.
#[doc(hidden)]
pub trait JavaMethodSignature<In: ?Sized, Out: ?Sized> {
    /// Get the method's JNI signature.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    fn __signature() -> std::string::String;
}

/// A trait for casting Java object types to their superclasses.
pub trait Cast<'env, As: Cast<'env, Object<'env>>>:
    JavaType<__JniType = jni_sys::jobject> + ToJni + FromJni<'env>
{
    /// Cast the object to itself or one of it's superclasses.
    ///
    /// Doesn't actually convert anything, the result is just the same object
    /// interpreted as one of it's superclasses.
    fn cast<'a>(&'a self) -> &'a As;
}

#[cfg(test)]
// JNI API.
#[allow(non_snake_case)]
mod ffi {
    use mockall::*;

    #[automock(mod create_vm_mock;)]
    // We're not using the non-test function.
    #[allow(dead_code)]
    extern "C" {
        pub fn JNI_CreateJavaVM(
            java_vm: *mut *mut jni_sys::JavaVM,
            jni_env: *mut *mut ::std::os::raw::c_void,
            arguments: *mut ::std::os::raw::c_void,
        ) -> jni_sys::jint;
    }

    #[automock(mod get_created_vms_mock;)]
    // We're not using the non-test function.
    #[allow(dead_code)]
    extern "C" {
        pub fn JNI_GetCreatedJavaVMs(
            java_vms: *mut *mut jni_sys::JavaVM,
            buffer_size: jni_sys::jsize,
            vms_count: *mut jni_sys::jsize,
        ) -> jni_sys::jint;
    }

    pub mod mock {
        pub use super::create_vm_mock::*;
        pub use super::get_created_vms_mock::*;
    }
}

cfg_if! {
    if #[cfg(test)] {
        use self::ffi::mock::JNI_CreateJavaVM;
    } else if #[cfg(all(not(test), feature = "libjvm"))] {
        use jni_sys::JNI_CreateJavaVM;
    } else if #[cfg(all(not(test), not(feature = "libjvm")))] {
        /// This is a stub for when we can't link to libjvm.
        // JNI API.
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn JNI_CreateJavaVM(
            _java_vm: *mut *mut jni_sys::JavaVM,
            _jni_env: *mut *mut c_void,
            _arguments: *mut c_void,
        ) -> jni_sys::jint {
            unimplemented!()
        }
    }
}

cfg_if! {
    if #[cfg(test)] {
        use self::ffi::mock::JNI_GetCreatedJavaVMs;
    } else if #[cfg(all(not(test), feature = "libjvm"))] {
        use jni_sys::JNI_GetCreatedJavaVMs;
    } else if #[cfg(all(not(test), not(feature = "libjvm")))] {
        /// This is a stub for when we can't link to libjvm.
        // JNI API.
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn JNI_GetCreatedJavaVMs(
            _java_vms: *mut *mut jni_sys::JavaVM,
            _buffer_size: jni_sys::jsize,
            _vms_count: *mut jni_sys::jsize,
        ) -> jni_sys::jint {
            unimplemented!()
        }
    }
}
