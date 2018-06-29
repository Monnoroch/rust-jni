use jni::*;
use jni_sys;
use std::char;
use std::iter;
#[cfg(test)]
use std::ptr;

/// A macro for generating [`JniType`](trait.JniType.html) implementation for primitive types.
macro_rules! jni_type_trait {
    ($type:ty, $method:ident, $static_method:ident) => {
        impl JniType for $type {
            unsafe fn call_method<In: ToJniTuple>(
                object: &Object,
                method_id: jni_sys::jmethodID,
                arguments: In,
            ) -> Self {
                In::$method(object, method_id, arguments)
            }

            unsafe fn call_static_method<In: ToJniTuple>(
                class: &Class,
                method_id: jni_sys::jmethodID,
                arguments: In,
            ) -> Self {
                In::$static_method(class, method_id, arguments)
            }
        }
    };
}

jni_type_trait!(
    jni_sys::jobject,
    call_object_method,
    call_static_object_method
);
jni_type_trait!((), call_void_method, call_static_void_method);
jni_type_trait!(
    jni_sys::jboolean,
    call_boolean_method,
    call_static_boolean_method
);
jni_type_trait!(jni_sys::jchar, call_char_method, call_static_char_method);
jni_type_trait!(jni_sys::jbyte, call_byte_method, call_static_byte_method);
jni_type_trait!(jni_sys::jshort, call_short_method, call_static_short_method);
jni_type_trait!(jni_sys::jint, call_int_method, call_static_int_method);
jni_type_trait!(jni_sys::jlong, call_long_method, call_static_long_method);
jni_type_trait!(jni_sys::jfloat, call_float_method, call_static_float_method);
jni_type_trait!(
    jni_sys::jdouble,
    call_double_method,
    call_static_double_method
);

macro_rules! generate_jni_type_tests {
    (
        $module:ident,
        $jni_type:ty,
        $default:expr,
        $result:expr,
        $jni_method:ident,
        $jni_static_method:ident
    ) => {
        #[cfg(test)]
        mod $module {
            use super::*;
            use std::mem;
            use testing::*;

            #[test]
            fn call_method() {
                static mut METHOD_CALLS: i32 = 0;
                static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
                static mut METHOD_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
                static mut METHOD_METHOD_ARGUMENT: jni_sys::jmethodID = ptr::null_mut();
                static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
                static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
                static mut METHOD_RESULT: $jni_type = $default;
                type VariadicFn = unsafe extern "C" fn(
                    env: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                    method_id: jni_sys::jmethodID,
                    ...
                ) -> $jni_type;
                type TestFn = unsafe extern "C" fn(
                    env: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                    method_id: jni_sys::jmethodID,
                    argument0: jni_sys::jint,
                    argument1: jni_sys::jdouble,
                ) -> $jni_type;
                unsafe extern "C" fn method(
                    env: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                    method_id: jni_sys::jmethodID,
                    argument0: jni_sys::jint,
                    argument1: jni_sys::jdouble,
                ) -> $jni_type {
                    METHOD_CALLS += 1;
                    METHOD_ENV_ARGUMENT = env;
                    METHOD_OBJECT_ARGUMENT = object;
                    METHOD_METHOD_ARGUMENT = method_id;
                    METHOD_ARGUMENT0 = argument0;
                    METHOD_ARGUMENT1 = argument1;
                    METHOD_RESULT
                }
                let vm = test_vm(ptr::null_mut());
                let raw_jni_env = jni_sys::JNINativeInterface_ {
                    $jni_method: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                    ..empty_raw_jni_env()
                };
                let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
                let env = test_env(&vm, raw_jni_env);
                let raw_object = 0x91011 as jni_sys::jobject;
                let object = test_object(&env, raw_object);
                let method_id = 0x7654 as jni_sys::jmethodID;
                let arguments = (17 as i32, 19. as f64);
                let result = $result;
                unsafe {
                    METHOD_RESULT = result;
                    assert_eq!(
                        <$jni_type>::call_method(&object, method_id, arguments),
                        result
                    );
                    assert_eq!(METHOD_CALLS, 1);
                    assert_eq!(METHOD_ENV_ARGUMENT, raw_jni_env);
                    assert_eq!(METHOD_OBJECT_ARGUMENT, raw_object);
                    assert_eq!(METHOD_METHOD_ARGUMENT, method_id);
                    assert_eq!(METHOD_ARGUMENT0, arguments.0);
                    assert_eq!(METHOD_ARGUMENT1, arguments.1);
                }
            }

            #[test]
            fn call_static_method() {
                static mut METHOD_CALLS: i32 = 0;
                static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
                static mut METHOD_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
                static mut METHOD_METHOD_ARGUMENT: jni_sys::jmethodID = ptr::null_mut();
                static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
                static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
                static mut METHOD_RESULT: $jni_type = $default;
                type VariadicFn = unsafe extern "C" fn(
                    env: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                    method_id: jni_sys::jmethodID,
                    ...
                ) -> $jni_type;
                type TestFn = unsafe extern "C" fn(
                    env: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                    method_id: jni_sys::jmethodID,
                    argument0: jni_sys::jint,
                    argument1: jni_sys::jdouble,
                ) -> $jni_type;
                unsafe extern "C" fn method(
                    env: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                    method_id: jni_sys::jmethodID,
                    argument0: jni_sys::jint,
                    argument1: jni_sys::jdouble,
                ) -> $jni_type {
                    METHOD_CALLS += 1;
                    METHOD_ENV_ARGUMENT = env;
                    METHOD_OBJECT_ARGUMENT = object;
                    METHOD_METHOD_ARGUMENT = method_id;
                    METHOD_ARGUMENT0 = argument0;
                    METHOD_ARGUMENT1 = argument1;
                    METHOD_RESULT
                }
                let vm = test_vm(ptr::null_mut());
                let raw_jni_env = jni_sys::JNINativeInterface_ {
                    $jni_static_method: Some(unsafe {
                        mem::transmute::<TestFn, VariadicFn>(method)
                    }),
                    ..empty_raw_jni_env()
                };
                let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
                let env = test_env(&vm, raw_jni_env);
                let raw_object = 0x91011 as jni_sys::jobject;
                let class = test_class(&env, raw_object);
                let method_id = 0x7654 as jni_sys::jmethodID;
                let arguments = (17 as i32, 19. as f64);
                let result = $result;
                unsafe {
                    METHOD_RESULT = result;
                    assert_eq!(
                        <$jni_type>::call_static_method(&class, method_id, arguments),
                        result
                    );
                    assert_eq!(METHOD_CALLS, 1);
                    assert_eq!(METHOD_ENV_ARGUMENT, raw_jni_env);
                    assert_eq!(METHOD_OBJECT_ARGUMENT, raw_object);
                    assert_eq!(METHOD_METHOD_ARGUMENT, method_id);
                    assert_eq!(METHOD_ARGUMENT0, arguments.0);
                    assert_eq!(METHOD_ARGUMENT1, arguments.1);
                }
            }
        }
    };
}

generate_jni_type_tests!(
    jni_type_void_tests,
    (),
    (),
    (),
    CallVoidMethod,
    CallStaticVoidMethod
);

generate_jni_type_tests!(
    jni_type_boolean_tests,
    jni_sys::jboolean,
    jni_sys::JNI_FALSE,
    jni_sys::JNI_TRUE,
    CallBooleanMethod,
    CallStaticBooleanMethod
);

generate_jni_type_tests!(
    jni_type_char_tests,
    jni_sys::jchar,
    0,
    42,
    CallCharMethod,
    CallStaticCharMethod
);

generate_jni_type_tests!(
    jni_type_byte_tests,
    jni_sys::jbyte,
    0,
    42,
    CallByteMethod,
    CallStaticByteMethod
);

generate_jni_type_tests!(
    jni_type_short_tests,
    jni_sys::jshort,
    0,
    42,
    CallShortMethod,
    CallStaticShortMethod
);

generate_jni_type_tests!(
    jni_type_int_tests,
    jni_sys::jint,
    0,
    42,
    CallIntMethod,
    CallStaticIntMethod
);

generate_jni_type_tests!(
    jni_type_long_tests,
    jni_sys::jlong,
    0,
    42,
    CallLongMethod,
    CallStaticLongMethod
);

generate_jni_type_tests!(
    jni_type_float_tests,
    jni_sys::jfloat,
    0.,
    42.,
    CallFloatMethod,
    CallStaticFloatMethod
);

generate_jni_type_tests!(
    jni_type_double_tests,
    jni_sys::jdouble,
    0.,
    42.,
    CallDoubleMethod,
    CallStaticDoubleMethod
);

/// A trait that implements calling JNI variadic functions using a macro to generate
/// it's instances for tuples of different sizes.
/// This is essentially the "[`JniType`](trait.JniType.html) for packed argument tuples".
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
// TODO: reimplement once Rust has variadic functions or variadic templates.
#[doc(hidden)]
pub trait ToJniTuple {
    unsafe fn call_constructor(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jobject;

    unsafe fn call_object_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jobject;

    unsafe fn call_static_object_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jobject;

    unsafe fn call_void_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> ();

    unsafe fn call_static_void_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> ();

    unsafe fn call_boolean_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jboolean;

    unsafe fn call_static_boolean_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jboolean;

    unsafe fn call_char_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jchar;

    unsafe fn call_static_char_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jchar;

    unsafe fn call_byte_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jbyte;

    unsafe fn call_static_byte_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jbyte;

    unsafe fn call_short_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jshort;

    unsafe fn call_static_short_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jshort;

    unsafe fn call_int_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jint;

    unsafe fn call_static_int_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jint;

    unsafe fn call_long_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jlong;

    unsafe fn call_static_long_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jlong;

    unsafe fn call_float_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jfloat;

    unsafe fn call_static_float_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jfloat;

    unsafe fn call_double_method(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jdouble;

    unsafe fn call_static_double_method(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: Self,
    ) -> jni_sys::jdouble;
}

macro_rules! call_jni_method {
    ($env:expr, $method:ident, $($argument:expr),*) => {
        {
            let raw_env = $env.raw_env();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env, $($argument),*)
        }
    };
}

macro_rules! jni_method_call {
    ($name:ident, $type:ty, $method:ident, $return_type:ty, $($argument:ident,)*) => {
        unsafe fn $name(
            object: &$type,
            method_id: jni_sys::jmethodID,
            arguments: Self
        ) -> $return_type {
            #[allow(non_snake_case)]
            let ($($argument,)*) = arguments;
            call_jni_method!(
                object.env(),
                $method,
                object.raw_object(),
                method_id
                $(,ToJni::__to_jni(&$argument))*
            )
        }
    }
}

macro_rules! peel_input_tuple_impls {
    () => ();
    ($type:ident, $jni_type:ident, $($other:ident,)*) => (input_tuple_impls! { $($other,)* });
}

macro_rules! input_tuple_impls {
    ( $($type:ident, $jni_type:ident,)*) => (
        impl<'a, $($type, $jni_type),*> ToJniTuple for ($($type,)*)
        where
            $($type: ToJni<__JniType = $jni_type>,)*
            $($jni_type: JniArgumentType),*
        {
            jni_method_call!(call_constructor, Class, NewObject, jni_sys::jobject, $($type,)*);
            jni_method_call!(call_object_method, Object, CallObjectMethod, jni_sys::jobject, $($type,)*);
            jni_method_call!(call_static_object_method, Class, CallStaticObjectMethod, jni_sys::jobject, $($type,)*);
            jni_method_call!(call_void_method, Object, CallVoidMethod, (), $($type,)*);
            jni_method_call!(call_static_void_method, Class, CallStaticVoidMethod, (), $($type,)*);
            jni_method_call!(call_boolean_method, Object, CallBooleanMethod, jni_sys::jboolean, $($type,)*);
            jni_method_call!(call_static_boolean_method, Class, CallStaticBooleanMethod, jni_sys::jboolean, $($type,)*);
            jni_method_call!(call_char_method, Object, CallCharMethod, jni_sys::jchar, $($type,)*);
            jni_method_call!(call_static_char_method, Class, CallStaticCharMethod, jni_sys::jchar, $($type,)*);
            jni_method_call!(call_byte_method, Object, CallByteMethod, jni_sys::jbyte, $($type,)*);
            jni_method_call!(call_static_byte_method, Class, CallStaticByteMethod, jni_sys::jbyte, $($type,)*);
            jni_method_call!(call_short_method, Object, CallShortMethod, jni_sys::jshort, $($type,)*);
            jni_method_call!(call_static_short_method, Class, CallStaticShortMethod, jni_sys::jshort, $($type,)*);
            jni_method_call!(call_int_method, Object, CallIntMethod, jni_sys::jint, $($type,)*);
            jni_method_call!(call_static_int_method, Class, CallStaticIntMethod, jni_sys::jint, $($type,)*);
            jni_method_call!(call_long_method, Object, CallLongMethod, jni_sys::jlong, $($type,)*);
            jni_method_call!(call_static_long_method, Class, CallStaticLongMethod, jni_sys::jlong, $($type,)*);
            jni_method_call!(call_float_method, Object, CallFloatMethod, jni_sys::jfloat, $($type,)*);
            jni_method_call!(call_static_float_method, Class, CallStaticFloatMethod, jni_sys::jfloat, $($type,)*);
            jni_method_call!(call_double_method, Object, CallDoubleMethod, jni_sys::jdouble, $($type,)*);
            jni_method_call!(call_static_double_method, Class, CallStaticDoubleMethod, jni_sys::jdouble, $($type,)*);
        }
        peel_input_tuple_impls! { $($type, $jni_type,)* }
    );
}

input_tuple_impls! {
    T0, T0Jni,
    T1, T1Jni,
    T2, T2Jni,
    T3, T3Jni,
    T4, T4Jni,
    T5, T5Jni,
    T6, T6Jni,
    T7, T7Jni,
    T8, T8Jni,
    T9, T9Jni,
    T10, T10Jni,
    T11, T11Jni,
}

#[cfg(test)]
macro_rules! generate_to_jni_tuple_tests {
    (
        $jni_type:ty,
        $default:expr,
        $result:expr,
        $method:ident,
        $jni_method:ident,
        $static_method:ident,
        $jni_static_method:ident
    ) => {
        #[test]
        fn $method() {
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            static mut METHOD_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
            static mut METHOD_METHOD_ARGUMENT: jni_sys::jmethodID = ptr::null_mut();
            static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
            static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
            static mut METHOD_RESULT: $jni_type = $default;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> $jni_type;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> $jni_type;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> $jni_type {
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                METHOD_OBJECT_ARGUMENT = object;
                METHOD_METHOD_ARGUMENT = method_id;
                METHOD_ARGUMENT0 = argument0;
                METHOD_ARGUMENT1 = argument1;
                METHOD_RESULT
            }
            let vm = test_vm(ptr::null_mut());
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                $jni_method: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
            let env = test_env(&vm, raw_jni_env);
            let raw_object = 0x91011 as jni_sys::jobject;
            let object = test_object(&env, raw_object);
            let method_id = 0x7654 as jni_sys::jmethodID;
            let arguments = (17 as i32, 19. as f64);
            let result = $result;
            unsafe {
                METHOD_RESULT = result;
                assert_eq!(ToJniTuple::$method(&object, method_id, arguments), result);
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, raw_jni_env);
                assert_eq!(METHOD_OBJECT_ARGUMENT, raw_object);
                assert_eq!(METHOD_METHOD_ARGUMENT, method_id);
                assert_eq!(METHOD_ARGUMENT0, arguments.0);
                assert_eq!(METHOD_ARGUMENT1, arguments.1);
            }
        }

        #[test]
        fn $static_method() {
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            static mut METHOD_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
            static mut METHOD_METHOD_ARGUMENT: jni_sys::jmethodID = ptr::null_mut();
            static mut METHOD_ARGUMENT0: jni_sys::jint = 0;
            static mut METHOD_ARGUMENT1: jni_sys::jdouble = 0.;
            static mut METHOD_RESULT: $jni_type = $default;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> $jni_type;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> $jni_type;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                argument0: jni_sys::jint,
                argument1: jni_sys::jdouble,
            ) -> $jni_type {
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                METHOD_OBJECT_ARGUMENT = object;
                METHOD_METHOD_ARGUMENT = method_id;
                METHOD_ARGUMENT0 = argument0;
                METHOD_ARGUMENT1 = argument1;
                METHOD_RESULT
            }
            let vm = test_vm(ptr::null_mut());
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                $jni_static_method: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
            let env = test_env(&vm, raw_jni_env);
            let raw_object = 0x91011 as jni_sys::jobject;
            let class = test_class(&env, raw_object);
            let method_id = 0x7654 as jni_sys::jmethodID;
            let arguments = (17 as i32, 19. as f64);
            let result = $result;
            unsafe {
                METHOD_RESULT = result;
                assert_eq!(
                    ToJniTuple::$static_method(&class, method_id, arguments),
                    result
                );
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, raw_jni_env);
                assert_eq!(METHOD_OBJECT_ARGUMENT, raw_object);
                assert_eq!(METHOD_METHOD_ARGUMENT, method_id);
                assert_eq!(METHOD_ARGUMENT0, arguments.0);
                assert_eq!(METHOD_ARGUMENT1, arguments.1);
            }
        }
    };
}

#[cfg(test)]
mod to_jni_tuple_tests {
    use super::*;
    use std::mem;
    use testing::*;

    generate_to_jni_tuple_tests!(
        jni_sys::jobject,
        ptr::null_mut() as jni_sys::jobject,
        0x1234 as jni_sys::jobject,
        call_object_method,
        CallObjectMethod,
        call_static_object_method,
        CallStaticObjectMethod
    );

    generate_to_jni_tuple_tests!(
        (),
        (),
        (),
        call_void_method,
        CallVoidMethod,
        call_static_void_method,
        CallStaticVoidMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jboolean,
        jni_sys::JNI_FALSE,
        jni_sys::JNI_TRUE,
        call_boolean_method,
        CallBooleanMethod,
        call_static_boolean_method,
        CallStaticBooleanMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jchar,
        0,
        42,
        call_char_method,
        CallCharMethod,
        call_static_char_method,
        CallStaticCharMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jbyte,
        0,
        42,
        call_byte_method,
        CallByteMethod,
        call_static_byte_method,
        CallStaticByteMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jshort,
        0,
        42,
        call_short_method,
        CallShortMethod,
        call_static_short_method,
        CallStaticShortMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jint,
        0,
        42,
        call_int_method,
        CallIntMethod,
        call_static_int_method,
        CallStaticIntMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jlong,
        0,
        42,
        call_long_method,
        CallLongMethod,
        call_static_long_method,
        CallStaticLongMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jfloat,
        0.,
        42.,
        call_float_method,
        CallFloatMethod,
        call_static_float_method,
        CallStaticFloatMethod
    );

    generate_to_jni_tuple_tests!(
        jni_sys::jdouble,
        0.,
        42.,
        call_double_method,
        CallDoubleMethod,
        call_static_double_method,
        CallStaticDoubleMethod
    );

    #[test]
    fn call_constructor() {
        // TODO(#25): test `f32` as well.
        static mut METHOD_CALLS: i32 = 0;
        static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        static mut METHOD_OBJECT_ARGUMENT: jni_sys::jobject = ptr::null_mut();
        static mut METHOD_METHOD_ARGUMENT: jni_sys::jmethodID = ptr::null_mut();
        static mut METHOD_ARGUMENT0: jni_sys::jboolean = 0;
        static mut METHOD_ARGUMENT1: jni_sys::jchar = 0;
        static mut METHOD_ARGUMENT2: jni_sys::jbyte = 0;
        static mut METHOD_ARGUMENT3: jni_sys::jshort = 0;
        static mut METHOD_ARGUMENT4: jni_sys::jint = 0;
        static mut METHOD_ARGUMENT5: jni_sys::jlong = 0;
        static mut METHOD_ARGUMENT7: jni_sys::jdouble = 0.;
        static mut METHOD_ARGUMENT8: jni_sys::jint = 0;
        static mut METHOD_ARGUMENT9: jni_sys::jint = 0;
        static mut METHOD_ARGUMENT10: jni_sys::jint = 0;
        static mut METHOD_ARGUMENT11: jni_sys::jint = 0;
        static mut METHOD_ARGUMENT12: jni_sys::jint = 0;
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
            argument0: jni_sys::jboolean,
            argument1: jni_sys::jchar,
            argument2: jni_sys::jbyte,
            argument3: jni_sys::jshort,
            argument4: jni_sys::jint,
            argument5: jni_sys::jlong,
            argument7: jni_sys::jdouble,
            argument8: jni_sys::jint,
            argument9: jni_sys::jint,
            argument10: jni_sys::jint,
            argument11: jni_sys::jint,
            argument12: jni_sys::jint,
        ) -> jni_sys::jobject;
        unsafe extern "C" fn method(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            argument0: jni_sys::jboolean,
            argument1: jni_sys::jchar,
            argument2: jni_sys::jbyte,
            argument3: jni_sys::jshort,
            argument4: jni_sys::jint,
            argument5: jni_sys::jlong,
            argument7: jni_sys::jdouble,
            argument8: jni_sys::jint,
            argument9: jni_sys::jint,
            argument10: jni_sys::jint,
            argument11: jni_sys::jint,
            argument12: jni_sys::jint,
        ) -> jni_sys::jobject {
            METHOD_CALLS += 1;
            METHOD_ENV_ARGUMENT = env;
            METHOD_OBJECT_ARGUMENT = object;
            METHOD_METHOD_ARGUMENT = method_id;
            METHOD_ARGUMENT0 = argument0;
            METHOD_ARGUMENT1 = argument1;
            METHOD_ARGUMENT2 = argument2;
            METHOD_ARGUMENT3 = argument3;
            METHOD_ARGUMENT4 = argument4;
            METHOD_ARGUMENT5 = argument5;
            METHOD_ARGUMENT7 = argument7;
            METHOD_ARGUMENT8 = argument8;
            METHOD_ARGUMENT9 = argument9;
            METHOD_ARGUMENT10 = argument10;
            METHOD_ARGUMENT11 = argument11;
            METHOD_ARGUMENT12 = argument12;
            METHOD_RESULT
        }
        let vm = test_vm(ptr::null_mut());
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            NewObject: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ..empty_raw_jni_env()
        };
        let raw_jni_env = &mut (&raw_jni_env as jni_sys::JNIEnv) as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, raw_jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let class = test_class(&env, raw_object);
        let method_id = 0x7654 as jni_sys::jmethodID;
        let arguments = (
            true, 'h', 15 as u8, 16 as i16, 17 as i32, 18 as i64, 20. as f64, 21 as i32, 22 as i32,
            23 as i32, 24 as i32, 25 as i32,
        );
        let result = 0x1234 as jni_sys::jobject;
        unsafe {
            METHOD_RESULT = result;
            assert_eq!(
                ToJniTuple::call_constructor(&class, method_id, arguments),
                result
            );
            assert_eq!(METHOD_CALLS, 1);
            assert_eq!(METHOD_ENV_ARGUMENT, raw_jni_env);
            assert_eq!(METHOD_OBJECT_ARGUMENT, raw_object);
            assert_eq!(METHOD_METHOD_ARGUMENT, method_id);
            assert_eq!(METHOD_ARGUMENT0, arguments.0.__to_jni());
            assert_eq!(METHOD_ARGUMENT1, arguments.1.__to_jni());
            assert_eq!(METHOD_ARGUMENT2, arguments.2.__to_jni());
            assert_eq!(METHOD_ARGUMENT3, arguments.3.__to_jni());
            assert_eq!(METHOD_ARGUMENT4, arguments.4.__to_jni());
            assert_eq!(METHOD_ARGUMENT5, arguments.5.__to_jni());
            assert_eq!(METHOD_ARGUMENT7, arguments.6.__to_jni());
            assert_eq!(METHOD_ARGUMENT8, arguments.7.__to_jni());
            assert_eq!(METHOD_ARGUMENT9, arguments.8.__to_jni());
            assert_eq!(METHOD_ARGUMENT10, arguments.9.__to_jni());
            assert_eq!(METHOD_ARGUMENT11, arguments.10.__to_jni());
            assert_eq!(METHOD_ARGUMENT12, arguments.11.__to_jni());
        }
    }
}

impl JniArgumentType for jni_sys::jboolean {}
impl JniArgumentType for jni_sys::jchar {}
impl JniArgumentType for jni_sys::jbyte {}
impl JniArgumentType for jni_sys::jshort {}
impl JniArgumentType for jni_sys::jint {}
impl JniArgumentType for jni_sys::jlong {}
impl JniArgumentType for jni_sys::jfloat {}
impl JniArgumentType for jni_sys::jdouble {}
impl JniArgumentType for jni_sys::jobject {}

/// Make [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) mappable to
/// [`jboolean`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jboolean.html).
impl JavaType for bool {
    #[doc(hidden)]
    type __JniType = jni_sys::jboolean;

    #[doc(hidden)]
    fn __signature() -> &'static str {
        "Z"
    }
}

/// Make [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) convertible to
/// [`jboolean`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jboolean.html).
impl ToJni for bool {
    unsafe fn __to_jni(&self) -> Self::__JniType {
        match self {
            true => jni_sys::JNI_TRUE,
            false => jni_sys::JNI_FALSE,
        }
    }
}

/// Make [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) convertible from
/// [`jboolean`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jboolean.html).
impl<'env> FromJni<'env> for bool {
    unsafe fn __from_jni(_: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
        match value {
            jni_sys::JNI_TRUE => true,
            jni_sys::JNI_FALSE => false,
            value => panic!("Unexpected jboolean value {:?}", value),
        }
    }
}

#[cfg(test)]
mod bool_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(bool::__signature(), "Z");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(true.__to_jni(), jni_sys::JNI_TRUE);
            assert_eq!(false.__to_jni(), jni_sys::JNI_FALSE);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(bool::__from_jni(&env, jni_sys::JNI_TRUE), true);
            assert_eq!(bool::__from_jni(&env, jni_sys::JNI_FALSE), false);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(bool::__from_jni(&env, true.__to_jni()), true);
            assert_eq!(bool::__from_jni(&env, false.__to_jni()), false);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(
                bool::__from_jni(&env, jni_sys::JNI_TRUE).__to_jni(),
                jni_sys::JNI_TRUE
            );
            assert_eq!(
                bool::__from_jni(&env, jni_sys::JNI_FALSE).__to_jni(),
                jni_sys::JNI_FALSE
            );
        }
    }
}

/// Make [`char`](https://doc.rust-lang.org/std/primitive.char.html) mappable to
/// [`jchar`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html).
impl JavaType for char {
    #[doc(hidden)]
    type __JniType = jni_sys::jchar;

    #[doc(hidden)]
    fn __signature() -> &'static str {
        "C"
    }
}

/// Make [`char`](https://doc.rust-lang.org/std/primitive.char.html) convertible to
/// [`jchar`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html).
#[doc(hidden)]
impl ToJni for char {
    unsafe fn __to_jni(&self) -> Self::__JniType {
        *self as Self::__JniType
    }
}

/// Make [`char`](https://doc.rust-lang.org/std/primitive.char.html) convertible from
/// [`jchar`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html).
#[doc(hidden)]
impl<'env> FromJni<'env> for char {
    unsafe fn __from_jni(_: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
        let mut decoder = char::decode_utf16(iter::once(value));
        // A character returned from Java is guaranteed to be a valid UTF-16 code point.
        let character = decoder.next().unwrap().unwrap();
        match decoder.next() {
            None => {}
            Some(second) => {
                panic!(
                    "Java character {:?} was mapped to more than one Rust characters: \
                     [{:?}, {:?}, ...].",
                    value, character, second,
                );
            }
        }
        character
    }
}

#[cfg(test)]
mod char_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(char::__signature(), "C");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!('h'.__to_jni(), 'h' as jni_sys::jchar);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(char::__from_jni(&env, 'h' as jni_sys::jchar), 'h');
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(char::__from_jni(&env, 'h'.__to_jni()), 'h');
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(
                char::__from_jni(&env, 'h' as jni_sys::jchar).__to_jni(),
                'h' as jni_sys::jchar
            );
        }
    }
}

/// A macro for generating [`JavaType`](trait.JavaType.html) implementations for most primitive
/// Rust types.
macro_rules! jni_io_traits {
    ($type:ty, $jni_type:ty, $signature:expr, $link:expr, $jni_sys_link:expr) => {
        /// Make
        #[doc = $link]
        /// mappable to
        #[doc = $jni_sys_link]
        ///.
        impl JavaType for $type {
            #[doc(hidden)]
            type __JniType = $jni_type;

            #[doc(hidden)]
            fn __signature() -> &'static str {
                $signature
            }
        }

        /// Make
        #[doc = $link]
        /// convertible to
        #[doc = $jni_sys_link]
        ///.
        #[doc(hidden)]
        impl ToJni for $type {
            unsafe fn __to_jni(&self) -> Self::__JniType {
                *self as Self::__JniType
            }
        }

        /// Make
        #[doc = $link]
        /// convertible from
        #[doc = $jni_sys_link]
        ///.
        #[doc(hidden)]
        impl<'env> FromJni<'env> for $type {
            unsafe fn __from_jni(_: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
                value as Self
            }
        }
    };
}

jni_io_traits!(
    (),
    (),
    "V",
    "[`()`](https://doc.rust-lang.org/std/primitive.unit.html)",
    "[`()`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html)"
);
jni_io_traits!(
    u8,
    jni_sys::jbyte,
    "B",
    "[`u8`](https://doc.rust-lang.org/std/primitive.u8.html)",
    "[`jbyte`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jbyte.html)"
);
jni_io_traits!(
    i16,
    jni_sys::jshort,
    "S",
    "[`i16`](https://doc.rust-lang.org/std/primitive.i16.html)",
    "[`jshort`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jshort.html)"
);
jni_io_traits!(
    i32,
    jni_sys::jint,
    "I",
    "[`i32`](https://doc.rust-lang.org/std/primitive.i32.html)",
    "[`jint`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jint.html)"
);
jni_io_traits!(
    i64,
    jni_sys::jlong,
    "J",
    "[`i64`](https://doc.rust-lang.org/std/primitive.i64.html)",
    "[`jlong`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jlong.html)"
);
// For some reason, floats need to be passed as 64-bit floats to JNI.
// When passed as 32-bit numbers, Java recieves `0.0` instead of the passed number.
// This also causes `__JniType` to not reside in `JavaType`, as this is the
// only exceptional case.
// TODO(#25): figure out the underlying cause of this.
// native call -> java: f64
// java -> native call: f32
// java -> native method: f64
// native method -> java: f64
// jni_io_traits!(
//     f32,
//     jni_sys::jfloat,
//     "F",
//     "[`f32`](https://doc.rust-lang.org/std/primitive.f32.html)",
//     "[`jfloat`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jfloat.html)"
// );
jni_io_traits!(
    f64,
    jni_sys::jdouble,
    "D",
    "[`f64`](https://doc.rust-lang.org/std/primitive.f64.html)",
    "[`jdouble`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jdouble.html)"
);

#[cfg(test)]
mod void_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(<()>::__signature(), "V");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(().__to_jni(), ());
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(<()>::__from_jni(&env, ()), ());
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(<()>::__from_jni(&env, ().__to_jni()), ());
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(<()>::__from_jni(&env, ()).__to_jni(), ());
        }
    }
}

#[cfg(test)]
mod byte_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(u8::__signature(), "B");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(217.__to_jni(), 217);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(u8::__from_jni(&env, 217 as u8 as i8), 217);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(u8::__from_jni(&env, (217 as u8).__to_jni()), 217);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(
                u8::__from_jni(&env, 217 as u8 as i8).__to_jni(),
                217 as u8 as i8
            );
        }
    }
}

#[cfg(test)]
mod short_tests {

    #[test]
    fn signature() {
        assert_eq!(i16::__signature(), "S");
    }
    use super::*;

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(217.__to_jni(), 217);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i16::__from_jni(&env, 217), 217);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i16::__from_jni(&env, (217 as i16).__to_jni()), 217);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i16::__from_jni(&env, 217).__to_jni(), 217);
        }
    }
}

#[cfg(test)]
mod int_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(i32::__signature(), "I");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(217.__to_jni(), 217);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i32::__from_jni(&env, 217), 217);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i32::__from_jni(&env, (217 as i32).__to_jni()), 217);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i32::__from_jni(&env, 217).__to_jni(), 217);
        }
    }
}

#[cfg(test)]
mod long_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(i64::__signature(), "J");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(217.__to_jni(), 217);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i64::__from_jni(&env, 217), 217);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i64::__from_jni(&env, (217 as i64).__to_jni()), 217);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(i64::__from_jni(&env, 217).__to_jni(), 217);
        }
    }
}

#[cfg(test)]
mod double_tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(f64::__signature(), "D");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!((217.).__to_jni(), 217.);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(f64::__from_jni(&env, 217.), 217.);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(f64::__from_jni(&env, (217. as f64).__to_jni()), 217.);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(f64::__from_jni(&env, 217.).__to_jni(), 217.);
        }
    }
}
