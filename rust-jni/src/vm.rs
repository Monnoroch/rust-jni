use crate::attach_arguments::AttachArguments;
use crate::env::JniEnv;
use crate::error::JniError;
use crate::init_arguments::InitArguments;
use crate::token::NoException;
use cfg_if::cfg_if;
use core::ptr::NonNull;
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
    java_vm: NonNull<jni_sys::JavaVM>,
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
    #[inline(always)]
    pub unsafe fn raw_jvm(&self) -> NonNull<jni_sys::JavaVM> {
        self.java_vm
    }

    /// Unsafe because one can pass an invalid `java_vm` pointer.
    pub(crate) unsafe fn from_ptr(java_vm: NonNull<jni_sys::JavaVM>) -> Self {
        Self { java_vm }
    }

    #[cfg(test)]
    pub(crate) fn test(ptr: *mut jni_sys::JavaVM) -> JavaVMRef {
        // It's fine if the VM is null in unit tests as they don't call the actual JNI API.
        JavaVMRef {
            java_vm: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    #[cfg(test)]
    pub(crate) fn test_default() -> JavaVMRef {
        JavaVMRef::test(1 as *mut jni_sys::JavaVM)
    }
}

#[cfg(test)]
mod java_vm_ref_tests {
    use super::*;

    #[test]
    fn raw_jvm() {
        let vm = JavaVMRef::test(0x1234 as *mut jni_sys::JavaVM);
        unsafe {
            assert_eq!(
                vm.raw_jvm(),
                NonNull::new_unchecked(0x1234 as *mut jni_sys::JavaVM)
            )
        };
    }
}

/// A struct for interacting with the Java VM. This struct owns the VM and will destroy it when
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
///
/// # Examples
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// use rust_jni::*;
/// use std::ptr;
///
/// let options = InitArguments::get_default(JniVersion::V8).unwrap()
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Gc))
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Jni));
///
/// let vm = JavaVM::create(&options).unwrap();
///
/// let vms = JavaVM::list().unwrap();
/// unsafe {
///     assert_eq!(vms[0].raw_jvm(), vm.raw_jvm());
/// }
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// [`JavaVM`](struct.JavaVM.html) is `Send + Sync`. It means it can be shared between threads.
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// use rust_jni::*;
/// use std::ptr;
/// use std::sync::Arc;
///
/// let vm = Arc::new(JavaVM::create(&InitArguments::default()).unwrap());
/// {
///     let vm = vm.clone();
///     ::std::thread::spawn(move || {
///         unsafe { vm.raw_jvm() };
///     });
/// }
/// unsafe { vm.raw_jvm() };
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
///
/// The main purpose of [`JavaVM`](struct.JavaVM.html) is to attach threads by provisioning
/// [`JniEnv`](struct.JniEnv.html)-s.
#[derive(Debug)]
pub struct JavaVM {
    java_vm: JavaVMRef,
    // This is just a hack for unit tests that don't actually call JNI.
    // Setting it to `false` allows to not `mem::forget` the value every time.
    #[cfg(test)]
    need_drop: bool,
}

impl JavaVM {
    /// Create a Java VM with the specified arguments.
    ///
    /// [Only one](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    /// Java VM per process is supported. When called for the second time will return an error.
    ///
    /// Currently this is the case even if the object is
    /// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed.
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
                // Should not fail because successfull `JNI_CreateJavaVM` call means the pointer is not null.
                let java_vm = NonNull::new(java_vm).unwrap();
                // We want to detach the current thread (which is automatically attached by JNI) because we want
                // to only allow attaching a thread once and the `attach` method will panic if the thread is already
                // attached. Detaching here makes this logic easier to implement.
                // Safe because `JNI_CreateJavaVM` returned OK and hence `java_vm`
                // is a valid `jni_sys::JavaVM` pointer and because `JNI_CreateJavaVM` attaches
                // the current thread.
                // [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/invocation.html#detachcurrentthread)
                // says trying to detach a thread that is not attached is a no-op.
                unsafe { Self::detach_or_error(java_vm) };

                Ok(Self {
                    java_vm: JavaVMRef { java_vm },
                    #[cfg(test)]
                    need_drop: true,
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
                        .map(|java_vm| unsafe {
                            // Should not fail because JNI_GetCreatedJavaVMs guarantees
                            // non-null Java VM pointers.
                            JavaVMRef::from_ptr(NonNull::new(java_vm).unwrap())
                        })
                        .collect()),
                    Some(error) => Err(error),
                }
            }
            Some(error) => Err(error),
        }
    }

    /// Unsafe because:
    /// 1. A user might pass an incorrect pointer.
    /// 2. The current thread might not be attached.
    pub(crate) unsafe fn detach_or_error(java_vm: NonNull<jni_sys::JavaVM>) {
        let error = JavaVM::detach(java_vm);
        // There is no way to recover from detach failure, except leak or fail.
        if error.is_some() {
            panic!(
                "Could not detach the current thread. Status: {:?}",
                error.unwrap()
            );
        }
    }

    /// Unsafe because:
    /// 1. A user might pass an incorrect pointer.
    /// 2. The current thread might not be attached.
    pub(crate) unsafe fn detach(java_vm: NonNull<jni_sys::JavaVM>) -> Option<JniError> {
        let detach_fn = (**java_vm.as_ptr()).DetachCurrentThread.unwrap();
        JniError::from_raw(detach_fn(java_vm.as_ptr()))
    }

    /// Get the raw Java VM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    ///
    /// [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/invocation.html#invocation-api-functions).
    #[inline(always)]
    pub unsafe fn raw_jvm(&self) -> NonNull<jni_sys::JavaVM> {
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
        self.with_attached_generic(env, closure)
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
        closure: impl for<'token> FnOnce(
            &'token JniEnv<'token>,
            NoException<'token>,
        ) -> (T, NoException<'token>),
    ) -> Result<T, JniError> {
        let env = self.attach_daemon(arguments)?;
        self.with_attached_generic(env, closure)
    }

    fn with_attached_generic<'vm, T>(
        &'vm self,
        env: JniEnv<'vm>,
        closure: impl for<'token> FnOnce(
            &'token JniEnv<'token>,
            NoException<'token>,
        ) -> (T, NoException<'token>),
    ) -> Result<T, JniError> {
        // Safe because we only get a single token here.
        let token = unsafe { env.token_internal() };
        let (result, token) = closure(&env, token);
        let token = token.consume();
        match env.detach(token) {
            None => Ok(result),
            Some(error) => Err(error),
        }
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
        unsafe {
            self.attach_generic(
                arguments,
                (**self.raw_jvm().as_ptr()).AttachCurrentThread.unwrap(),
            )
        }
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
                (**self.raw_jvm().as_ptr())
                    .AttachCurrentThreadAsDaemon
                    .unwrap(),
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
        let get_env_fn = (**self.raw_jvm().as_ptr()).GetEnv.unwrap();
        // Safe, because the arguments are correct.
        let error = JniError::from_raw(get_env_fn(
            self.raw_jvm().as_ptr(),
            (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
            arguments.version().to_raw(),
        ));
        match error {
            Some(JniError::ThreadDetached) => {
                let error = JniError::from_raw(attach_fn(
                    self.raw_jvm().as_ptr(),
                    (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
                    (&mut raw_arguments.raw_arguments) as *mut jni_sys::JavaVMAttachArgs
                        as *mut c_void,
                ));
                match error {
                    // Shuld not fail: successful call to AttachCurrentThread guarantees a non-null env pointer.
                    None => Ok(JniEnv::attached(
                        &self.java_vm,
                        NonNull::new(jni_env).unwrap(),
                    )),
                    Some(JniError::UnsupportedVersion) => panic!(
                        "Got upsupported version error when creating a Java VM. \
                         Should not happen as `InitArguments` are supposed to check \
                         for version support."
                    ),
                    Some(JniError::ThreadDetached) => {
                        panic!("Got `EDETACHED` when trying to attach a thread.")
                    }
                    // TODO(monnoroch): panic on more impossible errors.
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

    #[cfg(test)]
    pub(crate) fn test(ptr: *mut jni_sys::JavaVM) -> JavaVM {
        // It's fine if the VM is null in unit tests as they don't call the actual JNI API.
        JavaVM {
            java_vm: JavaVMRef::test(ptr),
            need_drop: false,
        }
    }
}

/// Implement [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
/// for [`JavaVM`](struct.JavaVM.html) to cast it to a reference to
/// [`JavaVMRef`](struct.JavaVMRef.html).
///
/// As [`JavaVM`](struct.JavaVM.html) will be destroyed when
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed, references to it's
/// [`JavaVMRef`-s](struct.JavaVMRef.html) should not outlive it.
/// This impl is not very useful and mostly serves as the documentation of this fact.
impl AsRef<JavaVMRef> for JavaVM {
    #[inline(always)]
    fn as_ref<'vm>(&'vm self) -> &'vm JavaVMRef {
        &self.java_vm
    }
}

/// Destroy [`JavaVM`](struct.JavaVM.html) when the value is
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#destroyjavavm)
impl Drop for JavaVM {
    fn drop(&mut self) {
        #[cfg(test)]
        {
            if !self.need_drop {
                return;
            }
        }

        // Safe because JavaVM can't be created from an invalid or non-owned Java VM pointer.
        let error = JniError::from_raw(unsafe {
            let destroy_fn = (**self.raw_jvm().as_ptr()).DestroyJavaVM.unwrap();
            destroy_fn(self.raw_jvm().as_ptr())
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

    #[test]
    fn raw_jvm() {
        let vm = JavaVM::test(0x1234 as *mut jni_sys::JavaVM);
        unsafe {
            assert_eq!(
                vm.raw_jvm(),
                NonNull::new_unchecked(0x1234 as *mut jni_sys::JavaVM)
            )
        };
    }

    #[test]
    fn as_ref() {
        let vm_ref = JavaVMRef::test(0x1234 as *mut jni_sys::JavaVM);
        let vm = JavaVM {
            java_vm: vm_ref,
            need_drop: false,
        };

        assert_eq!(vm.as_ref(), &vm_ref);
    }
}

#[cfg(test)]
mod java_vm_drop_tests {
    use super::*;
    use serial_test::serial;

    generate_java_vm_mock!(mock);

    #[test]
    #[serial]
    fn drop() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let destroy_vm_mock = mock::destroy_vm_context();
        {
            let mut vm = JavaVM::test(raw_java_vm_ptr);
            vm.need_drop = true;
            // Nothing has happened yet.
            destroy_vm_mock.checkpoint();
            destroy_vm_mock
                .expect()
                .times(1)
                .withf_st(move |x| *x == raw_java_vm_ptr)
                .return_const(jni_sys::JNI_OK);
        }
        // Expectations are checked after the scope has ended.
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Failed destroying the JavaVm. Status: Unknown(-1)")]
    fn drop_panics() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let destroy_vm_mock = mock::destroy_vm_context();
        destroy_vm_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR);
        {
            let mut vm = JavaVM::test(raw_java_vm_ptr);
            vm.need_drop = true;
        }
        // Nothing has happened.
        destroy_vm_mock.checkpoint();
    }
}

#[cfg(test)]
mod java_vm_create_tests {
    use super::*;
    use mockall::*;
    use serial_test::serial;
    use std::mem;

    generate_java_vm_mock!(mock);

    #[test]
    #[serial]
    fn create() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let create_vm_mock = jni_mock::JNI_CreateJavaVM_context();
        create_vm_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm, _jni_env, arguments| {
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
                                **java_vm = raw_java_vm_ptr;
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
            .withf_st(move |java_vm| *java_vm == raw_java_vm_ptr)
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let vm = JavaVM::create(&InitArguments::default()).unwrap();
        unsafe {
            assert_eq!(vm.raw_jvm(), NonNull::new(raw_java_vm_ptr).unwrap());
        }
        // Do not drop: we didn't mock the destructor.
        mem::forget(vm);
    }

    #[test]
    #[serial]
    fn create_error() {
        let create_vm_mock = jni_mock::JNI_CreateJavaVM_context();
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
        let create_vm_mock = jni_mock::JNI_CreateJavaVM_context();
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
        let create_vm_mock = jni_mock::JNI_CreateJavaVM_context();
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
    use mockall::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn list() {
        let raw_java_vm_ptr_1 = 0x1234 as *mut jni_sys::JavaVM;
        let raw_java_vm_ptr_2 = 0x1235 as *mut jni_sys::JavaVM;
        assert_ne!(raw_java_vm_ptr_1, raw_java_vm_ptr_2);

        let mut sequence = Sequence::new();
        let list_vms_mock = jni_mock::JNI_GetCreatedJavaVMs_context();
        list_vms_mock
            .expect()
            .times(1)
            .withf_st(move |java_vms, buffer_size, vms_count| {
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
            .withf_st(move |java_vms, buffer_size, vms_count| {
                if *buffer_size != 2 {
                    false
                } else {
                    unsafe {
                        **java_vms = raw_java_vm_ptr_1;
                        *((*java_vms).offset(1)) = raw_java_vm_ptr_2;
                        **vms_count = 2 as jni_sys::jint;
                    }
                    true
                }
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let vms = JavaVM::list().unwrap();
        unsafe {
            assert_eq!(vms[0].raw_jvm().as_ptr(), raw_java_vm_ptr_1);
            assert_eq!(vms[1].raw_jvm().as_ptr(), raw_java_vm_ptr_2);
        }
    }

    #[test]
    #[serial]
    fn list_error_first_call() {
        let list_vms_mock = jni_mock::JNI_GetCreatedJavaVMs_context();
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
        let list_vms_mock = jni_mock::JNI_GetCreatedJavaVMs_context();
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
mod java_vm_with_attached_tests {
    use super::*;
    use crate::java_string::from_java_string;
    use crate::version::JniVersion;
    use mockall::*;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::ffi::{c_void, CStr};

    generate_java_vm_mock!(mock);
    generate_jni_env_mock!(jni_mock);

    #[test]
    #[serial]
    fn with_attached() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm, _jni_env, version| {
                *java_vm == raw_java_vm_ptr && *version == jni_sys::JNI_VERSION_1_8
            })
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm, jni_env, argument| unsafe {
                let thread_name =
                    CStr::from_ptr((*(*argument as *mut jni_sys::JavaVMAttachArgs)).name)
                        .to_bytes_with_nul();
                if *java_vm != raw_java_vm_ptr
                    || from_java_string(thread_name).unwrap() != "test-name"
                {
                    return false;
                }
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_FALSE)
            .in_sequence(&mut sequence);
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm| *java_vm == raw_java_vm_ptr)
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        let result = vm
            .with_attached(
                &AttachArguments::named(JniVersion::V8, "test-name"),
                |env, token| {
                    unsafe {
                        assert_eq!(env.raw_jvm().as_ptr(), raw_java_vm_ptr);
                        assert_eq!(env.raw_env().as_ptr(), raw_env_ptr);
                    }
                    assert_eq!(env.has_token, RefCell::new(false));
                    (17, token)
                },
            )
            .unwrap();
        assert_eq!(result, 17);
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "GetEnv JNI method returned an unexpected error code Unknown(-1)")]
    fn with_attached_get_env_error() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.with_attached(&AttachArguments::new(JniVersion::V8), |_env, token| {
            ((), token)
        })
        .unwrap();
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Got `EDETACHED` when trying to attach a thread")]
    fn with_attached_cant_attach() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.with_attached(&AttachArguments::new(JniVersion::V8), |_env, token| {
            ((), token)
        })
        .unwrap();
    }

    #[test]
    #[serial]
    fn with_attached_attach_error() {
        let raw_env_ptr = 0x1234 as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |_java_vm, jni_env, _argument| unsafe {
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_ERR)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        let result = vm
            .with_attached(&AttachArguments::new(JniVersion::V8), |_env, token| {
                ((), token)
            })
            .unwrap_err();
        assert_eq!(result, JniError::Unknown(jni_sys::JNI_ERR));
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "upsupported version")]
    fn with_attached_unsupported_version() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EVERSION)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.with_attached(&AttachArguments::new(JniVersion::V8), |_env, token| {
            ((), token)
        })
        .unwrap();
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Newly attached thread has a pending exception")]
    fn with_attached_pending_exception() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |_java_vm, jni_env, _argument| unsafe {
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_TRUE)
            .in_sequence(&mut sequence);
        let exception_describe_mock = jni_mock::exception_describe_context();
        exception_describe_mock
            .expect()
            .times(1)
            .return_const(())
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.with_attached(&AttachArguments::new(JniVersion::V8), |_env, token| {
            ((), token)
        })
        .unwrap();
    }

    #[test]
    #[serial]
    fn with_attached_detach_error() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |_java_vm, jni_env, _argument| unsafe {
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_FALSE)
            .in_sequence(&mut sequence);
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        let result = vm
            .with_attached(&AttachArguments::new(JniVersion::V8), |_env, token| {
                ((), token)
            })
            .unwrap_err();
        assert_eq!(result, JniError::Unknown(jni_sys::JNI_ERR));
    }

    #[test]
    #[serial]
    fn with_attached_daemon() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |_java_vm, jni_env, _argument| unsafe {
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_FALSE)
            .in_sequence(&mut sequence);
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        let result = vm
            .with_attached(&AttachArguments::new(JniVersion::V8), |env, token| {
                unsafe {
                    assert_eq!(env.raw_jvm().as_ptr(), raw_java_vm_ptr);
                    assert_eq!(env.raw_env().as_ptr(), raw_env_ptr);
                }
                assert_eq!(env.has_token, RefCell::new(false));
                (17, token)
            })
            .unwrap();
        assert_eq!(result, 17);
    }
}

#[cfg(test)]
mod java_vm_attach_tests {
    use super::*;
    use crate::java_string::from_java_string;
    use crate::version::JniVersion;
    use mockall::*;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::ffi::{c_void, CStr};
    use std::mem;

    generate_java_vm_mock!(mock);
    generate_jni_env_mock!(jni_mock);

    #[test]
    #[serial]
    fn attach() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm, _jni_env, version| {
                *java_vm == raw_java_vm_ptr && *version == jni_sys::JNI_VERSION_1_8
            })
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm, jni_env, argument| unsafe {
                let thread_name =
                    CStr::from_ptr((*(*argument as *mut jni_sys::JavaVMAttachArgs)).name)
                        .to_bytes_with_nul();
                if *java_vm != raw_java_vm_ptr
                    || from_java_string(thread_name).unwrap() != "test-name"
                {
                    return false;
                }
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_FALSE)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        let env = vm
            .attach(&AttachArguments::named(JniVersion::V8, "test-name"))
            .unwrap();
        unsafe {
            assert_eq!(env.raw_jvm().as_ptr(), raw_java_vm_ptr);
            assert_eq!(env.raw_env().as_ptr(), raw_env_ptr);
        }
        assert_eq!(env.has_token, RefCell::new(true));
        // Don't want to drop a manually created `JniEnv` and `JavaVM`.
        mem::forget(env);
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "already attached")]
    fn attach_already_attached() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let get_env_mock = mock::get_env_context();
        get_env_mock.expect().times(1).return_const(jni_sys::JNI_OK);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "GetEnv JNI method returned an unexpected error code Unknown(-1)")]
    fn attach_get_env_error() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Got `EDETACHED` when trying to attach a thread")]
    fn attach_cant_attach() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    }

    #[test]
    #[serial]
    fn attach_attach_error() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_ERR)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        assert_eq!(
            vm.attach(&AttachArguments::new(JniVersion::V8))
                .unwrap_err(),
            JniError::Unknown(jni_sys::JNI_ERR as i32)
        );
    }

    #[test]
    #[serial]
    #[should_panic(expected = "upsupported version")]
    fn attach_unsupported_version() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EVERSION)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Newly attached thread has a pending exception")]
    fn attach_pending_exception() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_mock = mock::attach_current_thread_context();
        attach_current_thread_mock
            .expect()
            .times(1)
            .withf_st(move |_java_vm, jni_env, _argument| unsafe {
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_TRUE)
            .in_sequence(&mut sequence);
        let exception_describe_mock = jni_mock::exception_describe_context();
        exception_describe_mock
            .expect()
            .times(1)
            .return_const(())
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        vm.attach(&AttachArguments::new(JniVersion::V8)).unwrap();
    }

    #[test]
    #[serial]
    fn attach_daemon() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let mut sequence = Sequence::new();
        let get_env_mock = mock::get_env_context();
        get_env_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_EDETACHED)
            .in_sequence(&mut sequence);
        let attach_current_thread_as_daemon_mock = mock::attach_current_thread_as_daemon_context();
        attach_current_thread_as_daemon_mock
            .expect()
            .times(1)
            .withf_st(move |_java_vm, jni_env, _argument| unsafe {
                **jni_env = raw_env_ptr as *mut c_void;
                true
            })
            .return_const(jni_sys::JNI_OK)
            .in_sequence(&mut sequence);
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_FALSE)
            .in_sequence(&mut sequence);
        let vm = JavaVM::test(raw_java_vm_ptr);
        let env = vm
            .attach_daemon(&AttachArguments::new(JniVersion::V8))
            .unwrap();
        unsafe {
            assert_eq!(env.raw_jvm().as_ptr(), raw_java_vm_ptr);
            assert_eq!(env.raw_env().as_ptr(), raw_env_ptr);
        }
        assert_eq!(env.has_token, RefCell::new(true));
        // Don't want to drop a manually created `JniEnv` and `JavaVM`.
        mem::forget(env);
    }
}

cfg_if! {
    if #[cfg(any(test, feature = "mock-jvm"))] {
        generate_jni_functions_mock!(jni_mock);
    } else {
        use jni_sys::JNI_CreateJavaVM;
        use jni_sys::JNI_GetCreatedJavaVMs;
    }
}
