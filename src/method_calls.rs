use crate::class::Class;
use crate::java_string::*;
use crate::object::Object;
use crate::primitives::ToJniTuple;
use crate::result::JavaResult;
use crate::token::{from_nullable, get_and_clear_exception_if_thrown, NoException};
use crate::vm::*;
use jni_sys;
use std::os::raw::c_char;

include!("call_jni_method.rs");

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
        token,
        GetMethodID,
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
        token,
        GetStaticMethodID,
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
    match get_and_clear_exception_if_thrown(env) {
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
    use crate::object::test_object;
    use crate::testing::*;
    use std::mem;
    use std::ptr;

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
                JniCall::GetObjectClass(GetObjectClass {
                    object: RAW_OBJECT,
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "test-method".to_owned(),
                    signature: "(ID)Z".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred {
                    result: ptr::null_mut(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
            ],
            raw_jni_env
        );
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_object(&env, RAW_OBJECT);
        let arguments = (17 as i32, 19. as f64);
        unsafe {
            assert!(super::call_method::<Object, _, _, fn(i32, f64) -> bool>(
                &object,
                "test-method",
                arguments,
                &NoException::test()
            )
            .unwrap());
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
            JniCall::GetObjectClass(GetObjectClass {
                object: RAW_OBJECT,
                result: RAW_CLASS,
            }),
            JniCall::GetMethodID(GetMethodID {
                class: RAW_CLASS,
                name: "test-method".to_owned(),
                signature: "()Z".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
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
            )
            .unwrap_err();
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
                JniCall::GetObjectClass(GetObjectClass {
                    object: RAW_OBJECT,
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "test-method".to_owned(),
                    signature: "()Z".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
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
            )
            .unwrap_err();
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
    use crate::testing::*;
    use std::mem;
    use std::ptr;

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
            CallStaticBooleanMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ..empty_raw_jni_env()
        };
        let calls = test_raw_jni_env!(
            vec![
                JniCall::FindClass(FindClass {
                    name: "java/lang/Object".to_owned(),
                    result: RAW_CLASS,
                }),
                JniCall::GetStaticMethodID(GetStaticMethodID {
                    class: RAW_CLASS,
                    name: "test-method".to_owned(),
                    signature: "(ID)Z".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred {
                    result: ptr::null_mut(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
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
                )
                .unwrap()
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
            JniCall::FindClass(FindClass {
                name: "java/lang/Object".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        unsafe {
            let exception = super::call_static_method::<Object, _, _, fn() -> bool>(
                &env,
                "",
                (),
                &NoException::test(),
            )
            .unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }
    }

    #[test]
    fn no_such_method() {
        const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::FindClass(FindClass {
                name: "java/lang/Object".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::GetStaticMethodID(GetStaticMethodID {
                class: RAW_CLASS,
                name: "test-method".to_owned(),
                signature: "()Z".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        unsafe {
            let exception = super::call_static_method::<Object, _, _, fn() -> bool>(
                &env,
                "test-method",
                (),
                &NoException::test(),
            )
            .unwrap_err();
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
            CallStaticBooleanMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ..empty_raw_jni_env()
        };
        let calls = test_raw_jni_env!(
            vec![
                JniCall::FindClass(FindClass {
                    name: "java/lang/Object".to_owned(),
                    result: RAW_CLASS,
                }),
                JniCall::GetStaticMethodID(GetStaticMethodID {
                    class: RAW_CLASS,
                    name: "test-method".to_owned(),
                    signature: "()Z".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
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
            )
            .unwrap_err();
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
    use crate::testing::*;
    use std::mem;
    use std::ptr;

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
                JniCall::FindClass(FindClass {
                    name: "java/lang/Object".to_owned(),
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "<init>".to_owned(),
                    signature: "(ID)V".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred {
                    result: ptr::null_mut(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
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
            )
            .unwrap();
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
            JniCall::FindClass(FindClass {
                name: "java/lang/Object".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
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
            JniCall::FindClass(FindClass {
                name: "java/lang/Object".to_owned(),
                result: RAW_CLASS,
            }),
            JniCall::GetMethodID(GetMethodID {
                class: RAW_CLASS,
                name: "<init>".to_owned(),
                signature: "()V".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
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
                JniCall::FindClass(FindClass {
                    name: "java/lang/Object".to_owned(),
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "<init>".to_owned(),
                    signature: "(ID)V".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
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
            )
            .unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }
    }
}
