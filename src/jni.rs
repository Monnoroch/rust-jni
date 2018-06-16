use attach_arguments::{self, AttachArguments};
use init_arguments::{self, InitArguments};
#[cfg(test)]
use java_string::*;
use jni_sys;
use raw::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use version::{self, JniVersion};

/// Errors returned by JNI_CreateJavaVM and JNI_GetCreatedJavaVMs.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#return-codes)
// TODO(#17): add error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JniError {
    /// Unknown error.
    /// Needed for forward compability.
    Unknown(i32),
}

/// A token that represents that there is no pending Java exception in the current thread.
///
/// # Pending exceptions
///
/// When a JNI function is called, it can throw an exception. Then the current thread is said
/// to have a pending exception. Most JNI functions must not be called when there is a pending
/// exception. Read more about exception handling in
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/design.html#java-exceptions).
///
/// # Exception tokens
///
/// `rust-jni` tries to push as many programming errors as possible from run-time to compile-time.
/// To not allow caller to call JNI methods when there is a pending exception, these methods
/// will require the caller to provide a `NoException` token. The caller can obtain the token
/// after attaching the thread to the Java VM:
/// ```
/// use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
///
/// let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// ```
/// A token can not be obtained twice from a `JniEnv` value:
/// ```should_panic
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// let token = env.token(); // panics!
/// ```
/// A token also can not be obtained when there is a pending exception:
/// ```
/// // TODO: a should_panic example with a token obtained when there's a pending exception.
/// ```
/// Once obtained, the token can be used to call JNI methods:
/// ```
/// // TODO: example for a non-token-consuming method.
/// ```
/// Some JNI methods can throw exceptions themselves. In this case the token will be consumed:
/// ```
/// // TODO: example for a token-consuming method.
/// ```
/// Methods that consume the token will always return a `JniResult` value which will either
/// have a value and a new `NoException` token that can be used to call more JNI methods or
/// an `Exception` token:
/// ```
/// // TODO: example of a token-consuming method returning a new token.
/// ```
/// The token is bound to the `JniEnv` object, so it can't outlive it:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
///
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let token = {
///     let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
///     let token = env.token();
///     token
/// };
/// ```
/// Tokens that are returned from other methods as part of a `JniResult` are also bound to
/// the `JniEnv` object and can't outlive it:
/// ```
/// // TODO: a compile_fail example with a token, returned from a JNI method.
/// ```
/// If an `Exception` token was returned, it does not in fact mean, that there is
/// a pending exception, but it means that the `rust-jni` library can not prove that there
/// isn't one without explicitly calling the `ExceptionCheck` JNI method. Thus, the
/// `Exception` token can be `unwrap`-ped into a new `NoException` token and an optional
/// `Throwable` value, if there was, in fact, a pending exception. Unwrapping
/// the `Exception` token will clear the pending exception, so it is again safe to call JNI
/// methods:
/// ```
/// // TODO: an example for `Exception::unwrap`.
/// ```
/// Because the `Exception` token doesn't mean that there is definitely a pending exception,
/// `unwrap`-ping it can, in fact, return a `None` instead of a `Some(Throwable)`:
/// ```
/// // TODO: example of `NewLocalRef` returning `null` for a garbage collected local reference.
/// // but no exception occuring.
/// ```
///
/// # Error handling in Java method calls
///
/// Calling methods on Java objects is slightly different. To follow the Java semantics,
/// that a method either returns a result or throws an exception, all Java methods return
/// a `JavaResult` value, which is either an actual result or a `Throwable` value representing
/// the exception. Java methods never leave a pending exception, so they never consume the
/// `NoException` token, but they always require it to be presented:
/// ```
/// // TODO: an example of a non throwing Java method call.
/// ```
/// ```
/// // TODO: an example of a throwing Java method call.
/// ```
#[derive(Debug)]
pub struct NoException<'env> {
    _token: (),
    _env: PhantomData<JniEnv<'env>>,
}

impl<'env> NoException<'env> {
    /// Unsafe because it creates a new no-exception token when there might be a pending exception.
    unsafe fn new_env<'a>(_env: &JniEnv<'a>) -> NoException<'a> {
        // Safe because this function ensures correct lifetimes.
        Self::new_raw()
    }

    /// Unsafe because:
    /// 1. It creates a new no-exception token when there might be a pending exception
    /// 2. Doesn't ensure a correct lifetime
    unsafe fn new_raw<'a>() -> NoException<'a> {
        NoException {
            _token: (),
            _env: PhantomData::<JniEnv>,
        }
    }
}

/// `NoException` can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for NoException<'env> {}
// impl<'env> !Sync for NoException<'env> {}

/// A dual token to `NoException` that represents that there might be a pending exception in Java.
///
/// Read more about tokens in `NoException` documentation.
#[derive(Debug)]
pub struct Exception<'env> {
    _token: (),
    _env: PhantomData<JniEnv<'env>>,
}

/// `Exception` can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for NoException<'env> {}
// impl<'env> !Sync for NoException<'env> {}

/// A result of a JNI function. Either a value and a `NoException` token, when the function
/// didn't throw an exception or an `Exception` token when it did or it is unknown if it did.
pub type JniResult<'env, T> = Result<(T, NoException<'env>), Exception<'env>>;

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
/// The main purpose of `JavaVM` is to attach threads by provisioning `JniEnv`-s.
#[derive(Debug)]
pub struct JavaVM {
    java_vm: *mut jni_sys::JavaVM,
    owned: bool,
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
        let mut raw_arguments =
            init_arguments::to_raw(&arguments, &mut strings_buffer, &mut options_buffer);
        // Safe because we pass pointers to correct data structures.
        let status = unsafe {
            JNI_CreateJavaVM(
                (&mut java_vm) as *mut *mut jni_sys::JavaVM,
                (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
                &mut raw_arguments.raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
            )
        };
        match status {
            jni_sys::JNI_OK => {
                // We want to detach the current thread because we want to only allow attaching
                // a thread once and the `attach` method will panic if the thread is already
                // attached. Detaching here makes this logic easier to implement.
                // Safe because `JNI_CreateJavaVM` returned OK and hence `java_vm`
                // is a valid `jni_sys::JavaVM` pointer and because `JNI_CreateJavaVM` attaches
                // the current thread.
                unsafe { Self::detach(java_vm) };

                Ok(Self {
                    java_vm,
                    owned: true,
                })
            }
            jni_sys::JNI_EVERSION => panic!(
                "Got upsupported version error when creating a Java VM. \
                 Should not happen as `InitArguments` are supposed to check \
                 for version support."
            ),
            jni_sys::JNI_EDETACHED => {
                panic!("Unexpected `EDETACHED` error when creating a Java VM.")
            }
            status => Err(JniError::Unknown(status)),
        }
    }

    /// Get a list of created Java VMs.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getcreatedjavavms)
    pub fn list() -> Result<Vec<Self>, JniError> {
        let mut vms_created: jni_sys::jsize = 0;
        // Safe because arguments are correct.
        let status = unsafe {
            JNI_GetCreatedJavaVMs(
                ::std::ptr::null_mut(),
                0,
                (&mut vms_created) as *mut jni_sys::jsize,
            )
        };
        match status {
            jni_sys::JNI_OK => {
                let mut java_vms: Vec<*mut jni_sys::JavaVM> = vec![];
                java_vms.resize(vms_created as usize, ::std::ptr::null_mut());
                let mut tmp: jni_sys::jsize = 0;
                // Safe because arguments are ensured to be correct.
                let status = unsafe {
                    JNI_GetCreatedJavaVMs(
                        (java_vms.as_mut_ptr()) as *mut *mut jni_sys::JavaVM,
                        vms_created,
                        // Technically, a new VM could have been created since the previous call to
                        // `JNI_GetCreatedJavaVMs`. But then we also technically should not return
                        // any new ones, because they weren't there wneh this function was called.
                        (&mut tmp) as *mut jni_sys::jsize,
                    )
                };
                match status {
                    jni_sys::JNI_OK => Ok(java_vms
                        .iter()
                        .cloned()
                        .map(|java_vm| JavaVM {
                            java_vm,
                            owned: false,
                        })
                        .collect()),
                    status => Err(JniError::Unknown(status)),
                }
            }
            status => Err(JniError::Unknown(status)),
        }
    }

    /// Get the raw JavaVM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    pub unsafe fn raw_jvm(&self) -> *mut jni_sys::JavaVM {
        self.java_vm
    }

    /// Attach the current thread to the Java VM with a specific thread name.
    /// Returns a `JniEnv` instance and a `NoException` token that can be used to call JNI methods.
    ///
    /// Read more about tokens in docs for `NoException`.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn attach(&self, arguments: &AttachArguments) -> Result<JniEnv, JniError> {
        // Safe because the argument is ensured to be the correct method.
        unsafe { self.attach_generic(arguments, (**self.raw_jvm()).AttachCurrentThread.unwrap()) }
    }

    /// Attach the current thread to the Java VM as a daemon with a specific thread name.
    /// Returns a `JniEnv` instance and a `NoException` token that can be used to call JNI methods.
    ///
    /// Read more about tokens in docs for `NoException`.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/9/docs/specs/jni/invocation.html#attachcurrentthreadasdaemon)
    pub fn attach_daemon(&self, arguments: &AttachArguments) -> Result<JniEnv, JniError> {
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
        let mut raw_arguments = attach_arguments::to_raw(arguments, &mut buffer);
        let mut jni_env: *mut jni_sys::JNIEnv = ::std::ptr::null_mut();
        let get_env_fn = (**self.raw_jvm()).GetEnv.unwrap();
        // Safe, because the arguments are correct.
        let status = get_env_fn(
            self.raw_jvm(),
            (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
            version::to_raw(arguments.version()),
        );
        match status {
            jni_sys::JNI_EDETACHED => {
                let status = attach_fn(
                    self.raw_jvm(),
                    (&mut jni_env) as *mut *mut jni_sys::JNIEnv as *mut *mut c_void,
                    (&mut raw_arguments.raw_arguments) as *mut jni_sys::JavaVMAttachArgs
                        as *mut c_void,
                );
                match status {
                    jni_sys::JNI_OK => {
                        let env = JniEnv {
                            version: arguments.version(),
                            vm: self,
                            jni_env,
                            has_token: RefCell::new(true),
                        };
                        if env.has_exception() {
                            // We don't want to drop `JniEnv` with a pending exception.
                            mem::forget(env);
                            panic!("Newly attached thread has a pending exception.");
                        }
                        Ok(env)
                    }
                    jni_sys::JNI_EVERSION => panic!(
                        "Got upsupported version error when creating a Java VM. \
                         Should not happen as `InitArguments` are supposed to check \
                         for version support."
                    ),
                    jni_sys::JNI_EDETACHED => {
                        panic!("Got `EDETACHED` when trying to attach a thread.")
                    }
                    // TODO: panic on more impossible errors.
                    status => Err(JniError::Unknown(status)),
                }
            }
            jni_sys::JNI_OK => panic!(
                "This thread is already attached to the JVM. \
                 Attaching a thread twice is not allowed."
            ),
            // According to the
            // [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#getenv),
            // can only returd `OK`, `EDETACHED` and `EVERSION`.
            // Will not return `EVERSION` here, because the version was already checked when
            // creating the Java VM.
            status => panic!(
                "GetEnv JNI method returned an unexpected error code {}",
                status
            ),
        }
    }

    /// Unsafe because:
    /// 1. A user might pass an incorrect pointer.
    /// 2. The current thread might not be attached.
    unsafe fn detach(java_vm: *mut jni_sys::JavaVM) {
        let detach_fn = (**java_vm).DetachCurrentThread.unwrap();
        let status = detach_fn(java_vm);
        // There is no way to recover from detach failure, except leak or fail.
        if status != jni_sys::JNI_OK {
            panic!("Could not detach the current thread. Status: {}", status)
        }
    }
}

/// Make `JavaVM` be destroyed when the value is dropped.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#destroyjavavm)
impl Drop for JavaVM {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }

        // Safe because the argument is ensured to be the correct by construction.
        let status = unsafe {
            let destroy_fn = (**self.java_vm).DestroyJavaVM.unwrap();
            destroy_fn(self.java_vm)
        };

        if status != jni_sys::JNI_OK {
            panic!("Failed destroying the JavaVm. Status: {}", status);
        }
    }
}

/// Make `JavaVM` sendable between threads. Guaranteed to be safe by JNI.
unsafe impl Send for JavaVM {}

/// Make `JavaVM` shareable by multiple threads. Guaranteed to be safe by JNI.
unsafe impl Sync for JavaVM {}

#[cfg(test)]
mod java_vm_tests {
    use super::*;
    use init_arguments;
    use std::ffi::CStr;
    use std::mem;
    use testing::*;

    fn default_args() -> InitArguments {
        init_arguments::tests::default_args()
    }

    #[test]
    fn create() {
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
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let _locked =
            setup_create_java_vm_call(CreateJavaVMCall::new(jni_sys::JNI_OK, raw_java_vm_ptr));
        let arguments = default_args();
        let vm = JavaVM::create(&arguments).unwrap();
        assert_eq!(vm.java_vm, raw_java_vm_ptr);
        assert_eq!(arguments, get_create_java_vm_call_input());
        unsafe {
            assert_eq!(DETACH_CALLS, 1);
            assert_eq!(DETACH_ARGUMENT, raw_java_vm_ptr);
        };
        mem::forget(vm);
    }

    #[test]
    #[should_panic(expected = "Could not detach the current thread. Status: -1")]
    fn create_detach_error() {
        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let _locked =
            setup_create_java_vm_call(CreateJavaVMCall::new(jni_sys::JNI_OK, raw_java_vm_ptr));
        JavaVM::create(&default_args()).unwrap();
    }

    #[test]
    #[should_panic(expected = "upsupported version")]
    fn create_version_error() {
        let raw_java_vm = 0x1234 as *mut jni_sys::JavaVM;
        let _locked =
            setup_create_java_vm_call(CreateJavaVMCall::new(jni_sys::JNI_EVERSION, raw_java_vm));
        let arguments = default_args();
        let _ = JavaVM::create(&arguments);
    }

    #[test]
    #[should_panic(expected = "Unexpected `EDETACHED`")]
    fn create_detached_error() {
        let raw_java_vm = 0x1234 as *mut jni_sys::JavaVM;
        let _locked =
            setup_create_java_vm_call(CreateJavaVMCall::new(jni_sys::JNI_EDETACHED, raw_java_vm));
        let arguments = default_args();
        let _ = JavaVM::create(&arguments);
    }

    #[test]
    fn create_error() {
        let raw_java_vm = 0x1234 as *mut jni_sys::JavaVM;
        let _locked =
            setup_create_java_vm_call(CreateJavaVMCall::new(jni_sys::JNI_ERR, raw_java_vm));
        let arguments = default_args();
        assert_eq!(
            JavaVM::create(&arguments).unwrap_err(),
            JniError::Unknown(jni_sys::JNI_ERR as i32),
        );
    }

    #[test]
    fn drop() {
        static mut DESTROY_CALLS: i32 = 0;
        static mut DESTROY_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        unsafe extern "system" fn destroy_vm(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint {
            DESTROY_CALLS += 1;
            DESTROY_ARGUMENT = java_vm;
            jni_sys::JNI_OK
        }

        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DestroyJavaVM: Some(destroy_vm),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        {
            let _vm = JavaVM {
                java_vm: raw_java_vm_ptr,
                owned: true,
            };
            unsafe { assert_eq!(DESTROY_CALLS, 0) };
        }
        unsafe {
            assert_eq!(DESTROY_CALLS, 1);
            assert_eq!(DESTROY_ARGUMENT, raw_java_vm_ptr);
        };
    }

    #[test]
    fn drop_not_owned() {
        static mut DESTROY_CALLS: i32 = 0;
        static mut DESTROY_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        unsafe extern "system" fn destroy_vm(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint {
            DESTROY_CALLS += 1;
            DESTROY_ARGUMENT = java_vm;
            jni_sys::JNI_OK
        }

        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DestroyJavaVM: Some(destroy_vm),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        {
            let _vm = JavaVM {
                java_vm: raw_java_vm_ptr,
                owned: false,
            };
        }
        unsafe {
            assert_eq!(DESTROY_CALLS, 0);
        };
    }

    #[test]
    #[should_panic(expected = "Failed destroying the JavaVm. Status: -1")]
    fn drop_destroy_error() {
        unsafe extern "system" fn destroy_vm(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DestroyJavaVM: Some(destroy_vm),
            ..empty_raw_java_vm()
        };
        let raw_java_vm = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        JavaVM {
            java_vm: raw_java_vm,
            owned: true,
        };
    }

    #[test]
    fn list() {
        let raw_java_vm_ptr0 = 0x1234 as *mut jni_sys::JavaVM;
        let raw_java_vm_ptr1 = 0x5678 as *mut jni_sys::JavaVM;
        let mut java_vm_ptrs: [*mut jni_sys::JavaVM; 2] = [raw_java_vm_ptr0, raw_java_vm_ptr1];
        let _locked = setup_get_created_java_vms_call(GetCreatedJavaVMsCall::new(
            jni_sys::JNI_OK,
            2,
            java_vm_ptrs.as_mut_ptr(),
        ));
        let vms = JavaVM::list().unwrap();
        assert_eq!(vms[0].java_vm, raw_java_vm_ptr0);
        assert_eq!(vms[1].java_vm, raw_java_vm_ptr1);
    }

    #[test]
    fn list_error_count() {
        let _locked = setup_get_created_java_vms_call(GetCreatedJavaVMsCall::new(
            jni_sys::JNI_ERR,
            0,
            ptr::null_mut(),
        ));
        assert_eq!(
            JavaVM::list().unwrap_err(),
            JniError::Unknown(jni_sys::JNI_ERR as i32)
        );
    }

    #[test]
    fn list_error_list() {
        let raw_java_vm_ptr0 = 0x1234 as *mut jni_sys::JavaVM;
        let raw_java_vm_ptr1 = 0x5678 as *mut jni_sys::JavaVM;
        let mut java_vm_ptrs: [*mut jni_sys::JavaVM; 2] = [raw_java_vm_ptr0, raw_java_vm_ptr1];
        let _locked = setup_get_created_java_vms_call(GetCreatedJavaVMsCall::new_twice(
            jni_sys::JNI_OK,
            jni_sys::JNI_ERR,
            2,
            java_vm_ptrs.as_mut_ptr(),
        ));
        assert_eq!(
            JavaVM::list().unwrap_err(),
            JniError::Unknown(jni_sys::JNI_ERR as i32)
        );
    }

    #[test]
    fn raw_vm() {
        let raw_java_vm = 0x1234 as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm,
            owned: false,
        };
        unsafe {
            assert_eq!(vm.raw_jvm(), raw_java_vm);
        }
        mem::forget(vm);
    }

    #[test]
    fn attach() {
        static mut GET_ENV_CALLS: i32 = 0;
        static mut GET_ENV_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        static mut GET_ENV_VERSION_ARGUMENT: jni_sys::jint = 0;
        unsafe extern "system" fn get_env(
            java_vm: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            version: jni_sys::jint,
        ) -> jni_sys::jint {
            GET_ENV_CALLS += 1;
            GET_ENV_VM_ARGUMENT = java_vm;
            GET_ENV_VERSION_ARGUMENT = version;
            jni_sys::JNI_EDETACHED
        }

        static mut ATTACH_CALLS: i32 = 0;
        static mut ATTACH_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        static mut ATTACH_ENV_ARGUMENT: *mut c_void = ptr::null_mut();
        static mut ATTACH_ARGUMENT: *mut c_void = ptr::null_mut();
        unsafe extern "system" fn attach(
            java_vm: *mut jni_sys::JavaVM,
            jni_env: *mut *mut c_void,
            argument: *mut c_void,
        ) -> jni_sys::jint {
            *jni_env = ATTACH_ENV_ARGUMENT;
            ATTACH_CALLS += 1;
            ATTACH_VM_ARGUMENT = java_vm;
            ATTACH_ARGUMENT = argument;
            jni_sys::JNI_OK
        }

        static mut EXCEPTION_CHECK_CALLS: i32 = 0;
        static mut EXCEPTION_CHECK_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_check(
            jni_env: *mut jni_sys::JNIEnv,
        ) -> jni_sys::jboolean {
            EXCEPTION_CHECK_CALLS += 1;
            EXCEPTION_CHECK_ARGUMENT = jni_env;
            jni_sys::JNI_FALSE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        let init_arguments = init_arguments::new(JniVersion::V8);
        unsafe {
            ATTACH_ENV_ARGUMENT = raw_jni_env as *mut c_void;
        }
        let env = vm.attach(&AttachArguments::named(&init_arguments, "test-name"))
            .unwrap();
        unsafe {
            assert_eq!(GET_ENV_CALLS, 1);
            assert_eq!(GET_ENV_VM_ARGUMENT, raw_java_vm_ptr);
            assert_eq!(GET_ENV_VERSION_ARGUMENT, version::to_raw(JniVersion::V8));
            assert_eq!(ATTACH_CALLS, 1);
            assert_eq!(ATTACH_VM_ARGUMENT, raw_java_vm_ptr);
            assert_eq!(
                from_java_string(
                    CStr::from_ptr((*(ATTACH_ARGUMENT as *mut jni_sys::JavaVMAttachArgs)).name)
                        .to_bytes_with_nul()
                ).unwrap(),
                "test-name"
            );
            assert_eq!(EXCEPTION_CHECK_CALLS, 1);
            assert_eq!(EXCEPTION_CHECK_ARGUMENT, raw_jni_env);
            assert_eq!(env.raw_jvm(), raw_java_vm_ptr);
            assert_eq!(env.raw_env(), raw_jni_env);
        }
        assert_eq!(env.has_token, RefCell::new(true));
        // Don't want to drop a manually created `JniEnv`.
        mem::forget(env);
    }

    #[test]
    #[should_panic(expected = "already attached")]
    fn attach_already_attached() {
        unsafe extern "system" fn get_env(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: jni_sys::jint,
        ) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        unsafe extern "system" fn attach(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "GetEnv JNI method returned an unexpected error code -1")]
    fn attach_get_env_error() {
        unsafe extern "system" fn get_env(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: jni_sys::jint,
        ) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        unsafe extern "system" fn attach(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Got `EDETACHED` when trying to attach a thread")]
    fn attach_cant_attach() {
        unsafe extern "system" fn get_env(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: jni_sys::jint,
        ) -> jni_sys::jint {
            jni_sys::JNI_EDETACHED
        }
        unsafe extern "system" fn attach(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint {
            jni_sys::JNI_EDETACHED
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "upsupported version")]
    fn attach_unsupported_version() {
        unsafe extern "system" fn get_env(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: jni_sys::jint,
        ) -> jni_sys::jint {
            jni_sys::JNI_EDETACHED
        }
        unsafe extern "system" fn attach(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint {
            jni_sys::JNI_EVERSION
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
            .unwrap();
    }

    #[test]
    fn attach_attach_error() {
        unsafe extern "system" fn get_env(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: jni_sys::jint,
        ) -> jni_sys::jint {
            jni_sys::JNI_EDETACHED
        }
        unsafe extern "system" fn attach(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        assert_eq!(
            vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
                .unwrap_err(),
            JniError::Unknown(jni_sys::JNI_ERR as i32)
        );
    }

    #[test]
    #[should_panic(expected = "Newly attached thread has a pending exception")]
    fn attach_pending_exception() {
        unsafe extern "system" fn get_env(
            _: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            _: jni_sys::jint,
        ) -> jni_sys::jint {
            jni_sys::JNI_EDETACHED
        }

        static mut ATTACH_ENV_ARGUMENT: *mut c_void = ptr::null_mut();
        unsafe extern "system" fn attach(
            _: *mut jni_sys::JavaVM,
            jni_env: *mut *mut c_void,
            _: *mut c_void,
        ) -> jni_sys::jint {
            *jni_env = ATTACH_ENV_ARGUMENT;
            jni_sys::JNI_OK
        }

        unsafe extern "system" fn exception_check(_: *mut jni_sys::JNIEnv) -> jni_sys::jboolean {
            jni_sys::JNI_TRUE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        unsafe {
            ATTACH_ENV_ARGUMENT = raw_jni_env as *mut c_void;
        }
        vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
            .unwrap();
    }

    #[test]
    fn attach_daemon() {
        static mut GET_ENV_CALLS: i32 = 0;
        static mut GET_ENV_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        static mut GET_ENV_VERSION_ARGUMENT: jni_sys::jint = 0;
        unsafe extern "system" fn get_env(
            java_vm: *mut jni_sys::JavaVM,
            _: *mut *mut c_void,
            version: jni_sys::jint,
        ) -> jni_sys::jint {
            GET_ENV_CALLS += 1;
            GET_ENV_VM_ARGUMENT = java_vm;
            GET_ENV_VERSION_ARGUMENT = version;
            jni_sys::JNI_EDETACHED
        }

        static mut ATTACH_CALLS: i32 = 0;
        static mut ATTACH_VM_ARGUMENT: *mut jni_sys::JavaVM = ptr::null_mut();
        static mut ATTACH_ENV_ARGUMENT: *mut c_void = ptr::null_mut();
        static mut ATTACH_ARGUMENT: *mut c_void = ptr::null_mut();
        unsafe extern "system" fn attach(
            java_vm: *mut jni_sys::JavaVM,
            jni_env: *mut *mut c_void,
            argument: *mut c_void,
        ) -> jni_sys::jint {
            *jni_env = ATTACH_ENV_ARGUMENT;
            ATTACH_CALLS += 1;
            ATTACH_VM_ARGUMENT = java_vm;
            ATTACH_ARGUMENT = argument;
            jni_sys::JNI_OK
        }

        static mut EXCEPTION_CHECK_CALLS: i32 = 0;
        static mut EXCEPTION_CHECK_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_check(
            jni_env: *mut jni_sys::JNIEnv,
        ) -> jni_sys::jboolean {
            EXCEPTION_CHECK_CALLS += 1;
            EXCEPTION_CHECK_ARGUMENT = jni_env;
            jni_sys::JNI_FALSE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThreadAsDaemon: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        let init_arguments = init_arguments::new(JniVersion::V8);
        unsafe {
            ATTACH_ENV_ARGUMENT = raw_jni_env as *mut c_void;
        }
        let env = vm.attach_daemon(&AttachArguments::named(&init_arguments, "test-name"))
            .unwrap();
        unsafe {
            assert_eq!(GET_ENV_CALLS, 1);
            assert_eq!(GET_ENV_VM_ARGUMENT, raw_java_vm_ptr);
            assert_eq!(GET_ENV_VERSION_ARGUMENT, version::to_raw(JniVersion::V8));
            assert_eq!(ATTACH_CALLS, 1);
            assert_eq!(ATTACH_VM_ARGUMENT, raw_java_vm_ptr);
            assert_eq!(
                from_java_string(
                    CStr::from_ptr((*(ATTACH_ARGUMENT as *mut jni_sys::JavaVMAttachArgs)).name)
                        .to_bytes_with_nul()
                ).unwrap(),
                "test-name"
            );
            assert_eq!(EXCEPTION_CHECK_CALLS, 1);
            assert_eq!(EXCEPTION_CHECK_ARGUMENT, raw_jni_env);
            assert_eq!(env.raw_jvm(), raw_java_vm_ptr);
            assert_eq!(env.raw_env(), raw_jni_env);
        }
        assert_eq!(env.has_token, RefCell::new(true));
        // Don't want to drop a manually created `JniEnv`.
        mem::forget(env);
    }
}

/// The interface for interacting with JNI.
/// All calls to Java are performed through this interface.
/// JNI methods can only be called from threads, explicitly attached to the Java VM.
/// `JniEnv` represents such a thread.
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
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// unsafe {
///     assert_ne!(env.raw_env(), ptr::null_mut());
/// }
/// ```
/// `JniEnv` is `!Send`. It means it can't be passed between threads:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
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
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// {
///     let vm = vm.clone();
///     ::std::thread::spawn(move || {
///         let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
///         unsafe {
///             assert_ne!(env.raw_env(), ptr::null_mut());
///         }
///     });
/// }
/// unsafe {
///     assert_ne!(env.raw_env(), ptr::null_mut());
/// }
/// ```
/// The thread is automatically detached once the `JniEnv` is dropped.
///
/// `JniEnv` can't outlive the parent `JavaVM`. This code is not allowed:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// let env = {
///     let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
///     let vm = JavaVM::create(&init_arguments).unwrap();
///     vm.attach(&AttachArguments::new(&init_arguments)).unwrap() // doesn't compile!
/// };
/// ```
/// `JniEnv` represents a thread, attached to the Java VM. Thus there can't be two `JniEnv`-s
/// per thread. `JavaVM::attach` will panic if you attempt to do so:
/// ```should_panic
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniEnv, JniVersion};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap(); // panics!
/// ```
// TODO: docs about panicing on detach when there's a pending exception.
#[derive(Debug)]
pub struct JniEnv<'vm> {
    version: JniVersion,
    vm: &'vm JavaVM,
    jni_env: *mut jni_sys::JNIEnv,
    has_token: RefCell<bool>,
}

// JniEnv can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'vm> !Send for JniEnv<'vm> {}
// impl<'vm> !Sync for JniEnv<'vm> {}

macro_rules! call_jni_method {
    ($env:expr, $method:ident) => {
        {
            let raw_env = $env.raw_env();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env)
        }
    };
    ($env:expr, $method:ident, $($argument:expr),*) => {
        {
            let raw_env = $env.raw_env();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env, $($argument),*)
        }
    };
}

impl<'vm> JniEnv<'vm> {
    /// Get the raw JavaVM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    pub unsafe fn raw_jvm(&self) -> *mut jni_sys::JavaVM {
        self.vm.raw_jvm()
    }

    /// Get the raw JNI Env pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    pub unsafe fn raw_env(&self) -> *mut jni_sys::JNIEnv {
        self.jni_env
    }

    /// Get a `NoException` token indicating that there is no pending exception
    /// in this thread.
    ///
    /// Read more about tokens in `NoException` documentation.
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
            unsafe { NoException::new_env(self) }
        }
    }

    fn has_exception(&self) -> bool {
        // Safe because the argument is ensured to be the correct by construction.
        let value = unsafe { call_jni_method!(self, ExceptionCheck) };
        init_arguments::to_bool(value)
    }
}

/// `Drop` detaches the current thread from the Java VM.
/// [It's not safe](https://developer.android.com/training/articles/perf-jni#exceptions)
/// to do so with an exception pending, so it panics if this happens.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#detachcurrentthread)
impl<'vm> Drop for JniEnv<'vm> {
    fn drop(&mut self) {
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
mod jni_env_tests {
    use super::*;
    use std::mem;
    use testing::*;

    fn empty_vm() -> JavaVM {
        JavaVM {
            java_vm: 0x1234 as *mut jni_sys::JavaVM,
            owned: false,
        }
    }

    #[test]
    fn raw_jvm() {
        let vm = empty_vm();
        let env = JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: ptr::null_mut(),
            has_token: RefCell::new(true),
        };
        unsafe {
            assert_eq!(env.raw_jvm(), vm.raw_jvm());
        }
        // Don't want to drop a manually created `JniEnv`.
        mem::forget(env);
    }

    #[test]
    fn raw_env() {
        let vm = empty_vm();
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env,
            has_token: RefCell::new(true),
        };
        unsafe {
            assert_eq!(env.raw_env(), jni_env);
        }
        // Don't want to drop a manually created `JniEnv`.
        mem::forget(env);
    }

    #[test]
    fn drop() {
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
        let vm = JavaVM {
            java_vm: &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM,
            owned: false,
        };

        static mut EXCEPTION_CHECK_CALLS: i32 = 0;
        static mut EXCEPTION_CHECK_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_check(
            jni_env: *mut jni_sys::JNIEnv,
        ) -> jni_sys::jboolean {
            EXCEPTION_CHECK_CALLS += 1;
            EXCEPTION_CHECK_ARGUMENT = jni_env;
            jni_sys::JNI_FALSE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        {
            let _env = JniEnv {
                version: JniVersion::V8,
                vm: &vm,
                jni_env: raw_jni_env,
                has_token: RefCell::new(true),
            };
            unsafe {
                assert_eq!(DETACH_CALLS, 0);
            }
        }
        unsafe {
            assert_eq!(DETACH_CALLS, 1);
            assert_eq!(DETACH_ARGUMENT, vm.java_vm);
            assert_eq!(EXCEPTION_CHECK_CALLS, 1);
            assert_eq!(EXCEPTION_CHECK_ARGUMENT, raw_jni_env);
        }
    }

    #[test]
    #[should_panic(expected = "Dropping `JniEnv` with a pending exception is not allowed")]
    fn drop_exception_pending() {
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
        let vm = JavaVM {
            java_vm: &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM,
            owned: false,
        };

        unsafe extern "system" fn exception_check(_: *mut jni_sys::JNIEnv) -> jni_sys::jboolean {
            jni_sys::JNI_TRUE
        }
        static mut EXCEPTION_DESCRIBE_CALLS: i32 = 0;
        static mut EXCEPTION_DESCRIBE_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_describe(jni_env: *mut jni_sys::JNIEnv) {
            EXCEPTION_DESCRIBE_CALLS += 1;
            EXCEPTION_DESCRIBE_ARGUMENT = jni_env;
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ExceptionDescribe: Some(exception_describe),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        struct Test {
            raw_jni_env: *mut jni_sys::JNIEnv,
        }
        impl Drop for Test {
            fn drop(&mut self) {
                unsafe {
                    assert_eq!(1, EXCEPTION_DESCRIBE_CALLS);
                    assert_eq!(self.raw_jni_env, EXCEPTION_DESCRIBE_ARGUMENT);
                }
            }
        }
        // _test.drop() will be called to check the `ExceptionDescribe` call after the panic.
        let _test = Test { raw_jni_env };

        JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: raw_jni_env,
            has_token: RefCell::new(true),
        };
    }

    #[test]
    #[should_panic(expected = "Could not detach the current thread. Status: -1")]
    fn drop_detach_error() {
        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = JavaVM {
            java_vm: &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM,
            owned: false,
        };

        unsafe extern "system" fn exception_check(_: *mut jni_sys::JNIEnv) -> jni_sys::jboolean {
            jni_sys::JNI_FALSE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv,
            has_token: RefCell::new(true),
        };
    }

    #[test]
    fn token() {
        static mut EXCEPTION_CHECK_CALLS: i32 = 0;
        static mut EXCEPTION_CHECK_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_check(
            jni_env: *mut jni_sys::JNIEnv,
        ) -> jni_sys::jboolean {
            EXCEPTION_CHECK_CALLS += 1;
            EXCEPTION_CHECK_ARGUMENT = jni_env;
            jni_sys::JNI_FALSE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        let raw_java_vm_ptr = 0x1234 as *mut jni_sys::JavaVM;
        let vm = JavaVM {
            java_vm: raw_java_vm_ptr,
            owned: false,
        };
        let env = JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: raw_jni_env,
            has_token: RefCell::new(true),
        };
        env.token();
        unsafe {
            assert_eq!(EXCEPTION_CHECK_CALLS, 1);
            assert_eq!(EXCEPTION_CHECK_ARGUMENT, raw_jni_env);
        }
        assert_eq!(env.has_token, RefCell::new(false));
        // Don't want to drop a manually created `JniEnv`.
        mem::forget(env);
    }

    #[test]
    #[should_panic(expected = "Trying to obtain a second `NoException` token from the `JniEnv`")]
    fn token_twice() {
        unsafe extern "system" fn exception_check(_: *mut jni_sys::JNIEnv) -> jni_sys::jboolean {
            jni_sys::JNI_FALSE
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = JavaVM {
            java_vm: &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM,
            owned: false,
        };
        let env = JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: raw_jni_env,
            has_token: RefCell::new(false),
        };
        env.token();
    }

    #[test]
    #[should_panic(
        expected = "Trying to obtain a `NoException` token when there is a pending exception"
    )]
    fn token_pending_exception() {
        static mut EXCEPTION_CHECK_CALLS: i32 = 0;
        unsafe extern "system" fn exception_check(_: *mut jni_sys::JNIEnv) -> jni_sys::jboolean {
            if EXCEPTION_CHECK_CALLS == 0 {
                EXCEPTION_CHECK_CALLS += 1;
                jni_sys::JNI_TRUE
            } else {
                jni_sys::JNI_FALSE
            }
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionCheck: Some(exception_check),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;

        unsafe extern "system" fn detach(_: *mut jni_sys::JavaVM) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            DetachCurrentThread: Some(detach),
            ..empty_raw_java_vm()
        };
        let vm = JavaVM {
            java_vm: &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM,
            owned: false,
        };
        let env = JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: raw_jni_env,
            has_token: RefCell::new(true),
        };
        env.token();
    }
}
