use crate::error::JniError;
use crate::token::{ConsumedNoException, NoException};
use crate::version::JniVersion;
use crate::vm::{JavaVM, JavaVMRef};
use core::ptr::NonNull;
use jni_sys;
use std;
use std::cell::RefCell;
use std::mem;

include!("call_jni_method.rs");

/// The interface for interacting with Java.
/// All calls to Java are performed through this interface.
/// JNI methods can only be called from threads, explicitly attached to the Java VM.
/// [`JniEnv`](struct.JniEnv.html) represents such a thread.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#interface-function-table)
///
/// # Examples
/// The best way to obtain a [`&JniEnv`](struct.JniEnv.html) is to attach the current thread with the
/// [`with_attached`](struct.JavaVM.html#method.with_attached) method:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// use rust_jni::*;
/// use std::ptr;
///
/// let init_arguments = InitArguments::default();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm.with_attached(
///     &AttachArguments::new(init_arguments.version()),
///     |env: &JniEnv, token: NoException| {
///         unsafe { env.raw_env() };
///         ((), token)
///     },
/// );
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// The method also provides a [`NoException`](struct.NoException.html) token. See more about exception
/// handling in [`NoException`](struct.NoException.html) documentation.
///
/// If ownership of the [`JniEnv`](struct.JniEnv.html) is required it can be obtained by
/// [`attach`](struct.JavaVM.html#method.attach)-ing the current thread manually:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// # use std::ptr;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// unsafe { env.raw_env() };
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// The attached thread will automatically get detached when [`JniEnv`](struct.JniEnv.html) is
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed.
/// However, detaching the thread in presence of a pending exception is not safe. Always prefer calling
/// [`JniEnv::detach`](struct.JniEnv.html#method.detach) explicitly instead of relying on
/// [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html):
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// # use std::ptr;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let mut env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// let token = env.token();
/// let token = token.consume();
/// let error = env.detach(token);
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// See [`ConsumedNoException`](struct.ConsumedNoException.html) documentation for more details on the syntax.
/// Note that manual [`detach`](struct.JniEnv.html#method.detach)-ing is not required (or possible) when using
/// [`with_attached`](struct.JavaVM.html#method.with_attached).
///
/// [`JniEnv`](struct.JniEnv.html) can't outlive the parent [`JavaVM`](struct.JavaVM.html):
/// ```compile_fail
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::default();
/// let env = {
///     let vm = JavaVM::create(&init_arguments).unwrap();
///     vm
///         .attach(&AttachArguments::new(init_arguments.version()))
///         .unwrap()
/// }; // doesn't compile!
/// ```
/// [`JniEnv`](struct.JniEnv.html) represents a thread attached to the Java VM and thus there
/// can't be two [`JniEnv`](struct.JniEnv.html)-s per thread.
/// [`attach`](struct.JavaVM.html#methods.attach) will panic if you attempt to do so:
/// ```should_panic
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// #
/// # let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap(); // panics!
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {panic!()}
/// ```
/// Note how this error is impossible when using [`with_attached`](struct.JavaVM.html#method.with_attached)
/// to get a [`&JniEnv`](struct.JniEnv.html) since it manages the [`JniEnv`](struct.JniEnv.html) automatically.
///
/// Since [`JniEnv`](struct.JniEnv.html) represents a thread attached to the Java VM, it is
/// [`!Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) which means it can't be passed between threads:
/// ```compile_fail
/// # use rust_jni::*;
/// use std::thread;
///
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// thread::spawn(move || {
///     unsafe { env.raw_env() }; // doesn't compile!
/// });
/// ```
/// It is also [`!Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html) which means it can't be
/// shared between threads:
/// ```compile_fail
/// # use rust_jni::*;
/// use std::thread;
///
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// thread::spawn(|| {
///     unsafe { env.raw_env() }; // doesn't compile!
/// });
/// ```
/// Instead, you need to [`attach`](struct.JavaVM.html#method.attach) each new thread to the VM:
/// ```
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// # use rust_jni::*;
/// # use std::ptr;
/// # use std::thread;
/// use std::sync::Arc;
///
/// # let init_arguments = InitArguments::default();
/// let vm = Arc::new(JavaVM::create(&init_arguments).unwrap());
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// {
///     let vm = vm.clone();
///     thread::spawn(move || {
///         let env = vm
///             .attach(&AttachArguments::new(init_arguments.version()))
///             .unwrap();
///         unsafe { env.raw_env() };
///     });
/// }
/// unsafe { env.raw_env() };
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
/// The thread is automatically detached once the [`JniEnv`](struct.JniEnv.html) is
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed.
// TODO: docs about panicing on detach when there's a pending exception.
#[derive(Debug)]
pub struct JniEnv<'this> {
    vm: &'this JavaVMRef,
    jni_env: NonNull<jni_sys::JNIEnv>,
    pub(crate) has_token: RefCell<bool>,
    // This is just a hack for unit tests that don't actually call JNI.
    // Setting it to `false` allows to not `mem::forget` the value every time.
    #[cfg(test)]
    need_drop: bool,
}

// [`JniEnv`](struct.JniEnv.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'vm> !Send for JniEnv<'vm> {}
// impl<'vm> !Sync for JniEnv<'vm> {}

impl<'this> JniEnv<'this> {
    /// Get the raw Java VM pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    #[inline(always)]
    pub unsafe fn raw_jvm(&self) -> NonNull<jni_sys::JavaVM> {
        self.vm.raw_jvm()
    }

    /// Get the raw JNI environment pointer.
    ///
    /// This function provides low-level access to all of JNI and thus is unsafe.
    #[inline(always)]
    pub unsafe fn raw_env(&self) -> NonNull<jni_sys::JNIEnv> {
        self.jni_env
    }

    fn verify_token_not_borrowed(&self) {
        if !*self.has_token.borrow() {
            self.safe_panic(
                "Trying to obtain a second `NoException` token from the `JniEnv` value.",
            );
        }
    }

    /// Get a [`NoException`](struct.NoException.html) token indicating that there is no pending
    /// exception in this thread.
    ///
    /// Panics when trying to obtain the token for the second time.
    ///
    /// Read more about tokens in [`NoException`](struct.NoException.html) documentation.
    // TODO(#22): Return a token with the env if possible:
    // https://stackoverflow.com/questions/50891977/can-i-return-a-value-and-a-reference-to-it-from-a-function.
    pub fn token<'a>(&'a self) -> NoException<'a> {
        self.verify_token_not_borrowed();

        // Safe because:
        //  - We don't leak the [`Exception`](struct.Exception.html) token.
        //  - We do leak the [`NoException`](struct.NoException.html) token, but this method is the only way
        //    to obtain this token and above we just checked that there's no other token present.
        #[allow(unused_unsafe)]
        unsafe {
            match NoException::check_pending_exception(self) {
                Err(_) => {
                    self.safe_panic(
                        "Trying to obtain a `NoException` token when there is a pending exception.",
                    );
                }
                Ok(token) => {
                    *self.has_token.borrow_mut() = false;
                    token
                }
            }
        }
    }

    /// Get a [`NoException`](struct.NoException.html) token indicating that there is no pending
    /// exception in this thread.
    ///
    /// Like [`NoException`](struct.NoException.html#method.token) but doesn't borrow the
    /// [`JniEnv`](struct.JniEnv.html) mutably. Should only be used internally and with caution.
    ///
    /// Unsafe because the non-mutable borrow means the token can be obtained multiple times.
    pub(crate) unsafe fn token_internal<'a>(&'a self) -> NoException<'a> {
        self.verify_token_not_borrowed();
        *self.has_token.borrow_mut() = false;
        NoException::new(self)
    }

    /// Panic with a message. Since [`JniEnv`](struct.JniEnv.html) panics in
    /// [`Drop::drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop) we need to clear
    /// the possible exception before paincking ourselves.
    fn safe_panic(&self, message: &'static str) -> ! {
        // Describe and clear the exception to not cause panic in drop during panicking situation.
        // Safe because the argument is ensured to be the correct by construction.
        unsafe { call_jni_method!(self, ExceptionDescribe) };
        panic!(message);
    }

    /// Get JNI versoin.
    ///
    /// [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/functions.html#getversion)
    pub fn version(&self) -> JniVersion {
        JniVersion::from_raw(unsafe { call_jni_method!(self, GetVersion) })
    }

    /// Detach current thread.
    ///
    /// Calling this method consumes [`JniEnv`](struct.JniEnv.html). Detaching the thread is not allowed
    /// in presence of a pending exception and this method also consumes the
    /// [`NoException`](struct.NoException.html#method.token) to guarantee correctness.
    ///
    /// See [`ConsumedNoException`](struct.ConsumedNoException.html) documentation for more details.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#detachcurrentthread)
    pub fn detach(self, _token: ConsumedNoException) -> Option<JniError> {
        // Safe because all JNI arguments are correct by construction.
        let result = unsafe { JavaVM::detach(self.raw_jvm()) };
        mem::forget(self);
        result
    }

    pub(crate) unsafe fn native<'vm: 'env, 'env>(
        vm: &'vm JavaVMRef,
        jni_env: NonNull<jni_sys::JNIEnv>,
    ) -> JniEnv<'env> {
        JniEnv {
            vm,
            jni_env,
            has_token: RefCell::new(true),
            #[cfg(test)]
            need_drop: false,
        }
    }

    pub(crate) unsafe fn attached<'vm: 'env, 'env>(
        vm: &'vm JavaVMRef,
        jni_env: NonNull<jni_sys::JNIEnv>,
    ) -> JniEnv<'env> {
        let env = JniEnv {
            vm,
            jni_env,
            has_token: RefCell::new(true),
            #[cfg(test)]
            need_drop: true,
        };
        // Safe because we are not leaking the tokens anywhere.
        #[allow(unused_unsafe)]
        let exception_pending = unsafe { NoException::check_pending_exception(&env).is_err() };
        if exception_pending {
            // Describe and clear the exception to not cause panic in drop during panicking situation.
            // Safe because the argument is ensured to be the correct by construction.
            #[allow(unused_unsafe)]
            unsafe {
                call_jni_method!(env, ExceptionDescribe)
            };
            mem::forget(env);
            panic!("Newly attached thread has a pending exception.");
        }
        env
    }

    #[cfg(test)]
    pub(crate) fn test<'vm>(vm: &'vm JavaVMRef, ptr: *mut jni_sys::JNIEnv) -> JniEnv<'vm> {
        JniEnv {
            vm: &vm,
            // It's fine if the env is null in unit tests as they don't call the actual JNI API.
            jni_env: unsafe { NonNull::new_unchecked(ptr) },
            has_token: RefCell::new(true),
            need_drop: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_default<'vm>(vm: &'vm JavaVMRef) -> JniEnv<'vm> {
        JniEnv::test(vm, 0x1 as *mut jni_sys::JNIEnv)
    }
}

/// [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) detaches the current thread from the Java VM.
/// It's not safe to do so with an exception pending, so it panics if this happens.
///
/// Always prefer to detach the thread using [`JniEnv::detach`](struct.JniEnv.html#method.detach) instead of relying on
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ing the value because it's always safe
/// and also will return an error if it was returned by JNI.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#detachcurrentthread)
impl<'vm> Drop for JniEnv<'vm> {
    fn drop(&mut self) {
        #[cfg(test)]
        {
            if !self.need_drop {
                return;
            }
        }

        // Safe because we are not leaking the tokens anywhere.
        if unsafe { NoException::check_pending_exception(self).is_err() } {
            // We are fine aborting the program here, as this panic means a bug in the code using
            // [`rust-jni`](index.html): [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ing
            // [`JniEnv`](struct.JniEnv.html) in presence of a pending exception is not allowed.
            self.safe_panic(
                "Dropping `JniEnv` with a pending exception is not allowed. Please clear the \
                 exception by unwrapping the exception token before dropping it.",
            );
        }
        // Safe because the current thread is guaranteed to be attached and the argument is correct.
        unsafe {
            let error = JavaVM::detach(self.raw_jvm());
            if error.is_some() {
                // No meaningful way to handle the error except for logging it.
                println!(
                    "Error {:?} when calling `DetachCurrentThread` on {:?}",
                    error.unwrap(),
                    self
                );
            }
        }
    }
}

#[cfg(test)]
mod jni_env_tests {
    use super::*;
    use mockall::*;
    use serial_test::serial;

    generate_java_vm_mock!(mock);
    generate_jni_env_mock!(jni_mock);

    #[test]
    fn raw_jvm() {
        let vm = JavaVMRef::test(0x1234 as *mut jni_sys::JavaVM);
        let env = JniEnv::test_default(&vm);
        unsafe {
            assert_eq!(env.raw_jvm(), vm.raw_jvm());
        }
    }

    #[test]
    fn raw_env() {
        let vm = JavaVMRef::test_default();
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = JniEnv::test(&vm, jni_env);
        unsafe {
            assert_eq!(env.raw_env().as_ptr(), jni_env);
        }
    }

    #[test]
    #[serial]
    fn version() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let get_version_mock = jni_mock::get_version_context();
        get_version_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_VERSION_1_4);
        let vm = JavaVMRef::test_default();
        let env = JniEnv::test(&vm, raw_env_ptr);
        assert_eq!(env.version(), JniVersion::V4);
    }

    #[test]
    #[serial]
    fn detach() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock
            .expect()
            .times(1)
            .withf_st(move |java_vm| *java_vm == raw_java_vm_ptr)
            .return_const(jni_sys::JNI_OK);
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let mut env = JniEnv::test_default(&vm);
        env.need_drop = true; // need to test that drop wasn't called.
        assert_eq!(env.detach(ConsumedNoException), None);
    }

    #[test]
    #[serial]
    fn detach_error() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock.expect().return_const(jni_sys::JNI_ERR);
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let mut env = JniEnv::test_default(&vm);
        env.need_drop = true; // need to test that drop wasn't called.
        assert_eq!(
            env.detach(ConsumedNoException),
            Some(JniError::Unknown(jni_sys::JNI_ERR))
        );
    }

    #[test]
    #[serial]
    fn drop() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock.expect().return_const(jni_sys::JNI_ERR);
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_FALSE);
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        {
            let mut env = JniEnv::test(&vm, raw_env_ptr);
            env.need_drop = true;
        }
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Dropping `JniEnv` with a pending exception is not allowed")]
    fn drop_exception_pending() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock.expect().return_const(jni_sys::JNI_OK);
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let mut sequence = Sequence::new();
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
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let mut env = JniEnv::test(&vm, raw_env_ptr);
        env.need_drop = true;
    }

    #[test]
    #[serial]
    fn drop_detach_error() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock.expect().return_const(jni_sys::JNI_ERR);
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .return_const(jni_sys::JNI_FALSE);
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let mut env = JniEnv::test(&vm, raw_env_ptr);
        env.need_drop = true;
    }

    #[test]
    #[serial]
    fn token() {
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .withf_st(move |env| *env == raw_env_ptr)
            .return_const(jni_sys::JNI_FALSE);
        let raw_java_vm_ptr = 0x1234 as *mut jni_sys::JavaVM;
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let env = JniEnv::test(&vm, raw_env_ptr);
        env.token();
        assert_eq!(env.has_token, RefCell::new(false));
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(expected = "Trying to obtain a second `NoException` token from the `JniEnv`")]
    fn token_twice() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock.expect().return_const(jni_sys::JNI_OK);
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let mut sequence = Sequence::new();
        let exception_check_mock = jni_mock::exception_check_context();
        exception_check_mock
            .expect()
            .times(1)
            .return_const(jni_sys::JNI_FALSE)
            .in_sequence(&mut sequence);
        let exception_describe_mock = jni_mock::exception_describe_context();
        exception_describe_mock
            .expect()
            .times(1)
            .return_const(())
            .in_sequence(&mut sequence);
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let env = JniEnv {
            has_token: RefCell::new(true),
            ..JniEnv::test(&vm, raw_env_ptr)
        };
        env.token();
        env.token();
    }

    #[test]
    #[serial]
    // `serial` messes up compiler lints for other attributes.
    #[allow(unused_attributes)]
    #[should_panic(
        expected = "Trying to obtain a `NoException` token when there is a pending exception"
    )]
    fn token_pending_exception() {
        let raw_java_vm = mock::raw_java_vm();
        let raw_java_vm_ptr = &mut (&raw_java_vm as jni_sys::JavaVM) as *mut jni_sys::JavaVM;
        let detach_thread_mock = mock::detach_thread_context();
        detach_thread_mock.expect().return_const(jni_sys::JNI_OK);
        let raw_env = jni_mock::raw_jni_env();
        let raw_env_ptr = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        let mut sequence = Sequence::new();
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
        let vm = JavaVMRef::test(raw_java_vm_ptr);
        let env = JniEnv::test(&vm, raw_env_ptr);
        env.token();
    }
}
