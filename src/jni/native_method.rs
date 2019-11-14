use crate::java_string::*;
use crate::jni::*;
use jni_sys;
use std::cell::RefCell;
use std::panic;
use std::ptr;
use std::string;

/// Unsafe because an incorrect pointer can be passed as an argument.
unsafe fn throw_new_runtime_exception(raw_env: *mut jni_sys::JNIEnv, message: impl AsRef<str>) {
    let message = to_java_string(message.as_ref());
    let class_name = to_java_string("java/lang/RuntimeException");
    let find_class = (**raw_env).FindClass.unwrap();
    let class = find_class(raw_env, class_name.as_ptr() as *const i8);
    if class == ptr::null_mut() {
        panic!(
            "Could not find the java.lang.RuntimeException class on panic, aborting the program."
        );
    } else {
        let throw_new_fn = (**raw_env).ThrowNew.unwrap();
        let error = JniError::from_raw(throw_new_fn(raw_env, class, message.as_ptr() as *const i8));
        if error.is_some() {
            panic!("Could not throw a new runtime exception on panic, status {:?}, aborting the program.", error.unwrap());
        }
    }
}

/// A function to wrap calls to [`rust-jni`](index.html) API from generated native Java methods.
///
/// THIS FUNCTION SHOULD NOT BE CALLED MANUALLY.
///
/// This method should only be used by generated code for native methods and is unsafe
/// because an incorrect pointer can be passed to it as an argument.
#[doc(hidden)]
pub unsafe fn native_method_wrapper<T, R: JniType>(raw_env: *mut jni_sys::JNIEnv, callback: T) -> R
where
    T: for<'a> FnOnce(&'a JniEnv<'a>, NoException<'a>) -> JavaResult<'a, R> + panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| {
        let exception_check = ((**raw_env).ExceptionCheck).unwrap();
        if exception_check(raw_env) != jni_sys::JNI_FALSE {
            panic!("Native method called from a thread with a pending exception.");
        }

        let mut java_vm: *mut jni_sys::JavaVM = ptr::null_mut();
        let get_java_vm_fn = ((**raw_env).GetJavaVM).unwrap();
        let error = JniError::from_raw(get_java_vm_fn(
            raw_env,
            (&mut java_vm) as *mut *mut jni_sys::JavaVM,
        ));
        if error.is_some() {
            panic!(format!(
                "Could not get Java VM. Status: {:?}",
                error.unwrap()
            ));
        }

        // Safe because we pass a correct `java_vm` pointer.
        let vm = JavaVMRef::from_ptr(java_vm);
        let get_version_fn = ((**raw_env).GetVersion).unwrap();
        let env = JniEnv {
            version: JniVersion::from_raw(get_version_fn(raw_env)),
            vm: &vm,
            jni_env: raw_env,
            has_token: RefCell::new(true),
            native_method_call: true,
        };

        // Safe because we checked for a pending exception.
        let token = NoException::new(&env);
        let result = callback(&env, token);
        match result {
            Ok(result) => result,
            Err(exception) => {
                // Safe because we already cleared the pending exception at this point.
                let token = NoException::new(&env);
                exception.throw(token);
                R::default()
            }
        }
    });
    match result {
        Ok(result) => result,
        Err(error) => {
            if let Some(string) = error.downcast_ref::<string::String>() {
                throw_new_runtime_exception(raw_env, format!("Rust panic: {}", string));
            } else if let Some(string) = error.downcast_ref::<&str>() {
                throw_new_runtime_exception(raw_env, format!("Rust panic: {}", string));
            } else {
                throw_new_runtime_exception(raw_env, "Rust panic: generic panic.");
            }
            R::default()
        }
    }
}

#[cfg(test)]
mod native_method_wrapper_tests {
    use super::*;
    use crate::jni::throwable::test_throwable;
    use crate::testing::*;

    #[test]
    fn success() {
        const JAVA_VM: *mut jni_sys::JavaVM = 0x1234 as *mut jni_sys::JavaVM;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: JAVA_VM,
                result: jni_sys::JNI_OK,
            }),
            JniCall::GetVersion(GetVersion {
                result: jni_sys::JNI_VERSION_1_4,
            }),
        ]);
        let result = 10;
        unsafe {
            let actual_result = native_method_wrapper(calls.env, |env, _| {
                assert_eq!(env.raw_env(), calls.env);
                assert_eq!(env.raw_jvm(), JAVA_VM);
                assert_eq!(env.version(), JniVersion::V4);
                Ok(result)
            });
            assert_eq!(actual_result, result);
        }
    }

    #[test]
    fn exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const JAVA_VM: *mut jni_sys::JavaVM = 0x1234 as *mut jni_sys::JavaVM;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: JAVA_VM,
                result: jni_sys::JNI_OK,
            }),
            JniCall::GetVersion(GetVersion {
                result: jni_sys::JNI_VERSION_1_4,
            }),
            JniCall::Throw(Throw {
                object: EXCEPTION,
                result: jni_sys::JNI_OK,
            }),
            JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION }),
        ]);
        unsafe {
            let result: i32 = native_method_wrapper(calls.env, |env, _| {
                assert_eq!(env.raw_env(), calls.env);
                assert_eq!(env.raw_jvm(), JAVA_VM);
                assert_eq!(env.version(), JniVersion::V4);
                Err(test_throwable(env, EXCEPTION))
            });
            assert_eq!(result, <i32 as JniType>::default());
        }
    }

    #[test]
    fn panic() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        const JAVA_VM: *mut jni_sys::JavaVM = 0x1234 as *mut jni_sys::JavaVM;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: JAVA_VM,
                result: jni_sys::JNI_OK,
            }),
            JniCall::GetVersion(GetVersion {
                result: jni_sys::JNI_VERSION_1_4,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: ERROR".to_owned(),
                result: jni_sys::JNI_OK,
            }),
        ]);
        unsafe {
            let actual_result: i32 = native_method_wrapper(calls.env, |env, _| {
                assert_eq!(env.raw_env(), calls.env);
                assert_eq!(env.raw_jvm(), JAVA_VM);
                assert_eq!(env.version(), JniVersion::V4);
                panic!("ERROR");
            });
            assert_eq!(actual_result, <i32 as JniType>::default());
        }
    }

    #[test]
    fn panic_owned() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        const JAVA_VM: *mut jni_sys::JavaVM = 0x1234 as *mut jni_sys::JavaVM;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: JAVA_VM,
                result: jni_sys::JNI_OK,
            }),
            JniCall::GetVersion(GetVersion {
                result: jni_sys::JNI_VERSION_1_4,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: ERROR".to_owned(),
                result: jni_sys::JNI_OK,
            }),
        ]);
        unsafe {
            let actual_result: i32 = native_method_wrapper(calls.env, |env, _| {
                assert_eq!(env.raw_env(), calls.env);
                assert_eq!(env.raw_jvm(), JAVA_VM);
                assert_eq!(env.version(), JniVersion::V4);
                panic!("ERROR".to_owned());
            });
            assert_eq!(actual_result, <i32 as JniType>::default());
        }
    }

    #[test]
    fn non_string_panic() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        const JAVA_VM: *mut jni_sys::JavaVM = 0x1234 as *mut jni_sys::JavaVM;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: JAVA_VM,
                result: jni_sys::JNI_OK,
            }),
            JniCall::GetVersion(GetVersion {
                result: jni_sys::JNI_VERSION_1_4,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: generic panic.".to_owned(),
                result: jni_sys::JNI_OK,
            }),
        ]);
        unsafe {
            let actual_result: i32 = native_method_wrapper(calls.env, |_, _| {
                panic!(123);
            });
            assert_eq!(actual_result, <i32 as JniType>::default());
        }
    }

    #[test]
    fn has_exception() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: Native method called from a thread with a pending exception."
                    .to_owned(),
                result: jni_sys::JNI_OK,
            }),
        ]);
        unsafe {
            let result = native_method_wrapper(calls.env, |_, _| Ok(10));
            assert_eq!(result, <i32 as JniType>::default());
        }
    }

    #[test]
    fn get_java_vm_error() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: ptr::null_mut(),
                result: jni_sys::JNI_ERR,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: Could not get Java VM. Status: Unknown(-1)".to_owned(),
                result: jni_sys::JNI_OK,
            }),
        ]);
        unsafe {
            let result = native_method_wrapper(calls.env, |_, _| Ok(10));
            assert_eq!(result, <i32 as JniType>::default());
        }
    }

    #[test]
    fn throw_failed() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const JAVA_VM: *mut jni_sys::JavaVM = 0x1234 as *mut jni_sys::JavaVM;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_FALSE,
            }),
            JniCall::GetJavaVM(GetJavaVM {
                vm: JAVA_VM,
                result: jni_sys::JNI_OK,
            }),
            JniCall::GetVersion(GetVersion {
                result: jni_sys::JNI_VERSION_1_4,
            }),
            JniCall::Throw(Throw {
                object: EXCEPTION,
                result: jni_sys::JNI_ERR,
            }),
            JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: Throwing an exception has failed with status Unknown(-1)."
                    .to_owned(),
                result: jni_sys::JNI_OK,
            }),
        ]);
        unsafe {
            let result: i32 =
                native_method_wrapper(calls.env, |env, _| Err(test_throwable(env, EXCEPTION)));
            assert_eq!(result, <i32 as JniType>::default());
        }
    }

    #[test]
    #[should_panic(
        expected = "Could not find the java.lang.RuntimeException class on panic, aborting the program"
    )]
    fn find_class_error() {
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: ptr::null_mut(),
            }),
        ]);
        unsafe {
            native_method_wrapper(calls.env, |_, _| Ok(10));
        }
    }

    #[test]
    #[should_panic(
        expected = "Could not throw a new runtime exception on panic, status Unknown(-1), aborting the program"
    )]
    fn throw_new_error() {
        const RAW_CLASS: jni_sys::jobject = 0x209375 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            }),
            JniCall::FindClass(FindClass {
                name: "java/lang/RuntimeException".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::ThrowNew(ThrowNew {
                class: RAW_CLASS,
                message: "Rust panic: Native method called from a thread with a pending exception."
                    .to_owned(),
                result: jni_sys::JNI_ERR,
            }),
        ]);
        unsafe {
            native_method_wrapper(calls.env, |_, _| Ok(10));
        }
    }
}

/// Test that a value implements the [`JniArgumentType`](trait.JniArgumentType.html)
/// in compile-time.
///
/// THIS FUNCTION SHOULD NOT BE CALLED MANUALLY.
///
/// # Examples
/// ```
/// # extern crate rust_jni;
/// # extern crate jni_sys;
/// ::rust_jni::__generator::test_jni_argument_type(0 as ::jni_sys::jint);
/// ```
/// ```compile_fail
/// # extern crate rust_jni;
/// ::rust_jni::__generator::test_jni_argument_type(0 as u64);
/// ```
#[doc(hidden)]
pub fn test_jni_argument_type<T: JniArgumentType>(_value: T) {}

/// Test that a value implements the [`FromJni`](trait.FromJni.html)
/// in compile-time.
///
/// THIS FUNCTION SHOULD NOT BE CALLED MANUALLY.
///
/// # Examples
/// ```
/// # extern crate rust_jni;
/// # extern crate jni_sys;
/// ::rust_jni::__generator::test_from_jni_type(&(0 as i32));
/// ```
/// ```compile_fail
/// # extern crate rust_jni;
/// ::rust_jni::__generator::test_from_jni_type(&(0 as u64));
/// ```
#[doc(hidden)]
pub fn test_from_jni_type<'env, T: FromJni<'env>>(_value: &T) {}
