use crate::jni::throwable::Throwable;
use crate::jni::{FromJni, JniEnv};
use crate::result::JavaResult;
use core::marker::PhantomData;
use std::ptr;

include!("jni/call_jni_method.rs");

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
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
/// let token = env.token();
/// ```
/// Once obtained, the token can be used to call JNI methods:
/// ```
/// # use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion, java};
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
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
/// # let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
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
/// let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
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
///     let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
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
/// # let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
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
/// # let env = vm.attach(&AttachArguments::new(init_arguments.version())).unwrap();
/// let token = env.token();
/// let exception = java::lang::Class::find(&env, "invalid", &token).unwrap_err();
/// let exception_token = exception.throw(token); // there is a pending exception now.
/// let (exception, new_token) = exception_token.unwrap();
/// java::lang::String::empty(&env, &new_token); // can call Java methods again.
/// ```
#[derive(Debug)]
pub struct NoException<'env> {
    _env: PhantomData<&'env JniEnv<'env>>,
}

impl<'env> NoException<'env> {
    /// Unsafe because it creates a new no-exception token when there might be a pending exception.
    pub(crate) unsafe fn new_env<'a>(_env: &JniEnv<'a>) -> NoException<'a> {
        // Safe because this function ensures correct lifetimes.
        Self::new_raw()
    }

    /// Unsafe because:
    /// 1. It creates a new no-exception token when there might be a pending exception
    /// 2. Doesn't ensure a correct lifetime
    pub(crate) unsafe fn new_raw<'a>() -> NoException<'a> {
        NoException {
            _env: PhantomData::<&JniEnv>,
        }
    }

    /// Unsafe, because having two tokens will allow calling methods when there is a
    /// pending exception.
    pub(crate) unsafe fn clone(&self) -> Self {
        Self::new_raw()
    }

    #[cfg(test)]
    pub(crate) fn test<'a>() -> NoException<'a> {
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
    pub(crate) env: &'env JniEnv<'env>,
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
    pub(crate) unsafe fn new<'a>(env: &'a JniEnv<'a>, _token: NoException) -> Exception<'a> {
        Self::new_raw(env)
    }

    /// Unsafe because:
    /// 1. Unsafe because there might not actually be a pending exception when this method is
    /// called.
    /// 2. Doesn't ensure a correct lifetime
    pub(crate) unsafe fn new_raw<'a>(env: &'a JniEnv<'a>) -> Exception<'a> {
        Exception { env }
    }

    #[cfg(test)]
    pub(crate) fn test(env: &'env JniEnv<'env>) -> Self {
        // Safe because only used for unit-testing.
        unsafe { Self::new_raw(env) }
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;
    use crate::jni::test_env;
    use crate::jni::test_vm;
    use crate::testing::*;

    #[test]
    fn unwrap() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
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
pub(crate) type JniResult<'env, T> = Result<(T, NoException<'env>), Exception<'env>>;

/// Create a [`JniResult`](type.JniResult.html) from a nullable pointer.
///
/// Will return an [`Exception`](struct.Exception.html) token for the `null` value or the argument
/// and a [`NoException`](struct.NoException.html) token otherwise.
/// Unsafe because there might not be a pending exception.
pub(crate) unsafe fn from_nullable<'a, T>(
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

#[cfg(test)]
mod from_nullable_tests {
    use super::*;
    use crate::jni::test_env;
    use crate::jni::test_vm;

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

/// Get and clear the pending exception.
pub(crate) fn maybe_get_and_clear_exception<'a>(env: &'a JniEnv<'a>) -> Option<Throwable<'a>> {
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
    use crate::jni::test_env;
    use crate::jni::test_vm;
    use crate::testing::*;

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = maybe_get_and_clear_exception(&env).unwrap();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn exception_not_found() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionOccurred(ExceptionOccurred {
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        assert_eq!(maybe_get_and_clear_exception(&env), None);
    }
}

/// Get and clear the pending exception.
pub(crate) fn get_and_clear_exception<'a>(token: Exception<'a>) -> Throwable<'a> {
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
    use crate::jni::test_env;
    use crate::jni::test_vm;
    use crate::testing::*;

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = get_and_clear_exception(Exception::test(&env));
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    #[should_panic(expected = "No pending exception in presence of an Exception token")]
    fn exception_not_found() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionOccurred(ExceptionOccurred {
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        get_and_clear_exception(Exception::test(&env));
    }
}

/// Take a function that produces a [`JniResult`](type.JniResult.html), call it and produce
/// a [`JavaResult`](type.JavaResult.html) from it.
pub(crate) fn with_checked_exception<'a, Out, T: FnOnce(NoException<'a>) -> JniResult<'a, Out>>(
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
    use crate::jni::test_env;
    use crate::jni::test_vm;
    use crate::testing::*;

    #[test]
    fn no_exception() {
        let result =
            with_checked_exception(&NoException::test(), |_| Ok((17, NoException::test())))
                .unwrap();
        assert_eq!(result, 17);
    }

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception =
            with_checked_exception::<i32, _>(&NoException::test(), |_| Err(Exception::test(&env)))
                .unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }
}
