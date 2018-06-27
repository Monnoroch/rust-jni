use attach_arguments::{self, AttachArguments};
use init_arguments::{self, InitArguments};
use java_string::*;
use jni_sys;
use raw::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::string;
use version::{self, JniVersion};

/// Errors returned by JNI function.
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
/// [`rust-jni`](index.html) tries to push as many programming errors as possible from run-time to compile-time.
/// To not allow a caller to call JNI methods when there is a pending exception, these methods
/// will require the caller to provide a [`NoException`](struct.NoException.html) token.
/// The caller can obtain the token after attaching the thread to the Java VM:
/// ```
/// use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion};
///
/// let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// ```
/// A token can not be obtained twice from a `JniEnv` value:
/// ```should_panic
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion};
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
/// Methods that consume the token will always return a [`JniResult`](type.JniResult.html)
/// value which will either have a value and a new [`NoException`](struct.NoException.html) token
/// that can be used to call more JNI methods or an [`Exception`](struct.Exception.html) token:
/// ```
/// // TODO: example of a token-consuming method returning a new token.
/// ```
/// The token is bound to the [`JniEnv`](struct.JniEnv.html) object, so it can't outlive it:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion};
///
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let token = {
///     let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
///     let token = env.token();
///     token
/// }; // doesn't compile!
/// ```
/// Tokens that are returned from other methods as part of a [`JniResult`](type.JniResult.html)
/// are also bound to the [`JniEnv`](struct.JniEnv.html) object and can't outlive it:
/// ```
/// // TODO: a compile_fail example with a token, returned from a JNI method.
/// ```
/// If an [`Exception`](struct.Exception.html) token was returned, it means that
/// there is a pending exception. The [`Exception`](struct.Exception.html) token can be
/// [`unwrap`](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap)-ped into a new
/// [`NoException`](struct.NoException.html) token and a [`Throwable`](struct.Throwable.html)
/// value with the pending exception.
/// Unwrapping the [`Exception`](struct.Exception.html) token will clear the pending exception,
/// so it is again safe to call JNI methods:
/// ```
/// // TODO: an example for `Exception::unwrap`.
/// ```
///
/// # Error handling in Java method calls
///
/// Calling methods on Java objects is slightly different. [`rust-jni`](index.html) follows Java
/// semantics, whre a method either returns a result or throws an exception. All Java methods
/// return a [`JavaResult`](type.JavaResult.html) value, which is either an actual result or a
/// [`Throwable`](struct.Throwable.html) value representing the exception. Java methods never
/// leave a pending exception, so they never consume the
/// [`NoException`](struct.NoException.html) token, but they always require it to be presented:
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

    /// Unsafe, because having two tokens will allow calling methods when there is a
    /// pending exception.
    unsafe fn clone(&self) -> Self {
        Self::new_raw()
    }
}

// [`NoException`](struct.NoException.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for NoException<'env> {}
// impl<'env> !Sync for NoException<'env> {}

/// A dual token to [`NoException`](struct.NoException.html) that represents that there
/// might be a pending exception in Java.
///
/// Read more about exception tokens in [`NoException`](struct.NoException.html) documentation.
#[derive(Debug)]
pub struct Exception<'env> {
    _token: (),
    _env: PhantomData<JniEnv<'env>>,
}

impl<'env> Exception<'env> {
    /// Exchange a [`NoException`](struct.NoException.html) for an
    /// [`Exception`](struct.Exception.html) token. This means that [`rust-jni`](index.html)
    /// no onger can prove that there is no pending exception.
    /// Unsafe because there might not actually be a pending exception when this method is called.
    unsafe fn new<'a>(_env: &'a JniEnv<'a>, _token: NoException) -> Exception<'a> {
        Self::new_raw()
    }

    /// Unsafe because:
    /// 1. Unsafe because there might not actually be a pending exception when this method is called.
    /// 2. Doesn't ensure a correct lifetime
    unsafe fn new_raw<'a>() -> Exception<'a> {
        Exception {
            _token: (),
            _env: PhantomData::<JniEnv>,
        }
    }
}

// [`Exception`](struct.Exception.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for NoException<'env> {}
// impl<'env> !Sync for NoException<'env> {}

/// A result of a JNI function call. Either a value and a [`NoException`](struct.NoException.html)
/// token, when the function didn't throw an exception or an [`Exception`](struct.Exception.html)
/// token when it did or it is unknown if it did.
/// All JNI methods that are not calls to methods of Java classes use this type as their result.
pub type JniResult<'env, T> = Result<(T, NoException<'env>), Exception<'env>>;

/// Create a [`JniResult`](type.JniResult.html) from a nullable pointer.
///
/// Will return an [`Exception`](struct.Exception.html) token for the `null` value or the argument
/// and a [`NoException`](struct.NoException.html) token otherwise.
/// Unsafe because there might not be a pending exception.
unsafe fn from_nullable<'a, T>(
    env: &'a JniEnv<'a>,
    value: *mut T,
    token: NoException<'a>,
) -> JniResult<'a, *mut T> {
    if value == ptr::null_mut() {
        Err(Exception::new(env, token))
    } else {
        Ok((value, token))
    }
}

/// A type that represents a result of a Java method call. A Java method can either return
/// a result or throw a
/// [`Throwable`](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html).
pub type JavaResult<'env, T> = Result<T, Throwable<'env>>;

#[cfg(test)]
mod jni_result_tests {
    use super::*;

    #[test]
    fn from_nullable_null() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert!(
                from_nullable(&env, ptr::null_mut() as *mut i32, NoException::new_raw()).is_err()
            );
        }
    }

    #[test]
    fn from_nullable_non_null() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let ptr = 0x1234 as *mut i32;
        unsafe {
            let value = from_nullable(&env, ptr, NoException::new_raw());
            assert!(value.is_ok());
            assert_eq!(value.unwrap().0, ptr);
        }
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
                        // Safe because a correct pointer is passed.
                        .map(|java_vm| unsafe { Self::from_ptr(java_vm) })
                        .collect()),
                    status => Err(JniError::Unknown(status)),
                }
            }
            status => Err(JniError::Unknown(status)),
        }
    }

    /// Unsafe because one can pass an invalid `java_vm` pointer.
    unsafe fn from_ptr(java_vm: *mut jni_sys::JavaVM) -> JavaVM {
        JavaVM {
            java_vm,
            owned: false,
        }
    }

    /// Get the raw Java VM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    pub unsafe fn raw_jvm(&self) -> *mut jni_sys::JavaVM {
        self.java_vm
    }

    /// Attach the current thread to the Java VM with a specific thread name.
    /// Returns a [`JniEnv`](struct.JniEnv.html) instance for this thread.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn attach(&self, arguments: &AttachArguments) -> Result<JniEnv, JniError> {
        // Safe because the argument is ensured to be the correct method.
        unsafe { self.attach_generic(arguments, (**self.raw_jvm()).AttachCurrentThread.unwrap()) }
    }

    /// Attach the current thread to the Java VM as a daemon with a specific thread name.
    /// Returns a [`JniEnv`](struct.JniEnv.html) instance for this thread.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthreadasdaemon)
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
                        let mut env = JniEnv {
                            version: arguments.version(),
                            vm: self,
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

/// Make [`JavaVM`](struct.JavaVM.html) be destroyed when the value is dropped.
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

/// Make [`JavaVM`](struct.JavaVM.html) sendable between threads. Guaranteed to be safe by JNI.
unsafe impl Send for JavaVM {}

/// Make [`JavaVM`](struct.JavaVM.html) shareable by multiple threads. Guaranteed to be safe
/// by JNI.
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
        assert_eq!(vm.owned, true);
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
            let _vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        assert_eq!(env.native_method_call, false);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        let vm = test_vm(raw_java_vm_ptr);
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
        assert_eq!(env.native_method_call, false);
        // Don't want to drop a manually created `JniEnv`.
        mem::forget(env);
    }
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
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
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
///     vm.attach(&AttachArguments::new(&init_arguments)).unwrap() // doesn't compile!
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
    native_method_call: bool,
}

// [`JniEnv`](struct.JniEnv.html) can't be passed between threads.
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
            unsafe { NoException::new_env(self) }
        }
    }

    /// Get JNI version.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getversion)
    pub fn version(&self) -> JniVersion {
        self.version
    }

    fn has_exception(&self) -> bool {
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
pub fn test_vm(ptr: *mut jni_sys::JavaVM) -> JavaVM {
    JavaVM {
        java_vm: ptr,
        owned: false,
    }
}

#[cfg(test)]
pub fn test_env<'vm>(vm: &'vm JavaVM, ptr: *mut jni_sys::JNIEnv) -> JniEnv<'vm> {
    JniEnv {
        version: JniVersion::V8,
        vm: &vm,
        jni_env: ptr,
        has_token: RefCell::new(true),
        native_method_call: true,
    }
}

#[cfg(test)]
mod jni_env_tests {
    use super::*;
    use testing::*;

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
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        assert_eq!(env.version(), JniVersion::V8);
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
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
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
                native_method_call: false,
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
    fn drop_native_method() {
        let vm = test_vm(ptr::null_mut());
        test_env(&vm, ptr::null_mut());
        // This test would fail if any JNI methods were called by the `JniEnv::drop` method.
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
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
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
            native_method_call: false,
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
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
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
            native_method_call: false,
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
        let vm = test_vm(raw_java_vm_ptr);
        let env = test_env(&vm, raw_jni_env);
        env.token();
        unsafe {
            assert_eq!(EXCEPTION_CHECK_CALLS, 1);
            assert_eq!(EXCEPTION_CHECK_ARGUMENT, raw_jni_env);
        }
        assert_eq!(env.has_token, RefCell::new(false));
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
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        let env = JniEnv {
            version: JniVersion::V8,
            vm: &vm,
            jni_env: raw_jni_env,
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
        let vm = test_vm(&mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM);
        let env = test_env(&vm, raw_jni_env);
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
pub trait JniType {}

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
    fn __signature() -> string::String;
}

/// A macro for generating Java class boilerplate for Rust types, whcih is suitable for
/// `Object` type.
macro_rules! object_java_class {
    ($class:ident, $doc:meta) => {
        #[$doc]
        impl<'a> JavaType for $class<'a> {
            #[doc(hidden)]
            type __JniType = jni_sys::jobject;

            #[doc(hidden)]
            fn __signature() -> &'static str {
                concat!("L", "java/lang", "/", stringify!($class), ";")
            }
        }

        /// Make this class convertible to
        /// [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'a> ToJni for $class<'a> {
            unsafe fn __to_jni(&self) -> Self::__JniType {
                self.raw_object()
            }
        }
    };
}

/// A macro for generating Java class boilerplate for Rust types, except for the `Object` type.
macro_rules! java_class {
    ($class:ident, $doc:meta) => {
        object_java_class!($class, $doc);

        /// Make this class convertible from
        /// [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'env> FromJni<'env> for $class<'env> {
            unsafe fn __from_jni(env: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
                Self {
                    object: Object::__from_jni(env, value),
                }
            }
        }

        /// Allow Java object to be used in place of its superclass.
        impl<'env> ::std::ops::Deref for $class<'env> {
            type Target = Object<'env>;

            fn deref(&self) -> &Self::Target {
                &self.object
            }
        }
    };
}

/// A type representing the
/// [`java.lang.Object`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html) class
/// -- the root class of Java's class hierarchy.
///
/// [`Object` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html)
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct Object<'env> {
    env: &'env JniEnv<'env>,
    raw_object: jni_sys::jobject,
}

// [`Object`](struct.Object.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for Object<'env> {}
// impl<'env> !Sync for Object<'env> {}

impl<'env> Object<'env> {
    /// Get the raw object pointer.
    ///
    /// This function provides low-level access to the Java object and thus is unsafe.
    pub unsafe fn raw_object(&self) -> jni_sys::jobject {
        self.raw_object
    }

    /// Get the [`JniEnv`](../../struct.JniEnv.html) this object is bound to.
    pub fn env(&self) -> &'env JniEnv<'env> {
        self.env
    }

    /// Get the object's class.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getobjectclass)
    pub fn class(&self, _token: &NoException) -> Class<'env> {
        // Safe because arguments are ensured to be correct references by construction.
        let raw_java_class = unsafe { call_jni_method!(self.env, GetObjectClass, self.raw_object) };
        if raw_java_class == ptr::null_mut() {
            panic!("Object {:?} doesn't have a class.", self.raw_object);
        }
        // Safe because the argument is ensured to be correct references by construction.
        unsafe { Class::__from_jni(self.env, raw_java_class) }
    }

    /// Construct from a raw pointer. Unsafe because an invalid pointer may be passed
    /// as the argument.
    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_raw(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Self {
        Self { env, raw_object }
    }
}

object_java_class!(
    Object,
    doc = "Make [`Object`](struct.Object.html) mappable to \
           [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html)."
);

/// Make [`Object`](struct.Object.html) convertible from
/// [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
impl<'env> FromJni<'env> for Object<'env> {
    unsafe fn __from_jni(env: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
        Self::from_raw(env, value)
    }
}

/// Make [`Object`](struct.Object.html)-s reference be deleted when the value is dropped.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#deletelocalref)
impl<'env> Drop for Object<'env> {
    fn drop(&mut self) {
        // Safe because the argument is ensured to be correct references by construction.
        unsafe {
            call_jni_method!(self.env, DeleteLocalRef, self.raw_object);
        }
    }
}

#[cfg(test)]
fn test_object<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Object<'env> {
    Object { env, raw_object }
}

#[cfg(test)]
mod object_tests {
    use super::*;
    use std::mem;
    use testing::*;

    #[test]
    fn raw_object() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            assert_eq!(object.raw_object(), raw_object);
        }
        mem::forget(object);
    }

    #[test]
    fn env() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            assert_eq!(object.env().raw_env(), jni_env);
        }
        mem::forget(object);
    }

    #[test]
    fn signature() {
        assert_eq!(Object::__signature(), "Ljava/lang/Object;");
    }

    #[test]
    fn to_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            assert_eq!(object.__to_jni(), raw_object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = Object::__from_jni(&env, raw_object);
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            let object = Object::__from_jni(&env, object.__to_jni());
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = Object::__from_jni(&env, raw_object);
            assert_eq!(object.__to_jni(), raw_object);
            mem::forget(object);
        }
    }

    #[test]
    fn drop() {
        static mut DELETE_LOCAL_REF_CALLS: i32 = 0;
        static mut DELETE_LOCAL_REF_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut DELETE_LOCAL_REF_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn delete_local_ref(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
        ) {
            DELETE_LOCAL_REF_CALLS += 1;
            DELETE_LOCAL_REF_ENV_ARGUMENT = env;
            DELETE_LOCAL_REF_OBJECT_ARGUMENT = object;
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            DeleteLocalRef: Some(delete_local_ref),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        {
            let _object = test_object(&env, raw_object);
            unsafe {
                assert_eq!(DELETE_LOCAL_REF_CALLS, 0);
            }
        }
        unsafe {
            assert_eq!(DELETE_LOCAL_REF_CALLS, 1);
            assert_eq!(DELETE_LOCAL_REF_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(DELETE_LOCAL_REF_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    fn class() {
        static mut GET_OBJECT_CLASS_CALLS: i32 = 0;
        static mut GET_OBJECT_CLASS_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_OBJECT_CLASS_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_OBJECT_CLASS_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn get_object_class(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
        ) -> jni_sys::jobject {
            GET_OBJECT_CLASS_CALLS += 1;
            GET_OBJECT_CLASS_ENV_ARGUMENT = env;
            GET_OBJECT_CLASS_OBJECT_ARGUMENT = object;
            GET_OBJECT_CLASS_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            GetObjectClass: Some(get_object_class),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let raw_class = 0x1234 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            GET_OBJECT_CLASS_RESULT = raw_class;
        }
        let class = object.class(&unsafe { NoException::new_raw() });
        unsafe {
            assert_eq!(class.raw_object(), raw_class);
            assert_eq!(class.env().raw_env(), raw_jni_env);
            assert_eq!(GET_OBJECT_CLASS_CALLS, 1);
            assert_eq!(GET_OBJECT_CLASS_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_OBJECT_CLASS_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    #[should_panic(expected = "doesn't have a class")]
    fn class_not_found() {
        unsafe extern "system" fn get_object_class(
            _: *mut jni_sys::JNIEnv,
            _: jni_sys::jobject,
        ) -> jni_sys::jobject {
            ptr::null_mut() as jni_sys::jobject
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            GetObjectClass: Some(get_object_class),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let object = test_object(&env, ptr::null_mut());
        object.class(&unsafe { NoException::new_raw() });
    }
}

/// A type representing a Java
/// [`Throwable`](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html).
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct Throwable<'env> {
    object: Object<'env>,
}

impl<'env> Throwable<'env> {
    /// Throw the exception. Transfers ownership of the object to Java.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#throw)
    pub fn throw(self, token: NoException) -> Exception<'env> {
        // Safe because the argument is ensured to be correct references by construction.
        let status = unsafe {
            call_jni_method!(self.env(), Throw, self.raw_object() as jni_sys::jthrowable)
        };
        // Can't really handle failing throwing an exception.
        if status != jni_sys::JNI_OK {
            panic!("Throwing an exception has failed with status {}.", status);
        }
        // Safe becuase we just threw the exception.
        unsafe { Exception::new(self.env(), token) }
    }
}

java_class!(
    Throwable,
    doc = "Make [`Throwable`](struct.Throwable.html) mappable to \
           [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html)."
);

#[cfg(test)]
mod throwable_tests {
    use super::*;
    use std::mem;
    use std::ops::Deref;
    use testing::*;

    fn test_throwable<'env>(
        env: &'env JniEnv<'env>,
        raw_object: jni_sys::jobject,
    ) -> Throwable<'env> {
        Throwable {
            object: test_object(env, raw_object),
        }
    }

    #[test]
    fn deref_super() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let object = test_throwable(&env, ptr::null_mut());
        // Will not compile if is not deref-able.
        &object as &Deref<Target = Object>;
        mem::forget(object);
    }

    #[test]
    fn signature() {
        assert_eq!(Throwable::__signature(), "Ljava/lang/Throwable;");
    }

    #[test]
    fn to_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_throwable(&env, raw_object);
        unsafe {
            assert_eq!(object.__to_jni(), raw_object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = Throwable::__from_jni(&env, raw_object);
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_throwable(&env, raw_object);
        unsafe {
            let object = Throwable::__from_jni(&env, object.__to_jni());
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = Throwable::__from_jni(&env, raw_object);
            assert_eq!(object.__to_jni(), raw_object);
            mem::forget(object);
        }
    }

    #[test]
    fn drop() {
        static mut DELETE_LOCAL_REF_CALLS: i32 = 0;
        static mut DELETE_LOCAL_REF_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut DELETE_LOCAL_REF_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn delete_local_ref(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
        ) {
            DELETE_LOCAL_REF_CALLS += 1;
            DELETE_LOCAL_REF_ENV_ARGUMENT = env;
            DELETE_LOCAL_REF_OBJECT_ARGUMENT = object;
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            DeleteLocalRef: Some(delete_local_ref),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        {
            let _object = test_throwable(&env, raw_object);
            unsafe {
                assert_eq!(DELETE_LOCAL_REF_CALLS, 0);
            }
        }
        unsafe {
            assert_eq!(DELETE_LOCAL_REF_CALLS, 1);
            assert_eq!(DELETE_LOCAL_REF_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(DELETE_LOCAL_REF_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    fn throw() {
        static mut THROW_CALLS: i32 = 0;
        static mut THROW_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut THROW_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn throw(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
        ) -> jni_sys::jint {
            THROW_CALLS += 1;
            THROW_ENV_ARGUMENT = env;
            THROW_OBJECT_ARGUMENT = object;
            jni_sys::JNI_OK
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            Throw: Some(throw),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_throwable(&env, raw_object);
        object.throw(unsafe { NoException::new_raw() });
        unsafe {
            assert_eq!(THROW_CALLS, 1);
            assert_eq!(THROW_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(THROW_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    #[should_panic(expected = "Throwing an exception has failed with status -1.")]
    fn throw_failed() {
        unsafe extern "system" fn throw(
            _: *mut jni_sys::JNIEnv,
            _: jni_sys::jobject,
        ) -> jni_sys::jint {
            jni_sys::JNI_ERR
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            Throw: Some(throw),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let object = test_throwable(&env, ptr::null_mut());
        object.throw(unsafe { NoException::new_raw() });
    }
}

/// A type representing a Java
/// [`Class`](https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html).
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct Class<'env> {
    object: Object<'env>,
}

impl<'env> Class<'env> {
    /// Find an existing Java class by it's name. The name is a fully qualified class or array
    /// type name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#findclass)
    pub fn find<'a>(
        env: &'a JniEnv<'a>,
        class_name: &str,
        token: &NoException<'a>,
    ) -> JavaResult<'a, Class<'a>> {
        with_checked_exception(env, token, |token| {
            let class_name = to_java_string(class_name);
            // Safe because arguments are correct.
            let raw_java_class =
                unsafe { call_jni_method!(env, FindClass, class_name.as_ptr() as *const c_char) };
            // Safe because `FindClass` throws an exception before returning `null`.
            unsafe { from_nullable(env, raw_java_class, token) }.map(|(raw_java_class, token)| {
                (
                    // Safe because the argument is a valid class reference.
                    unsafe { Self::from_raw(env, raw_java_class) },
                    token,
                )
            })
        })
    }

    /// Unsafe because the argument mught not be a valid class reference.
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, raw_class: jni_sys::jclass) -> Class<'a> {
        Class {
            object: Object::__from_jni(env, raw_class as jni_sys::jobject),
        }
    }
}

java_class!(
    Class,
    doc = "Make [`Class`](struct.Class.html) mappable to \
           [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html)."
);

#[cfg(test)]
fn test_returned_null<T>(jni_env: jni_sys::JNINativeInterface_, function: T)
where
    T: for<'a> FnOnce(&'a JniEnv<'a>, NoException<'a>) -> Throwable<'a>,
{
    static mut EXCEPTION_OCCURED_CALLS: i32 = 0;
    static mut EXCEPTION_OCCURED_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
    static mut EXCEPTION_OCCURED_RESULT: jni_sys::jobject = ptr::null_mut();
    unsafe {
        EXCEPTION_OCCURED_CALLS = 0;
    }
    unsafe extern "system" fn exception_occured(env: *mut jni_sys::JNIEnv) -> jni_sys::jobject {
        EXCEPTION_OCCURED_CALLS += 1;
        EXCEPTION_OCCURED_ENV_ARGUMENT = env;
        EXCEPTION_OCCURED_RESULT
    }
    static mut EXCEPTION_CLEAR_CALLS: i32 = 0;
    static mut EXCEPTION_CLEAR_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
    unsafe {
        EXCEPTION_CLEAR_CALLS = 0;
    }
    unsafe extern "system" fn exception_clear(env: *mut jni_sys::JNIEnv) {
        EXCEPTION_CLEAR_CALLS += 1;
        EXCEPTION_CLEAR_ENV_ARGUMENT = env;
    }
    let vm = test_vm(ptr::null_mut());
    let raw_jni_env = jni_sys::JNINativeInterface_ {
        ExceptionOccurred: Some(exception_occured),
        ExceptionClear: Some(exception_clear),
        ..jni_env
    };
    let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
    let env = test_env(&vm, raw_jni_env);
    let raw_exception = 0x1234 as jni_sys::jobject;
    unsafe {
        EXCEPTION_OCCURED_RESULT = raw_exception;
    }
    let exception = function(&env, unsafe { NoException::new_raw() });
    unsafe {
        assert_eq!(exception.raw_object(), raw_exception);
        assert_eq!(exception.env().raw_env(), raw_jni_env);
        assert_eq!(EXCEPTION_OCCURED_CALLS, 1);
        assert_eq!(EXCEPTION_OCCURED_ENV_ARGUMENT, raw_jni_env);
        assert_eq!(EXCEPTION_CLEAR_CALLS, 1);
        assert_eq!(EXCEPTION_CLEAR_ENV_ARGUMENT, raw_jni_env);
    }
}

#[cfg(test)]
mod class_tests {
    use super::*;
    use std::ffi::CStr;
    use std::mem;
    use std::ops::Deref;
    use testing::*;

    fn test_class<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Class<'env> {
        Class {
            object: test_object(env, raw_object),
        }
    }

    #[test]
    fn deref_super() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let object = test_class(&env, ptr::null_mut());
        // Will not compile if is not deref-able.
        &object as &Deref<Target = Object>;
        mem::forget(object);
    }

    #[test]
    fn signature() {
        assert_eq!(Class::__signature(), "Ljava/lang/Class;");
    }

    #[test]
    fn to_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_class(&env, raw_object);
        unsafe {
            assert_eq!(object.__to_jni(), raw_object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = Class::__from_jni(&env, raw_object);
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_class(&env, raw_object);
        unsafe {
            let object = Class::__from_jni(&env, object.__to_jni());
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = Class::__from_jni(&env, raw_object);
            assert_eq!(object.__to_jni(), raw_object);
            mem::forget(object);
        }
    }

    #[test]
    fn drop() {
        static mut DELETE_LOCAL_REF_CALLS: i32 = 0;
        static mut DELETE_LOCAL_REF_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut DELETE_LOCAL_REF_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn delete_local_ref(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
        ) {
            DELETE_LOCAL_REF_CALLS += 1;
            DELETE_LOCAL_REF_ENV_ARGUMENT = env;
            DELETE_LOCAL_REF_OBJECT_ARGUMENT = object;
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            DeleteLocalRef: Some(delete_local_ref),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        {
            let _object = test_class(&env, raw_object);
            unsafe {
                assert_eq!(DELETE_LOCAL_REF_CALLS, 0);
            }
        }
        unsafe {
            assert_eq!(DELETE_LOCAL_REF_CALLS, 1);
            assert_eq!(DELETE_LOCAL_REF_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(DELETE_LOCAL_REF_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    fn find() {
        static mut FIND_CLASS_CALLS: i32 = 0;
        static mut FIND_CLASS_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut FIND_CLASS_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn find_class(
            env: *mut jni_sys::JNIEnv,
            name: *const c_char,
        ) -> jni_sys::jobject {
            assert_eq!(
                from_java_string(CStr::from_ptr(name).to_bytes_with_nul()).unwrap(),
                "test-class"
            );
            FIND_CLASS_CALLS += 1;
            FIND_CLASS_ENV_ARGUMENT = env;
            FIND_CLASS_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            FindClass: Some(find_class),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            FIND_CLASS_RESULT = raw_object;
        }

        let class = Class::find(&env, "test-class", &unsafe { NoException::new_raw() }).unwrap();
        unsafe {
            assert_eq!(class.raw_object(), raw_object);
            assert_eq!(class.env().raw_env(), raw_jni_env);
            assert_eq!(FIND_CLASS_CALLS, 1);
            assert_eq!(FIND_CLASS_ENV_ARGUMENT, raw_jni_env);
        }
    }

    #[test]
    fn find_not_found() {
        unsafe extern "system" fn find_class(
            _: *mut jni_sys::JNIEnv,
            _: *const c_char,
        ) -> jni_sys::jobject {
            ptr::null_mut() as jni_sys::jobject
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            FindClass: Some(find_class),
            ..empty_raw_jni_env()
        };
        test_returned_null(raw_jni_env, |env, token| {
            Class::find(env, "test-class", &token).unwrap_err()
        });
    }
}

/// A type representing a Java
/// [`String`](https://docs.oracle.com/javase/10/docs/api/java/lang/String.html).
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct String<'env> {
    object: Object<'env>,
}

impl<'env> String<'env> {
    /// Create a new empty string.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newstring)
    pub fn empty<'a>(env: &'a JniEnv<'a>, token: &NoException<'a>) -> JavaResult<'a, String<'a>> {
        // Safe because arguments are ensured to be the correct by construction.
        let raw_string =
            unsafe { call_jni_method!(env, NewString, ptr::null(), 0 as jni_sys::jsize) };
        // Safe because `raw_string` is a valid reference to a `String` object.
        unsafe { Self::from_ptr(env, raw_string, token) }
    }

    /// Create a new Java string from a Rust string.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newstringutf)
    pub fn new<'a>(
        env: &'a JniEnv<'a>,
        string: &str,
        token: &NoException<'a>,
    ) -> JavaResult<'a, String<'a>> {
        if string.is_empty() {
            return Self::empty(env, token);
        }

        let buffer = to_java_string(string);
        let raw_string =
            unsafe { call_jni_method!(env, NewStringUTF, buffer.as_ptr() as *const c_char) };
        // Safe because `raw_string` is a valid reference to a `String` object.
        unsafe { Self::from_ptr(env, raw_string, token) }
    }

    /// String length (the number of unicode characters).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringlength)
    pub fn len(&self, _token: &NoException) -> usize {
        // Safe because arguments are ensured to be the correct by construction.
        let length = unsafe {
            call_jni_method!(
                self.env(),
                GetStringLength,
                self.raw_object() as jni_sys::jstring
            )
        };
        length as usize
    }

    /// String size (the number of bytes in modified UTF-8).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringutflength)
    pub fn size(&self, _token: &NoException) -> usize {
        // Safe because arguments are ensured to be the correct by construction.
        let size = unsafe {
            call_jni_method!(
                self.env(),
                GetStringUTFLength,
                self.raw_object() as jni_sys::jstring
            )
        };
        size as usize + 1 // +1 for the '\0' byte.
    }

    /// Convert the Java `String` into a Rust `String`.
    ///
    /// This method has a different signature from the one in the `ToString` trait because
    /// extracting bytes from `String` is only safe when there is no pending exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringutfregion)
    pub fn as_string(&self, token: &NoException) -> string::String {
        let length = self.len(token);
        if length == 0 {
            return "".to_owned();
        }

        let size = self.size(token);
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        // Safe because arguments are ensured to be the correct by construction.
        unsafe {
            call_jni_method!(
                self.env(),
                GetStringUTFRegion,
                self.raw_object() as jni_sys::jstring,
                0 as jni_sys::jsize,
                length as jni_sys::jsize,
                buffer.as_mut_ptr() as *mut c_char
            );
            buffer.set_len(size);
        }
        from_java_string(buffer.as_slice()).unwrap().into_owned()
    }

    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_ptr<'a>(
        env: &'a JniEnv<'a>,
        raw_string: jni_sys::jstring,
        token: &NoException<'a>,
    ) -> JavaResult<'a, String<'a>> {
        with_checked_exception(env, token, |token| {
            from_nullable(env, raw_string, token).map(|(raw_string, token)| {
                (
                    // Safe because the argument is a valid class reference.
                    Self::from_raw(env, raw_string),
                    token,
                )
            })
        })
    }

    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, raw_string: jni_sys::jstring) -> String<'a> {
        String {
            object: Object::__from_jni(env, raw_string as jni_sys::jobject),
        }
    }
}

java_class!(
    String,
    doc = "Make [`String`](struct.String.html) mappable to \
           [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html)."
);

#[cfg(test)]
mod string_tests {
    use super::*;
    use std::ffi::CStr;
    use std::mem;
    use std::ops::Deref;
    use testing::*;

    fn test_string<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> String<'env> {
        String {
            object: test_object(env, raw_object),
        }
    }

    #[test]
    fn deref_super() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let object = test_string(&env, ptr::null_mut());
        // Will not compile if is not deref-able.
        &object as &Deref<Target = Object>;
        mem::forget(object);
    }

    #[test]
    fn signature() {
        assert_eq!(String::__signature(), "Ljava/lang/String;");
    }

    #[test]
    fn to_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_string(&env, raw_object);
        unsafe {
            assert_eq!(object.__to_jni(), raw_object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = String::__from_jni(&env, raw_object);
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_string(&env, raw_object);
        unsafe {
            let object = String::__from_jni(&env, object.__to_jni());
            assert_eq!(object.raw_object(), raw_object);
            assert_eq!(object.env().raw_env(), jni_env);
            mem::forget(object);
        }
        mem::forget(object);
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            let object = String::__from_jni(&env, raw_object);
            assert_eq!(object.__to_jni(), raw_object);
            mem::forget(object);
        }
    }

    #[test]
    fn drop() {
        static mut DELETE_LOCAL_REF_CALLS: i32 = 0;
        static mut DELETE_LOCAL_REF_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut DELETE_LOCAL_REF_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn delete_local_ref(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
        ) {
            DELETE_LOCAL_REF_CALLS += 1;
            DELETE_LOCAL_REF_ENV_ARGUMENT = env;
            DELETE_LOCAL_REF_OBJECT_ARGUMENT = object;
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            DeleteLocalRef: Some(delete_local_ref),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        {
            let _object = test_string(&env, raw_object);
            unsafe {
                assert_eq!(DELETE_LOCAL_REF_CALLS, 0);
            }
        }
        unsafe {
            assert_eq!(DELETE_LOCAL_REF_CALLS, 1);
            assert_eq!(DELETE_LOCAL_REF_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(DELETE_LOCAL_REF_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    fn empty() {
        static mut NEW_STRING_CALLS: i32 = 0;
        static mut NEW_STRING_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut NEW_STRING_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn new_string(
            env: *mut jni_sys::JNIEnv,
            name: *const jni_sys::jchar,
            size: jni_sys::jsize,
        ) -> jni_sys::jobject {
            assert_eq!(size, 0);
            assert_eq!(name, ptr::null());
            NEW_STRING_CALLS += 1;
            NEW_STRING_ENV_ARGUMENT = env;
            NEW_STRING_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewString: Some(new_string),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            NEW_STRING_RESULT = raw_object;
        }

        let string = String::empty(&env, &unsafe { NoException::new_raw() }).unwrap();
        unsafe {
            assert_eq!(string.raw_object(), raw_object);
            assert_eq!(string.env().raw_env(), raw_jni_env);
            assert_eq!(NEW_STRING_CALLS, 1);
            assert_eq!(NEW_STRING_ENV_ARGUMENT, raw_jni_env);
        }
    }

    #[test]
    fn empty_exception() {
        unsafe extern "system" fn new_string(
            _: *mut jni_sys::JNIEnv,
            _: *const jni_sys::jchar,
            _: jni_sys::jsize,
        ) -> jni_sys::jobject {
            ptr::null_mut()
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewString: Some(new_string),
            ..empty_raw_jni_env()
        };
        test_returned_null(raw_jni_env, |env, token| {
            String::empty(&env, &token).unwrap_err()
        });
    }

    #[test]
    fn new_empty() {
        static mut NEW_STRING_CALLS: i32 = 0;
        static mut NEW_STRING_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut NEW_STRING_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn new_string(
            env: *mut jni_sys::JNIEnv,
            name: *const jni_sys::jchar,
            size: jni_sys::jsize,
        ) -> jni_sys::jobject {
            assert_eq!(size, 0);
            assert_eq!(name, ptr::null());
            NEW_STRING_CALLS += 1;
            NEW_STRING_ENV_ARGUMENT = env;
            NEW_STRING_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewString: Some(new_string),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            NEW_STRING_RESULT = raw_object;
        }

        let string = String::new(&env, "", &unsafe { NoException::new_raw() }).unwrap();
        unsafe {
            assert_eq!(string.raw_object(), raw_object);
            assert_eq!(string.env().raw_env(), raw_jni_env);
            assert_eq!(NEW_STRING_CALLS, 1);
            assert_eq!(NEW_STRING_ENV_ARGUMENT, raw_jni_env);
        }
    }

    #[test]
    fn new_empty_exception() {
        unsafe extern "system" fn new_string(
            _: *mut jni_sys::JNIEnv,
            _: *const jni_sys::jchar,
            _: jni_sys::jsize,
        ) -> jni_sys::jobject {
            ptr::null_mut()
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewString: Some(new_string),
            ..empty_raw_jni_env()
        };
        test_returned_null(raw_jni_env, |env, token| {
            String::new(&env, "", &token).unwrap_err()
        });
    }

    #[test]
    fn new() {
        static mut NEW_STRING_CALLS: i32 = 0;
        static mut NEW_STRING_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut NEW_STRING_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn new_string_utf(
            env: *mut jni_sys::JNIEnv,
            string: *const c_char,
        ) -> jni_sys::jobject {
            assert_eq!(
                from_java_string(CStr::from_ptr(string).to_bytes_with_nul()).unwrap(),
                "test-string"
            );
            NEW_STRING_CALLS += 1;
            NEW_STRING_ENV_ARGUMENT = env;
            NEW_STRING_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewStringUTF: Some(new_string_utf),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            NEW_STRING_RESULT = raw_object;
        }

        let string = String::new(&env, "test-string", &unsafe { NoException::new_raw() }).unwrap();
        unsafe {
            assert_eq!(string.raw_object(), raw_object);
            assert_eq!(string.env().raw_env(), raw_jni_env);
            assert_eq!(NEW_STRING_CALLS, 1);
            assert_eq!(NEW_STRING_ENV_ARGUMENT, raw_jni_env);
        }
    }

    #[test]
    fn new_exception() {
        unsafe extern "system" fn new_string_utf(
            _: *mut jni_sys::JNIEnv,
            _: *const c_char,
        ) -> jni_sys::jobject {
            ptr::null_mut()
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewStringUTF: Some(new_string_utf),
            ..empty_raw_jni_env()
        };
        test_returned_null(raw_jni_env, |env, token| {
            String::new(&env, "test-string", &token).unwrap_err()
        });
    }

    #[test]
    fn len() {
        static mut GET_STRING_LENGTH_CALLS: i32 = 0;
        static mut GET_STRING_LENGTH_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_STRING_LENGTH_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_STRING_LENGTH_RESULT: jni_sys::jsize = 0;
        unsafe extern "system" fn get_string_length(
            env: *mut jni_sys::JNIEnv,
            string: jni_sys::jobject,
        ) -> jni_sys::jsize {
            GET_STRING_LENGTH_CALLS += 1;
            GET_STRING_LENGTH_ENV_ARGUMENT = env;
            GET_STRING_LENGTH_OBJECT_ARGUMENT = string;
            GET_STRING_LENGTH_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            GetStringLength: Some(get_string_length),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            GET_STRING_LENGTH_RESULT = 17;
        }

        let string = unsafe { String::from_raw(&env, raw_object) };
        assert_eq!(string.len(&unsafe { NoException::new_raw() }), 17);
        unsafe {
            assert_eq!(GET_STRING_LENGTH_CALLS, 1);
            assert_eq!(GET_STRING_LENGTH_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_STRING_LENGTH_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    fn size() {
        static mut GET_STRING_SIZE_CALLS: i32 = 0;
        static mut GET_STRING_SIZE_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_STRING_SIZE_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_STRING_SIZE_RESULT: jni_sys::jsize = 0;
        unsafe extern "system" fn get_string_utf_length(
            env: *mut jni_sys::JNIEnv,
            string: jni_sys::jobject,
        ) -> jni_sys::jsize {
            GET_STRING_SIZE_CALLS += 1;
            GET_STRING_SIZE_ENV_ARGUMENT = env;
            GET_STRING_SIZE_OBJECT_ARGUMENT = string;
            GET_STRING_SIZE_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            GetStringUTFLength: Some(get_string_utf_length),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            GET_STRING_SIZE_RESULT = 17;
        }

        let string = unsafe { String::from_raw(&env, raw_object) };
        assert_eq!(string.size(&unsafe { NoException::new_raw() }), 18);
        unsafe {
            assert_eq!(GET_STRING_SIZE_CALLS, 1);
            assert_eq!(GET_STRING_SIZE_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_STRING_SIZE_OBJECT_ARGUMENT, raw_object);
        }
    }

    #[test]
    fn as_string() {
        static mut GET_STRING_LENGTH_CALLS: i32 = 0;
        static mut GET_STRING_LENGTH_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_STRING_LENGTH_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_STRING_LENGTH_RESULT: jni_sys::jsize = 0;
        unsafe extern "system" fn get_string_length(
            env: *mut jni_sys::JNIEnv,
            string: jni_sys::jobject,
        ) -> jni_sys::jsize {
            GET_STRING_LENGTH_CALLS += 1;
            GET_STRING_LENGTH_ENV_ARGUMENT = env;
            GET_STRING_LENGTH_OBJECT_ARGUMENT = string;
            GET_STRING_LENGTH_RESULT
        }
        static mut GET_STRING_SIZE_CALLS: i32 = 0;
        static mut GET_STRING_SIZE_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_STRING_SIZE_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_STRING_SIZE_RESULT: jni_sys::jsize = 0;
        unsafe extern "system" fn get_string_utf_length(
            env: *mut jni_sys::JNIEnv,
            string: jni_sys::jobject,
        ) -> jni_sys::jsize {
            GET_STRING_SIZE_CALLS += 1;
            GET_STRING_SIZE_ENV_ARGUMENT = env;
            GET_STRING_SIZE_OBJECT_ARGUMENT = string;
            GET_STRING_SIZE_RESULT
        }
        static mut GET_STRING_UTF_REGION_CALLS: i32 = 0;
        static mut GET_STRING_UTF_REGION_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_STRING_UTF_REGION_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_STRING_UTF_REGION_LENGTH_ARGUMENT: jni_sys::jsize = 0;
        unsafe extern "system" fn get_string_utf_region(
            env: *mut jni_sys::JNIEnv,
            string: jni_sys::jobject,
            start: jni_sys::jsize,
            len: jni_sys::jsize,
            buffer: *mut c_char,
        ) {
            assert_eq!(start, 0);
            assert_ne!(buffer, ptr::null_mut());
            let test_buffer = to_java_string("test-string");
            for i in 0..test_buffer.len() {
                *buffer.offset(i as isize) = test_buffer[i] as i8;
            }
            GET_STRING_UTF_REGION_CALLS += 1;
            GET_STRING_UTF_REGION_ENV_ARGUMENT = env;
            GET_STRING_UTF_REGION_OBJECT_ARGUMENT = string;
            GET_STRING_UTF_REGION_LENGTH_ARGUMENT = len;
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            GetStringLength: Some(get_string_length),
            GetStringUTFLength: Some(get_string_utf_length),
            GetStringUTFRegion: Some(get_string_utf_region),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        unsafe {
            GET_STRING_LENGTH_RESULT = 17;
            GET_STRING_SIZE_RESULT = "test-string".len() as i32;
        }

        let string = unsafe { String::from_raw(&env, raw_object) };
        assert_eq!(
            string.as_string(&unsafe { NoException::new_raw() }),
            "test-string"
        );
        unsafe {
            assert_eq!(GET_STRING_LENGTH_CALLS, 1);
            assert_eq!(GET_STRING_LENGTH_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_STRING_LENGTH_OBJECT_ARGUMENT, raw_object);
            assert_eq!(GET_STRING_SIZE_CALLS, 1);
            assert_eq!(GET_STRING_SIZE_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_STRING_SIZE_OBJECT_ARGUMENT, raw_object);
            assert_eq!(GET_STRING_UTF_REGION_CALLS, 1);
            assert_eq!(GET_STRING_UTF_REGION_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_STRING_UTF_REGION_OBJECT_ARGUMENT, raw_object);
            assert_eq!(GET_STRING_UTF_REGION_LENGTH_ARGUMENT, 17);
        }
    }
}

/// Take a function that produces a [`JniResult`](type.JniResult.html), call it and produce
/// a [`JavaResult`](type.JavaResult.html) from it.
fn with_checked_exception<'a, Out, T: FnOnce(NoException<'a>) -> JniResult<'a, Out>>(
    env: &'a JniEnv<'a>,
    token: &NoException<'a>,
    function: T,
) -> JavaResult<'a, Out> {
    // Safe, because we check for a pending exception after the call.
    let token = unsafe { token.clone() };
    match function(token) {
        Err(_) => {
            // Safe because the argument is ensured to be correct references by construction.
            let raw_java_throwable = unsafe { call_jni_method!(env, ExceptionOccurred) };
            if raw_java_throwable == ptr::null_mut() {
                panic!("No pending exception in presence of an Exception token. Should not ever happen.");
            }
            // Safe because the argument is ensured to be correct references by construction.
            unsafe {
                call_jni_method!(env, ExceptionClear);
            }
            // Safe because the arguments are correct.
            unsafe { Err(Throwable::__from_jni(env, raw_java_throwable)) }
        }
        Ok((value, _)) => Ok(value),
    }
}

#[cfg(test)]
mod with_checked_exception_tests {
    use super::*;
    use testing::*;

    #[test]
    fn no_exception() {
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let result = with_checked_exception(&env, &unsafe { NoException::new_raw() }, |_| unsafe {
            Ok((17, NoException::new_raw()))
        }).unwrap();
        assert_eq!(result, 17);
    }

    #[test]
    fn exception() {
        test_returned_null(empty_raw_jni_env(), |env, token| {
            with_checked_exception::<i32, _>(&env, &token, |_| unsafe { Err(Exception::new_raw()) })
                .unwrap_err()
        });
    }

    #[test]
    #[should_panic(expected = "No pending exception in presence of an Exception token")]
    fn exception_not_found() {
        unsafe extern "system" fn exception_occured(_: *mut jni_sys::JNIEnv) -> jni_sys::jobject {
            ptr::null_mut() as jni_sys::jobject
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            ExceptionOccurred: Some(exception_occured),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        with_checked_exception::<i32, _>(&env, &unsafe { NoException::new_raw() }, |_| unsafe {
            Err(Exception::new_raw())
        }).unwrap_err();
    }
}
