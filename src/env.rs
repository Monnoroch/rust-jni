use crate::jni_bool;
use crate::token::NoException;
use crate::version::JniVersion;
use crate::vm::JavaVMRef;
use jni_sys;
use std;
use std::cell::RefCell;

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
/// use rust_jni::*;
/// use std::ptr;
///
/// let init_arguments = InitArguments::default();
/// let vm = JavaVM::create(&init_arguments).unwrap();
/// let _ = vm.with_attached(
///     &AttachArguments::new(init_arguments.version()),
///     |env: &JniEnv, token: NoException| {
///         assert_ne!(unsafe { env.raw_env() }, ptr::null_mut());
///         ((), token)
///     },
/// );
/// ```
/// The method also provides a [`NoException`](struct.NoException.html) token. See more about exception
/// handling in [`NoException`](struct.NoException.html) documentation.
///
/// If ownership of the [`JniEnv`](struct.JniEnv.html) is required it can be obtained by
/// [`attach`](struct.JavaVM.html#method.attach)-ing the current thread manually:
/// ```
/// # use rust_jni::*;
/// # use std::ptr;
/// #
/// # let init_arguments = InitArguments::default();
/// # let vm = JavaVM::create(&init_arguments).unwrap();
/// let env = vm
///     .attach(&AttachArguments::new(init_arguments.version()))
///     .unwrap();
/// assert_ne!(unsafe { env.raw_env() }, ptr::null_mut());
/// ```
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
///         assert_ne!(unsafe { env.raw_env() }, ptr::null_mut());
///     });
/// }
/// assert_ne!(unsafe { env.raw_env() }, ptr::null_mut());
/// ```
/// The thread is automatically detached once the [`JniEnv`](struct.JniEnv.html) is dropped.
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
    pub fn token<'a>(&'a self) -> NoException<'a> {
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
            has_token: RefCell::new(false),
            native_method_call: true,
        }
    }

    pub(crate) fn attached<'vm: 'env, 'env>(
        vm: &'vm JavaVMRef,
        jni_env: *mut jni_sys::JNIEnv,
    ) -> JniEnv<'env> {
        let mut env = JniEnv {
            vm,
            jni_env,
            has_token: RefCell::new(true),
            // We don't want to drop `JniEnv` with a pending exception.
            native_method_call: true,
        };
        if env.has_exception() {
            panic!("Newly attached thread has a pending exception.");
        }
        env.native_method_call = false;
        env
    }

    pub(crate) fn has_exception(&self) -> bool {
        // Safe because the argument is ensured to be the correct by construction.
        let value = unsafe { call_jni_method!(self, ExceptionCheck) };
        jni_bool::to_rust(value)
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
        unsafe { JavaVMRef::detach(self.raw_jvm()) };
    }
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
    use crate::vm::test_vm;
    use std::ptr;

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
            assert_eq!(DETACH_ARGUMENT, vm.raw_jvm());
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
