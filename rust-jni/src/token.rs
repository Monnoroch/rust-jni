use crate::env::JniEnv;
use crate::jni_bool;
use crate::result::JavaResult;
use crate::throwable::Throwable;
use std::mem;
use std::ptr::NonNull;

include!("call_jni_method.rs");

/// A token that represents that there is no pending Java exception in the current thread.
///
/// # Pending exceptions
///
/// When a JNI function is called it can throw an exception. Then the current thread is said
/// to have a pending exception. Most JNI functions must not be called when there is a pending exception.
/// Read more about exception handling in
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/design.html#java-exceptions).
///
/// # Exception tokens
///
/// [`rust-jni`](index.html) tries to push as many programming errors as possible from run-time
/// to compile-time. To not allow a caller to call JNI methods when there is a pending exception,
/// these methods will require the caller to provide a [`NoException`](struct.NoException.html)
/// token. Functions that can be called when there's a pending exception don't need this token to be called.
/// Functions that can't be called when there's a pending exception but can't throw exceptions themselves
/// borrow the token. Functions that throw an exception consume the [`NoException`](struct.NoException.html) token
/// and return a new [`Exception`](struct.Exception.html) token. Functions that may throw an exception borrow
/// the [`NoException`](struct.NoException.html) token and return a [`JavaResult`](type.JavaResult.html) with either
/// the returned value or the thrown exception in a [`Throwable`](java/lang/struct.Throwable.html) value.
///
/// The best way to get the token is to attach the current thread with the
/// [`with_attached`](struct.JavaVM.html#method.with_attached) method:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// use rust_jni::*;
///
/// let init_arguments = InitArguments::default();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm.with_attached(
///     &AttachArguments::new(init_arguments.version()),
///     |token: NoException| ((), token),
/// );
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// Note how the token needs to be returned, this ensures that there are no pending exceptions
/// when the thread is detached after the user code is done executing.
///
/// Once obtained, the token can be used to call JNI methods:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let empty_string_length = vm
///     .with_attached(
///         &AttachArguments::new(init_arguments.version()),
///         |token| {
///             let string = java::lang::String::empty(&token).unwrap();
///             (string.len(&token), token)
///         },
///     )
///     .unwrap();
/// assert_eq!(empty_string_length, 0);
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// The caller also can obtain the token after [`attach`](struct.JavaVM.html#method.attach)-ing
/// the thread to the Java VM manually:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let mut env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// let token = env.token();
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// When using this method a token also can not be obtained twice from a [`JniEnv`](struct.JniEnv.html) value.
/// [`JniEnv`](struct.JniEnv.html) panics on subsequent [`JniEnv::token`](struct.JniEnv.html#method.token) calls:
/// ```should_panic
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let mut env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// let token = env.token();
/// let token = env.token(); // panics!
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {panic!()}
/// ```
/// Note how this is a runtime error. Using the [`with_attached`](struct.JavaVM.html#method.with_attached) method
/// and never getting the token manually will completely prevent any runtime errors and therefore is the preferred
/// way to use the library. Manual [`attach`](struct.JavaVM.html#method.attach) is an escape hatch in case you need
/// ownership.
///
/// [`rust-jni`](index.html) follows Java semantics where a method either returns a result
/// or throws an exception. All Java methods return a [`JavaResult`](type.JavaResult.html) value,
/// which is either an actual result or a [`Throwable`](java/lang/struct.Throwable.html) value representing
/// the exception thrown by this method call. Java methods never leave a pending exception,
/// so they never consume the [`NoException`](struct.NoException.html) token, but they always
/// require it to be present:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm
///     .with_attached(
///         &AttachArguments::new(init_arguments.version()),
///         |token| {
///             let string = java::lang::Class::find(&token, "java/lang/String").unwrap();
///             let exception = java::lang::Class::find(&token, "invalid").unwrap_err();
///             ((), token)
///         },
///     );
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// The token is bound to the [`JniEnv`](struct.JniEnv.html) object, so it can't outlive it:
/// ```compile_fail
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let token = {
///     let mut env = vm
///         .attach(&AttachArguments::new(init_arguments.version()))
///         .unwrap();
///     let token = env.token();
///     token
/// }; // doesn't compile!
/// ```
/// Some JNI methods throw exceptions themselves. In this case the token will be consumed
/// so that there is no possible way to obtain a token when there is a pending exception:
/// ```compile_fail
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm
///     .with_attached(
///         &AttachArguments::new(init_arguments.version()),
///         |token| {
///             let exception = java::lang::Class::find(&token, "invalid").unwrap_err();
///             exception.throw(token);
///             // Doesn't compile! Can't use the token any more.
///             let _ = java::lang::String::empty(&token);
///             ((), token)
///         },
///     );
/// ```
/// Methods that consume the token will always return an [`Exception`](struct.Exception.html)
/// token. The [`Exception`](struct.Exception.html) token can be
/// [`unwrap`](struct.Exception.html#method.unwrap)-ped back into a
/// [`NoException`](struct.NoException.html) token and a [`Throwable`](java/lang/struct.Throwable.html)
/// value with the pending exception. Unwrapping the [`Exception`](struct.Exception.html) token
/// will clear the pending exception, so it is again safe to call JNI methods:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm
///     .with_attached(
///         &AttachArguments::new(init_arguments.version()),
///         |token| {
///             let exception = java::lang::Class::find(&token, "invalid").unwrap_err();
///             let exception_token = exception.throw(token);
///             let (exception, token) = exception_token.unwrap();
///             let _ = java::lang::String::empty(&token); // can call Java methods again.
///             ((), token)
///         },
///     );
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// Since [`NoException`](struct.NoException.html) token represents absence of a pending exception on
/// the current thread, it is [`!Send`](https://doc.rust-lang.org/std/marker/trait.Send.html)
/// and can't be passed between threads:
/// ```compile_fail
/// # use rust_jni::*;
/// use std::thread;
///
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm.with_attached(
///     &AttachArguments::new(init_arguments.version()),
///     |token| {
///         let token = thread::spawn(move || {
///             token // doesn't compile!
///         })
///         .join()
///         .unwrap();
///         ((), token)
///     },
/// );
/// ```
/// For the same reason [`NoException`](struct.NoException.html) token is also
/// [`!Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html) and can't be shared between threads:
/// ```compile_fail
/// # use rust_jni::*;
/// # use std::thread;
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm.with_attached(
///     &AttachArguments::new(init_arguments.version()),
///     |token| {
///         thread::spawn(|| {
///             let _ = &token; // doesn't compile!
///         });
///         ((), token)
///     },
/// );
/// ```
#[derive(Debug)]
pub struct NoException<'this> {
    env: &'this JniEnv<'this>,
}

/// A token that like [`NoException`](struct.NoException.html) represents that there is no
/// pending exception in the current thread, but can't be used to call any JNI api.
///
/// This token existis for the sole purpose of tricking Rust's borrow checker into
/// ensuring [`JniEnv::detach`](struct.JniEnv.html#method.detach) is called with no pending exception.
/// [`JniEnv::detach`](struct.JniEnv.html#method.detach) can't accept a [`NoException`](struct.NoException.html)
/// as it can't outlive it's [`JniEnv`](struct.JniEnv.html) which is being consumed by the
/// [`JniEnv::detach`](struct.JniEnv.html#method.detach) call.
///
/// In short, instead of:
/// ```compile_fail
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let mut env = vm
/// #    .attach(&AttachArguments::new(init_arguments.version()))
/// #    .unwrap();
/// let token = env.token();
/// env.detach(token);
/// ```
/// which doesn't compile, we have to write:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// # let mut env = vm
/// #    .attach(&AttachArguments::new(init_arguments.version()))
/// #    .unwrap();
/// let token = env.token();
/// let token = token.consume();
/// env.detach(token);
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
pub struct ConsumedNoException;

/// A result of a JNI call. Can be either a result ([`CallOutcome::Ok`](enum.CallOutcome.html#variant.Ok))
/// or a pending exception ([`CallOutcome::Err`](enum.CallOutcome.html#variant.Err)) or a result when it is not known
/// if there is a pending exception ([`CallOutcome::Unknown`](enum.CallOutcome.html#variant.Unknown)).
// TODO(https://github.com/rust-lang/cargo/issues/7606): make documentation visible.
pub(crate) enum CallOutcome<'a, T> {
    /// Successfull JNI call. [`NoException`](struct.NoException.html) token is present as a proof of that.
    Ok((T, NoException<'a>)),
    /// JNI call resulted in a pending exception. [`Exception`](struct.Exception.html) token is present
    /// as a proof of that.
    Err(Exception<'a>),
    /// JNI call might have resulted in a pending exception. No tokens are present, need to do a runtime check
    /// to create one of [`NoException`](struct.NoException.html) or [`Exception`](struct.Exception.html) based
    /// on the outcome of the check.
    Unknown(T),
}

impl<'this> NoException<'this> {
    /// Unsafe because it creates a new no-exception token when there might be a pending exception.
    #[inline(always)]
    pub(crate) unsafe fn new<'env>(env: &'env JniEnv<'env>) -> NoException<'env> {
        NoException { env }
    }

    /// Get the reference to the underlying [`JniEnv`](struct.JniEnv.html).
    #[inline(always)]
    pub fn env(&self) -> &'this JniEnv<'this> {
        self.env
    }

    /// Consume the [`NoException`](struct.NoException.html) token. After the token is consumed
    /// no JNI API can be called. The result can be passed to [`JniEnv::detach`](struct.JniEnv.html#method.detach).
    #[cold]
    #[inline]
    pub fn consume(self) -> ConsumedNoException {
        ConsumedNoException
    }

    /// Exchange a [`NoException`](struct.NoException.html) for an
    /// [`Exception`](struct.Exception.html) token. This means that [`rust-jni`](index.html)
    /// no longer can prove that there is no pending exception.
    /// Unsafe because there might not actually be a pending exception when this method is called.
    #[cold]
    #[inline(always)]
    pub(crate) unsafe fn exchange(self) -> Exception<'this> {
        Exception::new(self.env)
    }

    /// Execute code that can throw an exception without giving up the ownership of the
    /// [`NoException`](struct.NoException.html) token.
    ///
    /// This is the primary way [`rust-jni`](index.html) can accept the [`NoException`](struct.NoException.html)
    /// token by reference even when calling Java methods that can throw.
    /// [`with_owned`](struct.NoException.html#method.with_owned) temporarily creates a second
    /// [`NoException`](struct.NoException.html) token. It's safe to do, as
    /// [`with_owned`](struct.NoException.html#method.with_owned) borrows the existing token
    /// which means that there's no pending exception and it can create as many new tokens as it wishes without
    /// violating the guarantee that the token can only live while there's no pending exception. Once
    /// [`with_owned`](struct.NoException.html#method.with_owned) owns a [`NoException`](struct.NoException.html)
    /// token, it can call the provided callback with it. When the callback returns a
    /// [`CallOutcome`](enum.CallOutcome.html) [`with_owned`](struct.NoException.html#method.with_owned) performs
    /// the appropriate cleanup:
    ///   - If the callback returned successfully, drop the second [`NoException`](struct.NoException.html) token
    ///     it created and pass the callback result to the caller
    ///   - If the callback returned with an [`Exception`](struct.Exception.html) token,
    ///     [`unwrap`](struct.Exception.html#method.unwrap) it into a [`Throwable`](java/lang/struct.Throwable.html),
    ///     drop the additional [`NoException`](struct.NoException.html) and return the
    ///     [`Throwable`](java/lang/struct.Throwable.html) to the caller.
    ///   - If the callback doesn't know if there is a pending exception or not,
    ///     [`with_owned`](struct.NoException.html#method.with_owned) performs the runtime check by calling required
    ///     JNI methods. If there was a pending exception, it is cleared and a
    ///     [`Throwable`](java/lang/struct.Throwable.html) is returned to the caller. If there wasn't a pending
    ///     exception, the callback result is passed to the caller.
    ///
    /// All this relies on the callback's ability to return the correct result. It's always safe to return a
    /// [`CallOutcome::Unknown`](enum.CallOutcome.html#variant.Unknown) as
    /// [`with_owned`](struct.NoException.html#method.with_owned) does a runtime exception check. However, it
    /// is unsafe to return both [`CallOutcome::Ok`](enum.CallOutcome.html#variant.Ok) and
    /// [`CallOutcome::Err`](enum.CallOutcome.html#variant.Err), as
    /// [`with_owned`](struct.NoException.html#method.with_owned) trusts the callback and doesn't do any additional
    /// runitme checks to maximize performance. The caller needs to be careful to not return them incorrectly.
    ///
    /// This function by itself is safe, but it needs to execute trusted code
    /// that the caller promises is safe.
    // TODO(https://github.com/rust-lang/cargo/issues/7606): make documentation visible.
    pub(crate) fn with_owned<Out, F: FnOnce(Self) -> CallOutcome<'this, Out>>(
        &self,
        function: F,
    ) -> JavaResult<'this, Out> {
        // Safe, because we check for a pending exception after the call
        // and the additional token is dropped.
        let token = unsafe { self.clone() };
        // This is actually an unsafe call as it executes user code that needs to be trusted.
        #[allow(unused_unsafe)]
        let outcome = unsafe { function(token) };
        match outcome {
            CallOutcome::Ok((result, token)) => {
                // Drop the additional token so there's only one live token (borrowed by this method).
                mem::drop(token);
                Ok(result)
            }
            CallOutcome::Err(token) => {
                let (throwable, token) = token.unwrap();
                // Drop the additional token so there's only one live token (borrowed by this method).
                mem::drop(token);
                Err(throwable)
            }
            CallOutcome::Unknown(result) => {
                // Safe because the argument is ensured to be correct references by construction.
                match NonNull::new(unsafe { call_jni_method!(self.env, ExceptionOccurred) }) {
                    None => Ok(result),
                    Some(raw_java_throwable) => {
                        // Safe because the argument is ensured to be correct references by construction.
                        unsafe {
                            call_jni_method!(self.env, ExceptionClear);
                        }
                        // Safe because the arguments are correct.
                        Err(unsafe { Throwable::from_raw(self.env, raw_java_throwable) })
                    }
                }
            }
        }
    }

    /// Perform a runtime check for a pending exception. Return a corresponding exception token:
    /// [`NoException`](struct.NoException.html) when there isn't one or [`Exception`](struct.Exception.html)
    /// when there is.
    ///
    /// Calling this method may violate the guarantee that there's only one instance of a particular token
    /// at each point in time. Thus the caller is supposed to use the resulting token to run different code,
    /// but he shouldn't leak the token anywhere.
    ///
    /// Ideally this function would return a reference to the token, but it is not practical as it would involve
    /// static variables and all the complexity that comes with it.
    ///
    /// This function is unsafe as it relies on the caller to do the right thing: not to leak the token.
    pub(crate) unsafe fn check_pending_exception<'a>(
        env: &'a JniEnv<'a>,
    ) -> Result<NoException<'a>, Exception<'a>> {
        // Safe because the argument is ensured to be the correct by construction.
        #[allow(unused_unsafe)]
        let value = unsafe { call_jni_method!(env, ExceptionCheck) };
        if jni_bool::to_rust(value) {
            Err(Exception::new(env))
        } else {
            Ok(NoException::new(env))
        }
    }

    /// Unsafe, because having two tokens will allow calling methods when there is a
    /// pending exception.
    #[inline(always)]
    unsafe fn clone(&self) -> Self {
        NoException { env: self.env }
    }

    #[cfg(test)]
    pub(crate) fn test<'env>(env: &'env JniEnv<'env>) -> NoException<'env> {
        NoException { env }
    }
}

#[cfg(test)]
mod no_exception_tests {
    use super::*;
    use crate::vm::JavaVMRef;
    use mockall::*;
    use serial_test::serial;
    use std::mem::ManuallyDrop;
    use std::ptr;

    generate_jni_env_mock!(jni_mock);

    #[test]
    fn env() {
        let vm = JavaVMRef::test_default();
        let env = ManuallyDrop::new(JniEnv::test_default(&vm));
        let token = NoException::test(&env);
        unsafe {
            assert_eq!(token.env().raw_env(), env.raw_env());
        }
    }

    #[test]
    #[serial]
    fn with_owned_ok() {
        let vm = JavaVMRef::test_default();
        let env = ManuallyDrop::new(JniEnv::test_default(&vm));
        let token = NoException::test(&env);
        let result = token
            .with_owned(|token| CallOutcome::Ok((12, token)))
            .unwrap();
        assert_eq!(result, 12);
    }

    #[test]
    #[serial]
    fn with_owned_err() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let mut sequence = Sequence::new();
        let exception_occured_mock = jni_mock::exception_occured_context();
        let raw_throwable = 0x2835 as jni_sys::jthrowable;
        exception_occured_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .returning_st(move |_env| raw_throwable)
            .in_sequence(&mut sequence);
        let exception_clear_mock = jni_mock::exception_clear_context();
        exception_clear_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(())
            .in_sequence(&mut sequence);
        let vm = JavaVMRef::test_default();
        let env = ManuallyDrop::new(JniEnv::test(&vm, raw_env_ptr));
        let token = NoException::test(&env);
        let exception = token
            .with_owned::<(), _>(|token| CallOutcome::Err(unsafe { token.exchange() }))
            .unwrap_err();
        assert_eq!(unsafe { exception.raw_object().as_ptr() }, raw_throwable);
        // Prevent unmocked drop.
        mem::forget(exception);
    }

    #[test]
    #[serial]
    fn with_owned_unknown_exception() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let mut sequence = Sequence::new();
        let exception_occured_mock = jni_mock::exception_occured_context();
        let raw_throwable = 0x2835 as jni_sys::jthrowable;
        exception_occured_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .returning_st(move |_env| raw_throwable)
            .in_sequence(&mut sequence);
        let exception_clear_mock = jni_mock::exception_clear_context();
        exception_clear_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(())
            .in_sequence(&mut sequence);
        let vm = JavaVMRef::test_default();
        let env = ManuallyDrop::new(JniEnv::test(&vm, raw_env_ptr));
        let token = NoException::test(&env);
        let exception = token
            .with_owned(|_token| CallOutcome::Unknown(12))
            .unwrap_err();
        assert_eq!(unsafe { exception.raw_object().as_ptr() }, raw_throwable);
        // Prevent unmocked drop.
        mem::forget(exception);
    }

    #[test]
    #[serial]
    fn with_owned_unknown_no_exception() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let exception_occured_mock = jni_mock::exception_occured_context();
        exception_occured_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .returning_st(|_env| ptr::null_mut());
        let vm = JavaVMRef::test_default();
        let env = ManuallyDrop::new(JniEnv::test(&vm, raw_env_ptr));
        let token = NoException::test(&env);
        let result = token.with_owned(|_token| CallOutcome::Unknown(12)).unwrap();
        assert_eq!(result, 12);
    }
}

// [`NoException`](struct.NoException.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for NoException<'env> {}
// impl<'env> !Sync for NoException<'env> {}

/// A dual token to [`NoException`](struct.NoException.html) that represents that there
/// is a pending Java exception in the current thread.
///
/// Read more about exception tokens in [`NoException`](struct.NoException.html) documentation.
#[derive(Debug)]
pub struct Exception<'this> {
    pub(crate) env: &'this JniEnv<'this>,
}

impl<'this> Exception<'this> {
    #[cold]
    #[inline(always)]
    pub(crate) unsafe fn new<'a>(env: &'a JniEnv<'a>) -> Exception<'a> {
        Exception { env }
    }

    /// Get and clear the pending exception and a [`NoException`](struct.NoException.html) token
    /// to call more JNI methods.
    ///
    /// [`Exception`](struct.Exception.html) guarantees that there must be an exception in flight,
    /// thus the method will always return a [`Throwable`](java/lang/struct.Throwable.html).
    ///
    /// The [`Exception`](struct.Exception.html) token is consumed by this method and can't be used any more.
    #[cold]
    pub fn unwrap(self) -> (Throwable<'this>, NoException<'this>) {
        let throwable = {
            // Safe because there are no arguments to be invalid.
            let raw_java_throwable = unsafe { call_jni_method!(self.env, ExceptionOccurred) };
            // Should not fail because [`Exception`](struct.Exception.html) guarantees that
            // there must be an exception in flight.
            let raw_java_throwable = NonNull::new(raw_java_throwable).unwrap();
            // Safe because we construct Throwable from a valid pointer.
            unsafe { Throwable::from_raw(self.env, raw_java_throwable) }
        };
        let token = {
            // Safe because the argument is ensured to be a correct reference by construction.
            unsafe { call_jni_method!(self.env, ExceptionClear) };
            // Safe because we just cleared the exception.
            unsafe { NoException::new(self.env) }
        };
        (throwable, token)
    }

    // Safe because only used for unit-testing.
    #[cfg(test)]
    pub(crate) fn test(env: &'this JniEnv<'this>) -> Self {
        Self { env }
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;
    use crate::vm::JavaVMRef;
    use mockall::*;
    use serial_test::serial;
    use std::mem::ManuallyDrop;

    generate_jni_env_mock!(jni_mock);

    #[test]
    #[serial]
    fn unwrap() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let mut sequence = Sequence::new();
        let exception_occured_mock = jni_mock::exception_occured_context();
        let raw_throwable = 0x2835 as jni_sys::jthrowable;
        exception_occured_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .returning_st(move |_env| raw_throwable)
            .in_sequence(&mut sequence);
        let exception_clear_mock = jni_mock::exception_clear_context();
        exception_clear_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(())
            .in_sequence(&mut sequence);
        let vm = JavaVMRef::test_default();
        let env = ManuallyDrop::new(JniEnv::test(&vm, raw_env_ptr));
        let token = Exception::test(&env);
        let (exception, _) = token.unwrap();
        assert_eq!(unsafe { exception.raw_object().as_ptr() }, raw_throwable);
        // Prevent unmocked drop.
        mem::forget(exception);
    }
}

// [`Exception`](struct.Exception.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for NoException<'env> {}
// impl<'env> !Sync for NoException<'env> {}
