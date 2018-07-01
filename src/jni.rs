use self::method_calls::*;
use attach_arguments::{self, AttachArguments};
use init_arguments::{self, InitArguments};
use java_string::*;
use jni_sys;
use primitives::ToJniTuple;
use raw::*;
use std::cell::RefCell;
use std::fmt;
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
/// [`rust-jni`](index.html) tries to push as many programming errors as possible from run-time
/// to compile-time. To not allow a caller to call JNI methods when there is a pending exception,
/// these methods will require the caller to provide a [`NoException`](struct.NoException.html)
/// token. The caller can obtain the token after attaching the thread to the Java VM:
/// ```
/// use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion};
///
/// let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// ```
/// Once obtained, the token can be used to call JNI methods:
/// ```
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion, java};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// let string = java::lang::String::empty(&env, &token).unwrap();
/// ```
/// [`rust-jni`](index.html) follows Java semantics, where a method either returns a result
/// or throws an exception. All Java methods return a [`JavaResult`](type.JavaResult.html) value,
/// which is either an actual result or a [`Throwable`](struct.Throwable.html) value representing
/// the exception thrown by this method call. Java methods never leave a pending exception,
/// so they never consume the [`NoException`](struct.NoException.html) token, but they always
/// require it to be presented:
/// ```
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion, java};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// let string = java::lang::Class::find(&env, "java/lang/String", &token).unwrap();
/// let exception = java::lang::Class::find(&env, "invalid", &token).unwrap_err();
/// ```
/// A token can not be obtained twice from a [`JniEnv`](struct.JniEnv.html) value:
/// ```should_panic
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// let token = env.token(); // panics!
/// ```
/// There is no possible way to obtain a token when there is a pending exception.
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
/// Some JNI methods can throw exceptions themselves. In this case the token will be consumed:
/// ```compile_fail
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion, java};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// let exception = java::lang::String::empty(&env, &token).unwrap_err();
/// exception.throw(token);
/// java::lang::String::empty(&env, &token); // doesn't compile! Can't use the token any more.
/// ```
/// Methods that consume the token will always return an [`Exception`](struct.Exception.html)
/// token. The [`Exception`](struct.Exception.html) token can be
/// [`unwrap`](struct.Exception.html#method.unwrap)-ped into a new
/// [`NoException`](struct.NoException.html) token and a [`Throwable`](struct.Throwable.html)
/// value with the pending exception. Unwrapping the [`Exception`](struct.Exception.html) token
///  will clear the pending exception, so it is again safe to call JNI methods:
/// ```
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion, java};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
/// let token = env.token();
/// let exception = java::lang::Class::find(&env, "invalid", &token).unwrap_err();
/// let exception_token = exception.throw(token); // there is a pending exception now.
/// let (exception, new_token) = exception_token.unwrap();
/// java::lang::String::empty(&env, &new_token); // can call Java methods again.
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

    #[cfg(test)]
    fn test<'a>() -> NoException<'a> {
        // Safe because only used for unit-testing.
        unsafe { Self::new_raw() }
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
    env: &'env JniEnv<'env>,
}

impl<'env> Exception<'env> {
    /// Get and clear the pending exception and a [`NoException`](struct.NoException.html) token
    /// to call more JNI methods. The [`Exception`](struct.Exception.html) token is consumed
    /// by this method and can't be used any more.
    pub fn unwrap(self) -> (Throwable<'env>, NoException<'env>) {
        let throwable = get_and_clear_exception(self);
        // Safe because we just cleared the pending exception.
        let token = unsafe { NoException::new_raw() };
        (throwable, token)
    }

    /// Exchange a [`NoException`](struct.NoException.html) for an
    /// [`Exception`](struct.Exception.html) token. This means that [`rust-jni`](index.html)
    /// no onger can prove that there is no pending exception.
    /// Unsafe because there might not actually be a pending exception when this method is called.
    unsafe fn new<'a>(env: &'a JniEnv<'a>, _token: NoException) -> Exception<'a> {
        Self::new_raw(env)
    }

    /// Unsafe because:
    /// 1. Unsafe because there might not actually be a pending exception when this method is
    /// called.
    /// 2. Doesn't ensure a correct lifetime
    unsafe fn new_raw<'a>(env: &'a JniEnv<'a>) -> Exception<'a> {
        Exception { _token: (), env }
    }

    #[cfg(test)]
    fn test<'a>(env: &'a JniEnv<'a>) -> Exception<'a> {
        // Safe because only used for unit-testing.
        unsafe { Self::new_raw(env) }
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;
    use testing::*;

    #[test]
    fn unwrap() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let token = Exception::test(&env);
        let (exception, _) = token.unwrap();
        calls.assert_eq(&exception, EXCEPTION);
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
type JniResult<'env, T> = Result<(T, NoException<'env>), Exception<'env>>;

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
            assert!(from_nullable(&env, ptr::null_mut() as *mut i32, NoException::test()).is_err());
        }
    }

    #[test]
    fn from_nullable_non_null() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let ptr = 0x1234 as *mut i32;
        unsafe {
            let value = from_nullable(&env, ptr, NoException::test());
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

    /// Unsafe because one can pass an invalid `java_vm` pointer.
    unsafe fn from_ptr(java_vm: *mut jni_sys::JavaVM) -> JavaVM {
        JavaVM {
            java_vm,
            owned: false,
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
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
            result: jni_sys::JNI_FALSE,
        })]);
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
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = test_vm(raw_java_vm_ptr);
        let init_arguments = init_arguments::new(JniVersion::V8);
        unsafe {
            ATTACH_ENV_ARGUMENT = calls.env as *mut c_void;
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
            assert_eq!(env.raw_jvm(), raw_java_vm_ptr);
            assert_eq!(env.raw_env(), calls.env);
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
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
            result: jni_sys::JNI_TRUE,
        })]);
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
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThread: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = test_vm(raw_java_vm_ptr);
        unsafe {
            ATTACH_ENV_ARGUMENT = calls.env as *mut c_void;
        }
        vm.attach(&AttachArguments::new(&init_arguments::new(JniVersion::V8)))
            .unwrap();
    }

    #[test]
    fn attach_daemon() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
            result: jni_sys::JNI_FALSE,
        })]);
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
        let raw_java_vm = jni_sys::JNIInvokeInterface_ {
            GetEnv: Some(get_env),
            AttachCurrentThreadAsDaemon: Some(attach),
            ..empty_raw_java_vm()
        };
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let vm = test_vm(raw_java_vm_ptr);
        let init_arguments = init_arguments::new(JniVersion::V8);
        unsafe {
            ATTACH_ENV_ARGUMENT = calls.env as *mut c_void;
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
            assert_eq!(env.raw_jvm(), raw_java_vm_ptr);
            assert_eq!(env.raw_env(), calls.env);
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

macro_rules! call_nullable_jni_method {
    ($env:expr, $method:ident, $token:expr, $($argument:expr),*) => {
        with_checked_exception($token, |token| {
            let result =
                call_jni_method!($env, $method, $($argument),*);
            from_nullable($env, result, token)
        })
    }
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
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
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
                version: JniVersion::V8,
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
            JniCall::ExceptionCheck(ExceptionCheckCall {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::ExceptionDescribe(ExceptionDescribeCall {}),
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
            version: JniVersion::V8,
            vm: &vm,
            jni_env: calls.env,
            has_token: RefCell::new(true),
            native_method_call: false,
        };
    }

    #[test]
    #[should_panic(expected = "Could not detach the current thread. Status: -1")]
    fn drop_detach_error() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
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
            version: JniVersion::V8,
            vm: &vm,
            jni_env: calls.env,
            has_token: RefCell::new(true),
            native_method_call: false,
        };
    }

    #[test]
    fn token() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
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
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
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
            version: JniVersion::V8,
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
            JniCall::ExceptionCheck(ExceptionCheckCall {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::ExceptionCheck(ExceptionCheckCall {
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
    fn __signature() -> string::String;
}

/// A macro for generating Java class boilerplate for Rust types, whcih is suitable for
/// `Object` type.
macro_rules! object_java_class {
    ($class:ident, $link:expr) => {
        /// Make
        #[doc = $link]
        /// mappable to [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'a> JavaType for $class<'a> {
            #[doc(hidden)]
            type __JniType = jni_sys::jobject;

            #[doc(hidden)]
            fn __signature() -> &'static str {
                concat!("L", "java/lang", "/", stringify!($class), ";")
            }
        }

        /// Make
        #[doc = $link]
        /// convertible to [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'a> ToJni for $class<'a> {
            unsafe fn __to_jni(&self) -> Self::__JniType {
                self.raw_object()
            }
        }

        /// Make
        #[doc = $link]
        /// castable to itself.
        impl<'env> Cast<'env, $class<'env>> for $class<'env> {
            fn cast<'a>(&'a self) -> &'a $class<'env> {
                self
            }
        }

        /// Allow comparing
        #[doc = $link]
        /// to Java objects. Java objects are compared by-reference to preserve
        /// original Java semantics. To compare objects by value, call the
        /// [`equals`](struct.Object.html#method.equals) method.
        ///
        /// Will panic if there is a pending exception in the current thread.
        ///
        /// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
        /// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env> Eq for $class<'env> {}
    };
}

/// A macro for generating Java class boilerplate for Rust types, except for the `Object` type.
macro_rules! java_class {
    ($class:ident, $link:expr) => {
        object_java_class!($class, $link);

        /// Make
        #[doc = $link]
        /// convertible from [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'env> FromJni<'env> for $class<'env> {
            unsafe fn __from_jni(env: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
                Self {
                    object: Object::__from_jni(env, value),
                }
            }
        }

        /// Allow
        #[doc = $link]
        /// to be used in place of an [`Object`](struct.Object.html).
        impl<'env> ::std::ops::Deref for $class<'env> {
            type Target = Object<'env>;

            fn deref(&self) -> &Self::Target {
                &self.object
            }
        }

        /// Make
        #[doc = $link]
        /// castable to [`Object`](struct.Object.html).
        impl<'env> Cast<'env, Object<'env>> for $class<'env> {
            fn cast<'a>(&'a self) -> &'a Object<'env> {
                self
            }
        }

        /// Allow comparing
        #[doc = $link]
        /// to Java objects. Java objects are compared by-reference to preserve
        /// original Java semantics. To compare objects by value, call the
        /// [`equals`](struct.Object.html#method.equals) method.
        ///
        /// Will panic if there is a pending exception in the current thread.
        ///
        /// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
        /// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env, T> PartialEq<T> for $class<'env> where T: Cast<'env, Object<'env>> {
            fn eq(&self, other: &T) -> bool {
                Object::cast(self).eq(other)
            }
        }

        /// Allow displaying Java objects.
        ///
        /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
        ///
        /// This is mostly a convenience for debugging. Always prefer using
        /// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env> fmt::Display for $class<'env> {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                Object::cast(self).fmt(formatter)
            }
        }

        impl<'env> $class<'env> {
            /// Clone the
            #[doc = $link]
            ///. This is not a deep clone of the Java object,
            /// but a Rust-like clone of the value. Since Java objects are reference counted, this
            /// will increment the reference count.
            ///
            /// This method has a different signature from the one in the
            /// [`Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html) trait
            /// because cloning a Java object is only safe when there is no pending exception and
            /// because cloning a java object cat throw an exception.
            ///
            /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newlocalref)
            pub fn clone(&self, token: &NoException<'env>) -> JavaResult<'env, Self>
            where
                Self: Sized,
            {
                self.object
                    .clone(token)
                    .map(|object| Self { object })
            }

            /// Convert the object to a string.
            ///
            /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
            //
            // This function is needed because Java classes implement
            // [`ToString`](https://doc.rust-lang.org/std/string/trait.ToString.html) trait through
            // the [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) trait, which
            // has a
            // [`to_string`](https://doc.rust-lang.org/std/string/trait.ToString.html#tymethod.to_string)
            // method which takes precedence over methods inherited from
            // [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html)-s.
            pub fn to_string(&self, token: &NoException<'env>) -> JavaResult<'env, String<'env>> {
                Object::cast(self).to_string(token)
            }
        }
    };
}

#[cfg(test)]
macro_rules! generate_object_tests {
    ($class:ident, $signature:expr) => {
        #[test]
        fn signature() {
            assert_eq!($class::__signature(), $signature);
        }

        #[test]
        fn to_jni() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let raw_object = 0x91011 as jni_sys::jobject;
            let object = test_value(&env, raw_object);
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
                let object = $class::__from_jni(&env, raw_object);
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
            let object = test_value(&env, raw_object);
            unsafe {
                let object = $class::__from_jni(&env, object.__to_jni());
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
                let object = $class::__from_jni(&env, raw_object);
                assert_eq!(object.__to_jni(), raw_object);
                mem::forget(object);
            }
        }

        #[test]
        fn drop() {
            const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::DeleteLocalRef(DeleteLocalRefCall {
                object: RAW_OBJECT,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            {
                let _object = test_value(&env, RAW_OBJECT);
                // Haven't called the destructor at this point yet.
                assert_eq!(calls.current_call, 0);
            }
        }

        #[test]
        fn clone() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x1234 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::NewLocalRef(NewLocalRefCall {
                object: RAW_OBJECT1,
                result: RAW_OBJECT2,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT1);
            let clone = object.clone(&NoException::test()).unwrap();
            calls.assert_eq(&clone, RAW_OBJECT2);
        }

        #[test]
        fn clone_null() {
            const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::NewLocalRef(NewLocalRefCall {
                    object: RAW_OBJECT,
                    result: ptr::null_mut() as jni_sys::jobject,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClearCall {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            let exception = object.clone(&NoException::test()).unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }

        #[test]
        fn eq() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::ExceptionCheck(ExceptionCheckCall {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::IsSameObject(IsSameObjectCall {
                    object1: RAW_OBJECT1,
                    object2: RAW_OBJECT2,
                    result: jni_sys::JNI_TRUE,
                }),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object1 = test_value(&env, RAW_OBJECT1);
            let object2 = test_value(&env, RAW_OBJECT2);
            assert!(object1 == object2);
        }

        #[test]
        fn eq_not_same() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::ExceptionCheck(ExceptionCheckCall {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::IsSameObject(IsSameObjectCall {
                    object1: RAW_OBJECT1,
                    object2: RAW_OBJECT2,
                    result: jni_sys::JNI_FALSE,
                }),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object1 = test_value(&env, RAW_OBJECT1);
            let object2 = test_value(&env, RAW_OBJECT2);
            assert!(object1 != object2);
        }

        #[test]
        #[should_panic(
            expected = "Comparing Java objects with a pending exception in the current thread"
        )]
        fn eq_pending_exception() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
                result: jni_sys::JNI_TRUE,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object1 = test_value(&env, RAW_OBJECT1);
            let object2 = test_value(&env, RAW_OBJECT2);
            let _ = object1 == object2;
        }

        #[test]
        fn to_string() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                RAW_STRING
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::GetObjectClass(GetObjectClassCall {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            unsafe {
                let string = object.to_string(&NoException::test()).unwrap();
                calls.assert_eq(&string, RAW_STRING);
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, calls.env);
            }
        }

        #[test]
        fn to_string_exception() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x248670 as jni_sys::jmethodID;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                _: *mut jni_sys::JNIEnv,
                _: jni_sys::jobject,
                _: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                ptr::null_mut()
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::GetObjectClass(GetObjectClassCall {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClearCall {}),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            let exception = object.to_string(&NoException::test()).unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }

        #[test]
        fn display() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
            const LENGTH: usize = 5;
            const SIZE: usize = 11; // `"test-string".len()`.
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                RAW_STRING
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::ExceptionCheck(ExceptionCheckCall {
                        result: jni_sys::JNI_FALSE,
                    }),
                    JniCall::GetObjectClass(GetObjectClassCall {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                    JniCall::GetStringLength(GetStringLengthCall {
                        string: RAW_STRING,
                        result: LENGTH as jni_sys::jsize,
                    }),
                    JniCall::GetStringUTFLength(GetStringUTFLengthCall {
                        string: RAW_STRING,
                        result: SIZE as jni_sys::jsize,
                    }),
                    JniCall::GetStringUTFRegion(GetStringUTFRegionCall {
                        string: RAW_STRING,
                        start: 0,
                        len: LENGTH as jni_sys::jsize,
                        buffer: "test-string".to_owned(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_STRING }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            assert_eq!(ToString::to_string(&object), "test-string");
        }

        #[test]
        #[should_panic(
            expected = "Displaying a Java object with a pending exception in the current thread"
        )]
        fn display_exception_pending() {
            let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheckCall {
                result: jni_sys::JNI_TRUE,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, ptr::null_mut());
            ToString::to_string(&object);
        }
    };
}

#[cfg(test)]
macro_rules! generate_tests {
    ($class:ident, $signature:expr) => {
        generate_object_tests!($class, $signature);

        #[test]
        fn deref_super() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let object = test_value(&env, ptr::null_mut());
            // Will not compile if is not deref-able.
            &object as &Deref<Target = Object>;
            mem::forget(object);
        }

        #[test]
        fn cast() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let object = test_value(&env, ptr::null_mut());
            assert_eq!(&object as *const _, object.cast() as *const _);
            assert_eq!(&object.object as *const _, object.cast() as *const _);
            mem::forget(object);
        }
    };
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

/// A type representing the
/// [`java.lang.Object`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html) class
/// -- the root class of Java's class hierarchy.
///
/// [`Object` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html)
// TODO: examples.
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

    /// Compare with another Java object by reference.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#issameobject)
    pub fn is_same_as(&self, other: &Object, _token: &NoException) -> bool {
        // Safe because arguments are ensured to be correct references by construction.
        let same = unsafe {
            call_jni_method!(
                self.env(),
                IsSameObject,
                self.raw_object(),
                other.raw_object()
            )
        };
        // Safe because `bool` conversion is safe internally.
        unsafe { bool::__from_jni(self.env(), same) }
    }

    /// Check if the object is an instance of the class.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#isinstanceof)
    pub fn is_instance_of(&self, class: &Class, _token: &NoException) -> bool {
        // Safe because arguments are ensured to be correct references by construction.
        let is_instance = unsafe {
            call_jni_method!(
                self.env(),
                IsInstanceOf,
                self.raw_object(),
                class.raw_object()
            )
        };
        // Safe because `bool` conversion is safe internally.
        unsafe { bool::__from_jni(self.env(), is_instance) }
    }

    /// Clone the [`Object`](struct.Object.html). This is not a deep clone of the Java object,
    /// but a Rust-like clone of the value. Since Java objects are reference counted, this will
    /// increment the reference count.
    ///
    /// This method has a different signature from the one in the
    /// [`Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html) trait because
    /// cloning a Java object is only safe when there is no pending exception and because
    /// cloning a java object cat throw an exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newlocalref)
    pub fn clone(&self, token: &NoException<'env>) -> JavaResult<'env, Object<'env>> {
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewLocalRef` throws an exception before returning `null`.
        let raw_object =
            unsafe { call_nullable_jni_method!(self.env, NewLocalRef, token, self.raw_object)? };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(self.env, raw_object) })
    }

    /// Convert the object to a string.
    ///
    /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
    pub fn to_string(&self, token: &NoException<'env>) -> JavaResult<'env, String<'env>> {
        // Safe, because the method name and the signature are correct.
        unsafe { call_method::<Self, _, _, fn() -> String<'env>>(self, "toString", (), token) }
    }

    /// Construct from a raw pointer. Unsafe because an invalid pointer may be passed
    /// as the argument.
    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_raw(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Self {
        Self { env, raw_object }
    }
}

object_java_class!(Object, "[`Object`](struct.Object.html)");

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

/// Allow comparing [`Object`](struct.Object.html) to Java objects. Java objects are compared
/// by-reference to preserve original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for Object<'env>
where
    T: Cast<'env, Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        if self.env().has_exception() {
            panic!("Comparing Java objects with a pending exception in the current thread")
        } else {
            // Safe because we checked that there is no pending exception.
            let token = unsafe { NoException::new_env(self.env()) };
            self.is_same_as(other.cast(), &token)
        }
    }
}

/// Allow displaying Java objects for debug purposes.
///
/// [`Object::toString`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
///
/// This is mostly a convenience for debugging. Always prefer using
/// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env> fmt::Debug for Object<'env> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.env.has_exception() {
            // Can't call `to_string` with a pending exception.
            write!(
                formatter,
                "Object {{ env: {:?}, object: {:?}, string: \
                 <can't call Object::toString string because of a pending exception in the current thread> }}",
                self.env, self.raw_object
            )
        } else {
            // Safe because we checked that there is no pending exception.
            let token = unsafe { NoException::new_env(self.env) };
            match self.to_string(&token) {
                Ok(string) => write!(
                    formatter,
                    "Object {{ env: {:?}, object: {:?} string: {} }}",
                    self.env,
                    self.raw_object,
                    string.as_string(&token),
                ),
                Err(exception) => match exception.to_string(&token) {
                    Ok(message) => write!(
                        formatter,
                        "Object {{ env: {:?}, object: {:?}, string: \
                         <Object::toString threw an exception: {:?}> }}",
                        self.env,
                        self.raw_object,
                        message.as_string(&token)
                    ),
                    Err(_) => write!(
                        formatter,
                        "Object {{ env: {:?}, object: {:?}, string: \
                         <Object::toString threw an exception> }}",
                        self.env, self.raw_object
                    ),
                },
            }
        }
    }
}

/// Allow displaying Java objects.
///
/// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
///
/// This is mostly a convenience for debugging. Always prefer using
/// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env> fmt::Display for Object<'env> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.env.has_exception() {
            panic!("Displaying a Java object with a pending exception in the current thread.");
        } else {
            // Safe because we checked that there is no pending exception.
            let token = unsafe { NoException::new_env(self.env) };
            match self.to_string(&token) {
                Ok(string) => write!(formatter, "{}", string.as_string(&token)),
                Err(exception) => match exception.to_string(&token) {
                    Ok(message) => write!(
                        formatter,
                        "Object::toString threw an exception: {}",
                        message.as_string(&token)
                    ),
                    Err(_) => write!(
                        formatter,
                        "<Object::toString threw an exception which could not be formatted>"
                    ),
                },
            }
        }
    }
}

#[cfg(test)]
pub fn test_object<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Object<'env> {
    Object { env, raw_object }
}

#[cfg(test)]
mod object_tests {
    use super::*;
    use std::ffi::CStr;
    use std::mem;
    use testing::*;

    #[cfg(test)]
    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Object<'env> {
        test_object(env, raw_object)
    }

    generate_object_tests!(Object, "Ljava/lang/Object;");

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
    fn cast() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let object = test_value(&env, ptr::null_mut());
        assert_eq!(&object as *const _, object.cast() as *const _);
        mem::forget(object);
    }

    #[test]
    fn class() {
        const RAW_OBJECT: jni_sys::jobject = 0x093599 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x347658 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetObjectClass(GetObjectClassCall {
            object: RAW_OBJECT,
            result: RAW_CLASS,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        let class = object.class(&NoException::test());
        calls.assert_eq(&class, RAW_CLASS);
    }

    #[test]
    #[should_panic(expected = "doesn't have a class")]
    fn class_not_found() {
        const RAW_OBJECT: jni_sys::jobject = 0x093599 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetObjectClass(GetObjectClassCall {
            object: RAW_OBJECT,
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        object.class(&NoException::test());
    }

    #[test]
    fn is_same_as_same() {
        const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsSameObject(IsSameObjectCall {
            object1: RAW_OBJECT1,
            object2: RAW_OBJECT2,
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object1 = test_value(&env, RAW_OBJECT1);
        let object2 = test_value(&env, RAW_OBJECT2);
        assert!(object1.is_same_as(&object2, &NoException::test()));
    }

    #[test]
    fn is_same_as_not_same() {
        const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsSameObject(IsSameObjectCall {
            object1: RAW_OBJECT1,
            object2: RAW_OBJECT2,
            result: jni_sys::JNI_FALSE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object1 = test_value(&env, RAW_OBJECT1);
        let object2 = test_value(&env, RAW_OBJECT2);
        assert!(!object1.is_same_as(&object2, &NoException::test()));
    }

    #[test]
    fn is_instance_of() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsInstanceOf(IsInstanceOfCall {
            object: RAW_OBJECT,
            class: RAW_CLASS,
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_object(&env, RAW_OBJECT);
        let class = test_class(&env, RAW_CLASS);
        assert!(object.is_instance_of(&class, &NoException::test()));
    }

    #[test]
    fn is_not_instance_of() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsInstanceOf(IsInstanceOfCall {
            object: RAW_OBJECT,
            class: RAW_CLASS,
            result: jni_sys::JNI_FALSE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_object(&env, RAW_OBJECT);
        let class = test_class(&env, RAW_CLASS);
        assert!(!object.is_instance_of(&class, &NoException::test()));
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
    /// Create a new [`Throwable`](struct.Throwable.html) with a message.
    ///
    /// [`Throwable(String)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#<init>(java.lang.String))
    pub fn new<'a>(
        env: &'a JniEnv<'a>,
        message: String<'a>,
        token: &NoException<'a>,
    ) -> JavaResult<'a, Throwable<'a>> {
        // Safe because the name and the signature are correct.
        unsafe { call_constructor::<Throwable<'a>, _, fn(String)>(env, (message,), token) }
    }

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

java_class!(Throwable, "[`Throwable`](struct.Throwable.html)");

#[cfg(test)]
mod throwable_tests {
    use super::*;
    use std::ffi::CStr;
    use std::mem;
    use std::ops::Deref;
    use testing::*;

    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Throwable<'env> {
        Throwable {
            object: test_object(env, raw_object),
        }
    }

    generate_tests!(Throwable, "Ljava/lang/Throwable;");

    #[test]
    fn throw() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::Throw(ThrowCall {
            object: RAW_OBJECT,
            result: jni_sys::JNI_OK,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        object.throw(NoException::test());
    }

    #[test]
    #[should_panic(expected = "Throwing an exception has failed with status -1.")]
    fn throw_failed() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::Throw(ThrowCall {
            object: RAW_OBJECT,
            result: jni_sys::JNI_ERR,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        object.throw(NoException::test());
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
        let class_name = to_java_string(class_name);
        // Safe because the arguments are correct and because `FindClass` throws an exception
        // before returning `null`.
        let raw_class = unsafe {
            call_nullable_jni_method!(env, FindClass, token, class_name.as_ptr() as *const c_char)?
        };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(env, raw_class) })
    }

    /// Get the parent class of this class. Will return
    /// [`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None) for the
    /// [`Object`](struct.Object.html) class or any interface.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getsuperclass)
    pub fn parent(&self, _token: &NoException) -> Option<Class<'env>> {
        // Safe because the argument is ensured to be correct references by construction.
        let raw_java_class =
            unsafe { call_jni_method!(self.env(), GetSuperclass, self.raw_object()) };
        if raw_java_class == ptr::null_mut() {
            None
        } else {
            // Safe because the argument is ensured to be a correct reference.
            Some(unsafe { Self::__from_jni(self.env(), raw_java_class) })
        }
    }

    /// Check if this class is a subtype of the other class.
    ///
    /// In Java a class is a subtype of the other class if that other class is a direct or
    /// an indirect parent of this class or an interface this class or any it's parent is
    /// implementing.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#isassignablefrom)
    pub fn is_subtype_of(&self, class: &Class, _token: &NoException) -> bool {
        // Safe because arguments are ensured to be the correct by construction.
        let assignable = unsafe {
            call_jni_method!(
                self.env(),
                IsAssignableFrom,
                self.raw_object() as jni_sys::jclass,
                class.raw_object() as jni_sys::jclass
            )
        };
        // Safe because `bool` conversion is safe internally.
        unsafe { bool::__from_jni(self.env(), assignable) }
    }

    /// Unsafe because the argument mught not be a valid class reference.
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, raw_class: jni_sys::jclass) -> Class<'a> {
        Class {
            object: Object::__from_jni(env, raw_class as jni_sys::jobject),
        }
    }
}

java_class!(Class, "[`Class`](struct.Class.html)");

#[cfg(test)]
pub fn test_class<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Class<'env> {
    Class {
        object: test_object(env, raw_object),
    }
}

#[cfg(test)]
mod class_tests {
    use super::*;
    use std::ffi::CStr;
    use std::mem;
    use std::ops::Deref;
    use testing::*;

    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Class<'env> {
        test_class(env, raw_object)
    }

    generate_tests!(Class, "Ljava/lang/Class;");

    #[test]
    fn find() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::FindClass(FindClassCall {
            name: "test-class".to_owned(),
            result: RAW_OBJECT,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = Class::find(&env, "test-class", &NoException::test()).unwrap();
        calls.assert_eq(&class, RAW_OBJECT);
    }

    #[test]
    fn find_not_found() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::FindClass(FindClassCall {
                name: "test-class".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = Class::find(&env, "test-class", &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn is_subtype_of() {
        const RAW_CLASS1: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const RAW_CLASS2: jni_sys::jobject = 0x294875 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsAssignableFrom(IsAssignableFromCall {
            class1: RAW_CLASS1,
            class2: RAW_CLASS2,
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class1 = test_class(&env, RAW_CLASS1);
        let class2 = test_class(&env, RAW_CLASS2);
        assert!(class1.is_subtype_of(&class2, &NoException::test()));
    }

    #[test]
    fn is_not_subtype_of() {
        const RAW_CLASS1: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const RAW_CLASS2: jni_sys::jobject = 0x294875 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsAssignableFrom(IsAssignableFromCall {
            class1: RAW_CLASS1,
            class2: RAW_CLASS2,
            result: jni_sys::JNI_FALSE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class1 = test_class(&env, RAW_CLASS1);
        let class2 = test_class(&env, RAW_CLASS2);
        assert!(!class1.is_subtype_of(&class2, &NoException::test()));
    }

    #[test]
    fn parent() {
        const RAW_CLASS1: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const RAW_CLASS2: jni_sys::jobject = 0x294875 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetSuperclass(GetSuperclassCall {
            class: RAW_CLASS1,
            result: RAW_CLASS2,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = test_class(&env, RAW_CLASS1);
        let parent = class.parent(&NoException::test()).unwrap();
        calls.assert_eq(&parent, RAW_CLASS2);
    }

    #[test]
    fn no_parent() {
        const RAW_CLASS: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetSuperclass(GetSuperclassCall {
            class: RAW_CLASS,
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = test_class(&env, RAW_CLASS);
        assert!(class.parent(&NoException::test()).is_none());
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
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewString` throws an exception before returning `null`.
        let raw_string = unsafe {
            call_nullable_jni_method!(env, NewString, token, ptr::null(), 0 as jni_sys::jsize)?
        };
        // Safe because the argument is a valid string reference.
        Ok(unsafe { Self::from_raw(env, raw_string) })
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
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewStringUTF` throws an exception before returning `null`.
        let raw_string = unsafe {
            call_nullable_jni_method!(env, NewStringUTF, token, buffer.as_ptr() as *const c_char)?
        };
        // Safe because the argument is a valid string reference.
        Ok(unsafe { Self::from_raw(env, raw_string) })
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

    /// Get the string value of an integer.
    ///
    /// [`String::valueOf(int)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/String.html#valueOf(int))
    pub fn value_of_int(
        env: &'env JniEnv<'env>,
        value: i32,
        token: &NoException<'env>,
    ) -> JavaResult<'env, String<'env>> {
        // Safe because the name and the signature are correct.
        unsafe {
            call_static_method::<Self, _, _, fn(i32) -> String<'env>>(
                env,
                "valueOf",
                (value,),
                token,
            )
        }
    }

    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, raw_string: jni_sys::jstring) -> String<'a> {
        String {
            object: Object::__from_jni(env, raw_string as jni_sys::jobject),
        }
    }
}

java_class!(String, "[`String`](struct.String.html)");

#[cfg(test)]
mod string_tests {
    use super::*;
    use std::ffi::CStr;
    use std::mem;
    use std::ops::Deref;
    use testing::*;

    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> String<'env> {
        String {
            object: test_object(env, raw_object),
        }
    }

    generate_tests!(String, "Ljava/lang/String;");

    #[test]
    fn empty() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::NewString(NewStringCall {
            name: ptr::null(),
            size: 0,
            result: RAW_STRING,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = String::empty(&env, &NoException::test()).unwrap();
        calls.assert_eq(&string, RAW_STRING);
    }

    #[test]
    fn empty_exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::NewString(NewStringCall {
                name: ptr::null(),
                size: 0,
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = String::empty(&env, &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn new_empty() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::NewString(NewStringCall {
            name: ptr::null(),
            size: 0,
            result: RAW_STRING,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = String::new(&env, "", &NoException::test()).unwrap();
        calls.assert_eq(&string, RAW_STRING);
    }

    #[test]
    fn new_empty_exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::NewString(NewStringCall {
                name: ptr::null(),
                size: 0,
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = String::new(&env, "", &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn new() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::NewStringUTF(NewStringUTFCall {
            string: "test-string".to_owned(),
            result: RAW_STRING,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = String::new(&env, "test-string", &NoException::test()).unwrap();
        calls.assert_eq(&string, RAW_STRING);
    }

    #[test]
    fn new_exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::NewStringUTF(NewStringUTFCall {
                string: "test-string".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = String::new(&env, "test-string", &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn len() {
        const LENGTH: usize = 17;
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetStringLength(GetStringLengthCall {
            string: RAW_STRING,
            result: 17 as jni_sys::jsize,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.len(&NoException::test()), LENGTH);
    }

    #[test]
    fn size() {
        const LENGTH: usize = 17;
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetStringUTFLength(GetStringUTFLengthCall {
            string: RAW_STRING,
            result: 17 as jni_sys::jsize,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.size(&NoException::test()), LENGTH + 1);
    }

    #[test]
    fn as_string() {
        const LENGTH: usize = 5;
        const SIZE: usize = 11; // `"test-string".len()`.
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::GetStringLength(GetStringLengthCall {
                string: RAW_STRING,
                result: LENGTH as jni_sys::jsize,
            }),
            JniCall::GetStringUTFLength(GetStringUTFLengthCall {
                string: RAW_STRING,
                result: SIZE as jni_sys::jsize,
            }),
            JniCall::GetStringUTFRegion(GetStringUTFRegionCall {
                string: RAW_STRING,
                start: 0,
                len: LENGTH as jni_sys::jsize,
                buffer: "test-string".to_owned(),
            }),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.as_string(&NoException::test()), "test-string");
    }

    #[test]
    fn as_string_empty() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetStringLength(GetStringLengthCall {
            string: RAW_STRING,
            result: 0,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.as_string(&NoException::test()), "");
    }

    #[test]
    fn to_string_call() {
        static mut FIND_CLASS_CALLS: i32 = 0;
        static mut FIND_CLASS_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut FIND_CLASS_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn find_class(
            env: *mut jni_sys::JNIEnv,
            name: *const c_char,
        ) -> jni_sys::jobject {
            assert_eq!(
                from_java_string(CStr::from_ptr(name).to_bytes_with_nul()).unwrap(),
                "java/lang/String"
            );
            FIND_CLASS_CALLS += 1;
            FIND_CLASS_ENV_ARGUMENT = env;
            FIND_CLASS_RESULT
        }
        static mut GET_METHOD_ID_CALLS: i32 = 0;
        static mut GET_METHOD_ID_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut GET_METHOD_ID_CLASS_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut GET_METHOD_ID_RESULT: jni_sys::jmethodID = ptr::null_mut();
        unsafe extern "system" fn get_method_id(
            env: *mut jni_sys::JNIEnv,
            class: jni_sys::jobject,
            name: *const c_char,
            signature: *const c_char,
        ) -> jni_sys::jmethodID {
            assert_eq!(
                from_java_string(CStr::from_ptr(name).to_bytes_with_nul()).unwrap(),
                "valueOf"
            );
            assert_eq!(
                from_java_string(CStr::from_ptr(signature).to_bytes_with_nul()).unwrap(),
                "(I)Ljava/lang/String;"
            );
            GET_METHOD_ID_CALLS += 1;
            GET_METHOD_ID_ENV_ARGUMENT = env;
            GET_METHOD_ID_CLASS_ARGUMENT = class;
            GET_METHOD_ID_RESULT
        }
        static mut METHOD_CALLS: i32 = 0;
        static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut METHOD_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut METHOD_METHOD_ARGUMENT: jni_sys::jmethodID = ptr::null_mut();
        static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
        static mut METHOD_RESULT: jni_sys::jobject = ptr::null_mut();
        type VariadicFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            ...
        ) -> jni_sys::jobject;
        type TestFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            argument0: jni_sys::jint,
        ) -> jni_sys::jobject;
        unsafe extern "C" fn method(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            argument0: jni_sys::jint,
        ) -> jni_sys::jobject {
            METHOD_CALLS += 1;
            METHOD_ENV_ARGUMENT = env;
            METHOD_OBJECT_ARGUMENT = object;
            METHOD_METHOD_ARGUMENT = method_id;
            METHOD_ARGUMENT0 = argument0;
            METHOD_RESULT
        }
        static mut EXCEPTION_OCCURED_CALLS: i32 = 0;
        static mut EXCEPTION_OCCURED_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_occured(env: *mut jni_sys::JNIEnv) -> jni_sys::jobject {
            EXCEPTION_OCCURED_CALLS += 1;
            EXCEPTION_OCCURED_ENV_ARGUMENT = env;
            ptr::null_mut()
        }
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
            FindClass: Some(find_class),
            GetStaticMethodID: Some(get_method_id),
            CallStaticObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ExceptionOccurred: Some(exception_occured),
            DeleteLocalRef: Some(delete_local_ref),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_class = 0x57469 as jni_sys::jclass;
        let method_id = 0x7654 as jni_sys::jmethodID;
        let argument = 17;
        let result = 0x569805 as jni_sys::jstring;
        unsafe {
            FIND_CLASS_RESULT = raw_class;
            GET_METHOD_ID_RESULT = method_id;
            METHOD_RESULT = result;
            let string = String::value_of_int(&env, argument, &NoException::test()).unwrap();
            assert_eq!(string.raw_object(), result);
            assert_eq!(string.env().raw_env(), raw_jni_env);
            assert_eq!(FIND_CLASS_CALLS, 1);
            assert_eq!(FIND_CLASS_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_METHOD_ID_CALLS, 1);
            assert_eq!(GET_METHOD_ID_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(GET_METHOD_ID_CLASS_ARGUMENT, raw_class);
            assert_eq!(METHOD_CALLS, 1);
            assert_eq!(METHOD_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(METHOD_OBJECT_ARGUMENT, raw_class);
            assert_eq!(METHOD_METHOD_ARGUMENT, method_id);
            assert_eq!(METHOD_ARGUMENT0, argument);
            assert_eq!(EXCEPTION_OCCURED_CALLS, 1);
            assert_eq!(EXCEPTION_OCCURED_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(DELETE_LOCAL_REF_CALLS, 1);
            assert_eq!(DELETE_LOCAL_REF_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(DELETE_LOCAL_REF_OBJECT_ARGUMENT, raw_class);
        }
    }

    #[test]
    fn to_string_exception_thrown() {
        unsafe extern "system" fn find_class(
            _: *mut jni_sys::JNIEnv,
            _: *const c_char,
        ) -> jni_sys::jobject {
            0x57469 as jni_sys::jclass
        }
        unsafe extern "system" fn get_method_id(
            _: *mut jni_sys::JNIEnv,
            _: jni_sys::jobject,
            _: *const c_char,
            _: *const c_char,
        ) -> jni_sys::jmethodID {
            0x257949 as jni_sys::jmethodID
        }
        type VariadicFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            ...
        ) -> jni_sys::jobject;
        type TestFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jobject;
        unsafe extern "C" fn method(
            _: *mut jni_sys::JNIEnv,
            _: jni_sys::jobject,
            _: jni_sys::jmethodID,
        ) -> jni_sys::jobject {
            ptr::null_mut()
        }
        static mut EXCEPTION_OCCURED_CALLS: i32 = 0;
        static mut EXCEPTION_OCCURED_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut EXCEPTION_OCCURED_RESULT: jni_sys::jobject = ptr::null_mut();
        unsafe extern "system" fn exception_occured(env: *mut jni_sys::JNIEnv) -> jni_sys::jobject {
            EXCEPTION_OCCURED_CALLS += 1;
            EXCEPTION_OCCURED_ENV_ARGUMENT = env;
            EXCEPTION_OCCURED_RESULT
        }
        static mut EXCEPTION_CLEAR_CALLS: i32 = 0;
        static mut EXCEPTION_CLEAR_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        unsafe extern "system" fn exception_clear(env: *mut jni_sys::JNIEnv) {
            EXCEPTION_CLEAR_CALLS += 1;
            EXCEPTION_CLEAR_ENV_ARGUMENT = env;
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            FindClass: Some(find_class),
            GetStaticMethodID: Some(get_method_id),
            CallStaticObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ExceptionOccurred: Some(exception_occured),
            ExceptionClear: Some(exception_clear),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_exception = 0x32885 as jni_sys::jthrowable;
        unsafe {
            EXCEPTION_OCCURED_RESULT = raw_exception;
            let exception = String::value_of_int(&env, 17, &NoException::test()).unwrap_err();
            assert_eq!(exception.raw_object(), raw_exception);
            assert_eq!(exception.env().raw_env(), raw_jni_env);
            assert_eq!(EXCEPTION_OCCURED_CALLS, 1);
            assert_eq!(EXCEPTION_OCCURED_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(EXCEPTION_CLEAR_CALLS, 1);
            assert_eq!(EXCEPTION_CLEAR_ENV_ARGUMENT, raw_jni_env);
        }
    }
}

/// Get and clear the pending exception.
fn maybe_get_and_clear_exception<'a>(env: &'a JniEnv<'a>) -> Option<Throwable<'a>> {
    // Safe because the argument is ensured to be correct references by construction.
    let raw_java_throwable = unsafe { call_jni_method!(env, ExceptionOccurred) };
    if raw_java_throwable == ptr::null_mut() {
        return None;
    }
    // Safe because the argument is ensured to be correct references by construction.
    unsafe {
        call_jni_method!(env, ExceptionClear);
    }
    // Safe because the arguments are correct.
    Some(unsafe { Throwable::__from_jni(env, raw_java_throwable) })
}

#[cfg(test)]
mod maybe_get_and_clear_exception_tests {
    use super::*;
    use testing::*;

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = maybe_get_and_clear_exception(&env).unwrap();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn exception_not_found() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionOccurred(ExceptionOccurredCall {
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        assert_eq!(maybe_get_and_clear_exception(&env), None);
    }
}

/// Get and clear the pending exception.
fn get_and_clear_exception<'a>(token: Exception<'a>) -> Throwable<'a> {
    match maybe_get_and_clear_exception(token.env) {
        None => panic!(
            "No pending exception in presence of an Exception token. Should not ever happen."
        ),
        Some(exception) => exception,
    }
}

#[cfg(test)]
mod get_and_clear_exception_tests {
    use super::*;
    use testing::*;

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = get_and_clear_exception(Exception::test(&env));
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    #[should_panic(expected = "No pending exception in presence of an Exception token")]
    fn exception_not_found() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionOccurred(ExceptionOccurredCall {
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        get_and_clear_exception(Exception::test(&env));
    }
}

/// Take a function that produces a [`JniResult`](type.JniResult.html), call it and produce
/// a [`JavaResult`](type.JavaResult.html) from it.
fn with_checked_exception<'a, Out, T: FnOnce(NoException<'a>) -> JniResult<'a, Out>>(
    token: &NoException<'a>,
    function: T,
) -> JavaResult<'a, Out> {
    // Safe, because we check for a pending exception after the call.
    let token = unsafe { token.clone() };
    match function(token) {
        Ok((value, _)) => Ok(value),
        Err(token) => Err(get_and_clear_exception(token)),
    }
}

#[cfg(test)]
mod with_checked_exception_tests {
    use super::*;
    use testing::*;

    #[test]
    fn no_exception() {
        let result = with_checked_exception(&NoException::test(), |_| {
            Ok((17, NoException::test()))
        }).unwrap();
        assert_eq!(result, 17);
    }

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClearCall {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = with_checked_exception::<i32, _>(&NoException::test(), |_| {
            Err(Exception::test(&env))
        }).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }
}

pub mod method_calls {
    use super::*;

    /// Get the id of a method by it's name and type.
    /// Unsafe, because it is possible to send ivalid name-type combination.
    unsafe fn get_method_id<
        'env,
        In: ToJniTuple,
        Out: FromJni<'env>,
        T: JavaMethodSignature<In, Out> + ?Sized,
    >(
        class: &Class<'env>,
        name: &str,
        token: &NoException<'env>,
    ) -> JavaResult<'env, jni_sys::jmethodID> {
        let name = to_java_string(name);
        let signature = to_java_string(&T::__signature());
        // Safe because arguments are ensured to be the correct by construction and because
        // `GetMethodID` throws an exception before returning `null`.
        call_nullable_jni_method!(
            class.env(),
            GetMethodID,
            token,
            class.raw_object(),
            name.as_ptr() as *const c_char,
            signature.as_ptr() as *const c_char
        )
    }

    /// Get the id of a static method by it's name and type.
    /// Unsafe, because it is possible to pass an ivalid name-type combination.
    unsafe fn get_static_method_id<
        'env,
        In: ToJniTuple,
        Out: FromJni<'env>,
        T: JavaMethodSignature<In, Out> + ?Sized,
    >(
        class: &Class<'env>,
        name: &str,
        token: &NoException<'env>,
    ) -> JavaResult<'env, jni_sys::jmethodID> {
        let name = to_java_string(name);
        let signature = to_java_string(&T::__signature());
        // Safe because arguments are ensured to be the correct by construction and because
        // `GetStaticMethodID` throws an exception before returning `null`.
        call_nullable_jni_method!(
            class.env(),
            GetStaticMethodID,
            token,
            class.raw_object(),
            name.as_ptr() as *const c_char,
            signature.as_ptr() as *const c_char
        )
    }

    /// Get the id of a static method by it's name and type.
    /// Unsafe, because it's possible to pass an incorrect result.
    unsafe fn from_method_call_result<'env, Out: FromJni<'env>>(
        env: &'env JniEnv<'env>,
        result: Out::__JniType,
    ) -> JavaResult<'env, Out> {
        match maybe_get_and_clear_exception(env) {
            // Safe because arguments are ensured to be the correct by construction.
            None => Ok(Out::__from_jni(env, result)),
            Some(exception) => Err(exception),
        }
    }

    /// Call a method on an object by it's name.
    ///
    /// THIS FUNCTION SHOULD NOT BE CALLED MANUALLY.
    ///
    /// This method is unsafe because a caller can provide an incorrect method name or arguments.
    #[doc(hidden)]
    pub unsafe fn call_method<
        'env,
        Class: Cast<'env, Object<'env>> + 'env,
        In: ToJniTuple,
        Out: FromJni<'env>,
        T: JavaMethodSignature<In, Out> + ?Sized,
    >(
        object: &Class,
        name: &str,
        arguments: In,
        token: &NoException<'env>,
    ) -> JavaResult<'env, Out> {
        let class = object.cast().class(&token);
        let method_id = get_method_id::<_, _, T>(&class, name, token)?;
        // Safe because arguments are ensured to be the correct by construction.
        let result = Out::__JniType::call_method(object.cast(), method_id, arguments);
        from_method_call_result(object.cast().env(), result)
    }

    #[cfg(test)]
    mod call_method_tests {
        use super::*;
        use std::mem;
        use testing::*;

        #[test]
        fn call() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
            static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jboolean;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jboolean;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jboolean {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                METHOD_ARGUMENT0 = argument0;
                METHOD_ARGUMENT1 = argument1;
                jni_sys::JNI_TRUE
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallBooleanMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::GetObjectClass(GetObjectClassCall {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "test-method".to_owned(),
                        signature: "(ID)Z".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_object(&env, RAW_OBJECT);
            let arguments = (17 as i32, 19. as f64);
            unsafe {
                assert!(
                    super::call_method::<Object, _, _, fn(i32, f64) -> bool>(
                        &object,
                        "test-method",
                        arguments,
                        &NoException::test()
                    ).unwrap()
                );
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, calls.env);
                assert_eq!(METHOD_ARGUMENT0, arguments.0);
                assert_eq!(METHOD_ARGUMENT1, arguments.1);
            }
        }

        #[test]
        fn no_such_method() {
            const RAW_OBJECT: jni_sys::jobject = 0x098957 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::GetObjectClass(GetObjectClassCall {
                    object: RAW_OBJECT,
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodIDCall {
                    class: RAW_CLASS,
                    name: "test-method".to_owned(),
                    signature: "()Z".to_owned(),
                    result: ptr::null_mut(),
                }),
                JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClearCall {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_object(&env, RAW_OBJECT);
            unsafe {
                let exception = super::call_method::<Object, _, _, fn() -> bool>(
                    &object,
                    "test-method",
                    (),
                    &NoException::test(),
                ).unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }

        #[test]
        fn exception_thrown() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x938475 as jni_sys::jmethodID;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jboolean;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jboolean;
            unsafe extern "C" fn method(
                _: *mut jni_sys::JNIEnv,
                _: jni_sys::jobject,
                _: jni_sys::jmethodID,
            ) -> jni_sys::jboolean {
                jni_sys::JNI_FALSE
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallBooleanMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::GetObjectClass(GetObjectClassCall {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "test-method".to_owned(),
                        signature: "()Z".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClearCall {}),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_object(&env, RAW_OBJECT);
            unsafe {
                let exception = super::call_method::<Object, _, _, fn() -> bool>(
                    &object,
                    "test-method",
                    (),
                    &NoException::test(),
                ).unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }
    }

    /// Call a static method on a class by it's name.
    ///
    /// THIS FUNCTION SHOULD NOT BE CALLED MANUALLY.
    ///
    /// This method is unsafe because a caller can provide an incorrect method name or arguments.
    #[doc(hidden)]
    pub unsafe fn call_static_method<
        'env,
        Class: Cast<'env, Object<'env>>,
        In: ToJniTuple,
        Out: FromJni<'env>,
        T: JavaMethodSignature<In, Out> + ?Sized,
    >(
        env: &'env JniEnv<'env>,
        name: &str,
        arguments: In,
        token: &NoException<'env>,
    ) -> JavaResult<'env, Out> {
        let signature = Class::__signature();
        let class = self::Class::find(env, &signature[1..signature.len() - 1], token)?;
        let method_id = get_static_method_id::<_, _, T>(&class, name, token)?;
        // Safe because arguments are ensured to be the correct by construction.
        let result = Out::__JniType::call_static_method(&class, method_id, arguments);
        from_method_call_result(env, result)
    }

    #[cfg(test)]
    mod call_static_method_tests {
        use super::*;
        use std::mem;
        use testing::*;

        #[test]
        fn call() {
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
            static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jboolean;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jboolean;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jboolean {
                assert_eq!(object, RAW_CLASS);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                METHOD_ARGUMENT0 = argument0;
                METHOD_ARGUMENT1 = argument1;
                jni_sys::JNI_TRUE
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallStaticBooleanMethod: Some(unsafe {
                    mem::transmute::<TestFn, VariadicFn>(method)
                }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::FindClass(FindClassCall {
                        name: "java/lang/Object".to_owned(),
                        result: RAW_CLASS,
                    }),
                    JniCall::GetStaticMethodID(GetStaticMethodIDCall {
                        class: RAW_CLASS,
                        name: "test-method".to_owned(),
                        signature: "(ID)Z".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let arguments = (17 as i32, 19. as f64);
            unsafe {
                assert!(
                    super::call_static_method::<Object, _, _, fn(i32, f64) -> bool>(
                        &env,
                        "test-method",
                        arguments,
                        &NoException::test()
                    ).unwrap()
                );
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, calls.env);
                assert_eq!(METHOD_ARGUMENT0, arguments.0);
                assert_eq!(METHOD_ARGUMENT1, arguments.1);
            }
        }

        #[test]
        fn no_such_class() {
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::FindClass(FindClassCall {
                    name: "java/lang/Object".to_owned(),
                    result: ptr::null_mut(),
                }),
                JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClearCall {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            unsafe {
                let exception = super::call_static_method::<Object, _, _, fn() -> bool>(
                    &env,
                    "",
                    (),
                    &NoException::test(),
                ).unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }

        #[test]
        fn no_such_method() {
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::FindClass(FindClassCall {
                    name: "java/lang/Object".to_owned(),
                    result: RAW_CLASS,
                }),
                JniCall::GetStaticMethodID(GetStaticMethodIDCall {
                    class: RAW_CLASS,
                    name: "test-method".to_owned(),
                    signature: "()Z".to_owned(),
                    result: ptr::null_mut(),
                }),
                JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClearCall {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            unsafe {
                let exception = super::call_static_method::<Object, _, _, fn() -> bool>(
                    &env,
                    "test-method",
                    (),
                    &NoException::test(),
                ).unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }

        #[test]
        fn exception_thrown() {
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x928375 as jni_sys::jmethodID;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jboolean;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jboolean;
            unsafe extern "C" fn method(
                _: *mut jni_sys::JNIEnv,
                _: jni_sys::jobject,
                _: jni_sys::jmethodID,
            ) -> jni_sys::jboolean {
                jni_sys::JNI_FALSE
            }
            let vm = test_vm(ptr::null_mut());
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallStaticBooleanMethod: Some(unsafe {
                    mem::transmute::<TestFn, VariadicFn>(method)
                }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::FindClass(FindClassCall {
                        name: "java/lang/Object".to_owned(),
                        result: RAW_CLASS,
                    }),
                    JniCall::GetStaticMethodID(GetStaticMethodIDCall {
                        class: RAW_CLASS,
                        name: "test-method".to_owned(),
                        signature: "()Z".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClearCall {}),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let env = test_env(&vm, calls.env);
            unsafe {
                let exception = super::call_static_method::<Object, _, _, fn() -> bool>(
                    &env,
                    "test-method",
                    (),
                    &NoException::test(),
                ).unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }
    }

    /// Call a constructor of a class by it's name.
    ///
    /// THIS FUNCTION SHOULD NOT BE CALLED MANUALLY.
    ///
    /// This method is unsafe because a caller can provide incorrect arguments.
    #[doc(hidden)]
    pub unsafe fn call_constructor<
        'env,
        Class: Cast<'env, Object<'env>>,
        In: ToJniTuple,
        T: JavaMethodSignature<In, ()> + ?Sized,
    >(
        env: &'env JniEnv<'env>,
        arguments: In,
        token: &NoException<'env>,
    ) -> JavaResult<'env, Class> {
        let signature = Class::__signature();
        let class = self::Class::find(env, &signature[1..signature.len() - 1], token)?;
        let method_id = get_method_id::<_, _, T>(&class, "<init>", token)?;
        // Safe because arguments are ensured to be the correct by construction.
        let result = In::call_constructor(&class, method_id, arguments);
        from_method_call_result(env, result)
    }

    #[cfg(test)]
    mod call_constructor_tests {
        use super::*;
        use std::mem;
        use testing::*;

        #[test]
        fn call() {
            const RAW_OBJECT: jni_sys::jobject = 0x54983 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
            static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                class: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jobject;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                class: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jobject;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                class: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jobject {
                assert_eq!(class, RAW_CLASS);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                METHOD_ARGUMENT0 = argument0;
                METHOD_ARGUMENT1 = argument1;
                RAW_OBJECT
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                NewObject: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::FindClass(FindClassCall {
                        name: "java/lang/Object".to_owned(),
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "<init>".to_owned(),
                        signature: "(ID)V".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let arguments = (17 as i32, 19. as f64);
            unsafe {
                let object = super::call_constructor::<Object, _, fn(i32, f64)>(
                    &env,
                    arguments,
                    &NoException::test(),
                ).unwrap();
                calls.assert_eq(&object, RAW_OBJECT);
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, calls.env);
                assert_eq!(METHOD_ARGUMENT0, arguments.0);
                assert_eq!(METHOD_ARGUMENT1, arguments.1);
            }
        }

        #[test]
        fn no_such_class() {
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::FindClass(FindClassCall {
                    name: "java/lang/Object".to_owned(),
                    result: ptr::null_mut(),
                }),
                JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClearCall {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            unsafe {
                let exception =
                    super::call_constructor::<Object, _, fn()>(&env, (), &NoException::test())
                        .unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }

        #[test]
        fn no_such_method() {
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::FindClass(FindClassCall {
                    name: "java/lang/Object".to_owned(),
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodIDCall {
                    class: RAW_CLASS,
                    name: "<init>".to_owned(),
                    signature: "()V".to_owned(),
                    result: ptr::null_mut(),
                }),
                JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClearCall {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            unsafe {
                let exception =
                    super::call_constructor::<Object, _, fn()>(&env, (), &NoException::test())
                        .unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }

        #[test]
        fn exception_thrown() {
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2928475 as jni_sys::jmethodID;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                class: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jobject;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                class: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> jni_sys::jobject;
            unsafe extern "C" fn method(
                _: *mut jni_sys::JNIEnv,
                _: jni_sys::jobject,
                _: jni_sys::jmethodID,
                _: jni_sys::jint,
                _: jni_sys::jdouble,
            ) -> jni_sys::jobject {
                ptr::null_mut()
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                NewObject: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::FindClass(FindClassCall {
                        name: "java/lang/Object".to_owned(),
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodIDCall {
                        class: RAW_CLASS,
                        name: "<init>".to_owned(),
                        signature: "(ID)V".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurredCall { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClearCall {}),
                    JniCall::DeleteLocalRef(DeleteLocalRefCall { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let arguments = (17 as i32, 19. as f64);
            unsafe {
                let exception = super::call_constructor::<Object, _, fn(i32, f64)>(
                    &env,
                    arguments,
                    &NoException::test(),
                ).unwrap_err();
                calls.assert_eq(&exception, EXCEPTION);
            }
        }
    }
}
