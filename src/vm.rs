use crate::attach_arguments::AttachArguments;
use crate::env::JniEnv;
use crate::error::JniError;
use crate::init_arguments::InitArguments;
use crate::token::NoException;
use cfg_if::cfg_if;
use jni_sys;
use std;
use std::os::raw::c_void;
use std::ptr;

/// A struct for interacting with the Java VM without owning it.
///
/// See more documentation in [`JavaVM`](struct.JavaVM.html).
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

/// A struct for interacting with the Java VM. This struct owns the VM and will destroy it upon being dropped.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
///
/// # Examples
/// ```
/// use rust_jni::*;
/// use std::ptr;
///
/// let options = InitArguments::get_default(JniVersion::V8).unwrap()
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Gc))
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Jni));
///
/// let vm = JavaVM::create(&options).unwrap();
/// assert_ne!(unsafe { vm.raw_jvm() }, ptr::null_mut());
///
/// let vms = JavaVM::list().unwrap();
/// unsafe {
///     assert_eq!(vms[0].raw_jvm(), vm.raw_jvm());
/// }
/// ```
/// [`JavaVM`](struct.JavaVM.html) is `Send + Sync`. It means it can be shared between threads.
/// ```
/// use rust_jni::*;
/// use std::ptr;
/// use std::sync::Arc;
///
/// let vm = Arc::new(JavaVM::create(&InitArguments::default()).unwrap());
/// {
///     let vm = vm.clone();
///     ::std::thread::spawn(move || {
///         assert_ne!(unsafe { vm.raw_jvm() }, ptr::null_mut());
///     });
/// }
/// assert_ne!(unsafe { vm.raw_jvm() }, ptr::null_mut());
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
    ///
    /// Currently this is the case even if the object is dropped.
    /// TODO(monnoroch): figure out why and document it.
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
    /// Returns a list of non-owning [`JavaVMRef`](struct.JavaVMRef.html)-s.
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
        closure: impl for<'token> FnOnce(
            &'token JniEnv<'token>,
            NoException<'token>,
        ) -> (T, NoException<'token>),
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
    /// To have compile-time guarantees use [`with_attached`](struct.JavaVM.html#method.with_attached) instead.
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
    /// To have compile-time guarantees use [`with_attached_daemon`](struct.JavaVM.html#method.with_attached_daemon) instead.
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
                    None => Ok(JniEnv::attached(&self.java_vm, jni_env)),
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
}

/// Implement [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
/// for [`JavaVM`](struct.JavaVM.html) to cast it to a reference to
/// [`JavaVMRef`](struct.JavaVMRef.html).
///
/// As [`JavaVM`](struct.JavaVM.html) will be destroyed when dropped, references to it's
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

#[cfg(test)]
pub(crate) fn test_vm(ptr: *mut jni_sys::JavaVM) -> JavaVMRef {
    JavaVMRef { java_vm: ptr }
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
