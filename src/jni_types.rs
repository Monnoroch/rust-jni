use crate::class::Class;
use crate::object::Object;
use crate::traits::ToJni;
use jni_sys;
use std::ptr;

include!("call_jni_method.rs");

pub(crate) mod private {
    use super::*;

    /// A trait that represents a JNI type. It's implemented for all JNI primitive types
    /// and [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
    /// Implements Java method calls and provides the default value for this JNI type.
    ///
    /// THIS TRAIT SHOULD NOT BE USED MANUALLY.
    pub trait JniType {
        fn default() -> Self;

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

    /// A trait that represents primitive JNI types.
    ///
    /// THIS TRAIT SHOULD NOT BE USED MANUALLY.
    pub trait JniPrimitiveType: JniType {
        fn signature() -> &'static str;
    }

    /// A trait that represents JNI types that can be passed as arguments to JNI functions.
    ///
    /// THIS TRAIT SHOULD NOT BE USED MANUALLY.
    pub trait JniArgumentType: JniType {}

    /// A trait that implements calling JNI variadic functions using a macro to generate
    /// it's instances for tuples of different sizes.
    /// This is essentially the "[`JniType`](trait.JniType.html) for packed argument tuples".
    ///
    /// THIS TRAIT SHOULD NOT BE USED MANUALLY.
    // TODO: reimplement once Rust has variadic functions or variadic templates.
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
}

use private::*;

/// A macro for generating [`JniType`](trait.JniType.html) implementation for primitive types.
macro_rules! jni_type_trait {
    ($type:ty, $default:expr, $method:ident, $static_method:ident) => {
        impl JniType for $type {
            fn default() -> Self {
                $default
            }

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
    ptr::null_mut(),
    call_object_method,
    call_static_object_method
);

/// A macro for generating [`JniPrimitiveType`](trait.JniPrimitiveType.html) implementation for primitive types.
macro_rules! jni_primitive_type_trait {
    ($type:ty, $default:expr, $signature:expr, $method:ident, $static_method:ident) => {
        jni_type_trait!($type, $default, $method, $static_method);

        impl JniPrimitiveType for $type {
            fn signature() -> &'static str {
                $signature
            }
        }
    };
}

jni_primitive_type_trait!((), (), "V", call_void_method, call_static_void_method);
jni_primitive_type_trait!(
    jni_sys::jboolean,
    jni_sys::JNI_FALSE,
    "Z",
    call_boolean_method,
    call_static_boolean_method
);
jni_primitive_type_trait!(
    jni_sys::jchar,
    0,
    "C",
    call_char_method,
    call_static_char_method
);
jni_primitive_type_trait!(
    jni_sys::jbyte,
    0,
    "B",
    call_byte_method,
    call_static_byte_method
);
jni_primitive_type_trait!(
    jni_sys::jshort,
    0,
    "S",
    call_short_method,
    call_static_short_method
);
jni_primitive_type_trait!(
    jni_sys::jint,
    0,
    "I",
    call_int_method,
    call_static_int_method
);
jni_primitive_type_trait!(
    jni_sys::jlong,
    0,
    "J",
    call_long_method,
    call_static_long_method
);
jni_primitive_type_trait!(
    jni_sys::jfloat,
    0.,
    "F",
    call_float_method,
    call_static_float_method
);
jni_primitive_type_trait!(
    jni_sys::jdouble,
    0.,
    "D",
    call_double_method,
    call_static_double_method
);

macro_rules! generate_jni_type_test_cases {
    (
        $jni_type:ty,
        $default:expr,
        $result:expr,
        $jni_method:ident,
        $jni_static_method:ident
    ) => {
        #[test]
        fn default() {
            assert_eq!(<$jni_type as JniType>::default(), $default);
        }

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
                    <$jni_type as JniType>::call_method(&object, method_id, arguments),
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
                    <$jni_type as JniType>::call_static_method(&class, method_id, arguments),
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
            use crate::class::test_class;
            use crate::env::test_env;
            use crate::object::test_object;
            use crate::testing::*;
            use crate::vm::test_vm;
            use std::mem;

            generate_jni_type_test_cases!(
                $jni_type,
                $default,
                $result,
                $jni_method,
                $jni_static_method
            );
        }
    };
}

generate_jni_type_tests!(
    jni_type_object_tests,
    jni_sys::jobject,
    ptr::null_mut(),
    0x1234 as jni_sys::jobject,
    CallObjectMethod,
    CallStaticObjectMethod
);

macro_rules! generate_jni_primitive_type_tests {
    (
        $module:ident,
        $jni_type:ty,
        $default:expr,
        $result:expr,
        $signature:expr,
        $jni_method:ident,
        $jni_static_method:ident
    ) => {
        #[cfg(test)]
        mod $module {
            use super::*;
            use crate::class::test_class;
            use crate::env::test_env;
            use crate::object::test_object;
            use crate::testing::*;
            use crate::vm::test_vm;
            use std::mem;

            generate_jni_type_test_cases!(
                $jni_type,
                $default,
                $result,
                $jni_method,
                $jni_static_method
            );

            #[test]
            fn signature() {
                assert_eq!(<$jni_type as JniPrimitiveType>::signature(), $signature);
            }
        }
    };
}

generate_jni_primitive_type_tests!(
    jni_type_void_tests,
    (),
    (),
    (),
    "V",
    CallVoidMethod,
    CallStaticVoidMethod
);

generate_jni_primitive_type_tests!(
    jni_type_boolean_tests,
    jni_sys::jboolean,
    jni_sys::JNI_FALSE,
    jni_sys::JNI_TRUE,
    "Z",
    CallBooleanMethod,
    CallStaticBooleanMethod
);

generate_jni_primitive_type_tests!(
    jni_type_char_tests,
    jni_sys::jchar,
    0,
    42,
    "C",
    CallCharMethod,
    CallStaticCharMethod
);

generate_jni_primitive_type_tests!(
    jni_type_byte_tests,
    jni_sys::jbyte,
    0,
    42,
    "B",
    CallByteMethod,
    CallStaticByteMethod
);

generate_jni_primitive_type_tests!(
    jni_type_short_tests,
    jni_sys::jshort,
    0,
    42,
    "S",
    CallShortMethod,
    CallStaticShortMethod
);

generate_jni_primitive_type_tests!(
    jni_type_int_tests,
    jni_sys::jint,
    0,
    42,
    "I",
    CallIntMethod,
    CallStaticIntMethod
);

generate_jni_primitive_type_tests!(
    jni_type_long_tests,
    jni_sys::jlong,
    0,
    42,
    "J",
    CallLongMethod,
    CallStaticLongMethod
);

generate_jni_primitive_type_tests!(
    jni_type_float_tests,
    jni_sys::jfloat,
    0.,
    42.,
    "F",
    CallFloatMethod,
    CallStaticFloatMethod
);

generate_jni_primitive_type_tests!(
    jni_type_double_tests,
    jni_sys::jdouble,
    0.,
    42.,
    "D",
    CallDoubleMethod,
    CallStaticDoubleMethod
);

macro_rules! jni_method_call {
    ($name:ident, $type:ty, $method:ident, $return_type:ty, $($argument:ident,)*) => {
        unsafe fn $name(
            object: &$type,
            method_id: jni_sys::jmethodID,
            arguments: Self
        ) -> $return_type {
            #[allow(non_snake_case)]
            let ($($argument,)*) = arguments;
            assert!(!object.is_null(), concat!("Can't call ", stringify!($method), " on a null ", stringify!($type), "."));
            call_jni_method!(
                object.env(),
                $method,
                object.raw_object(),
                method_id
                $(,ToJni::to_jni(&$argument))*
            )
        }
    }
}

macro_rules! peel_input_tuple_impls {
    () => ();
    ($type:ident, $($other:ident,)*) => (input_tuple_impls! { $($other,)* });
}

macro_rules! input_tuple_impls {
    ( $($type:ident,)*) => (
        impl<'a, $($type),*> ToJniTuple for ($($type,)*)
        where
            $($type: ToJni,)*
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
        peel_input_tuple_impls! { $($type,)* }
    );
}

input_tuple_impls! {
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11,
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
    use crate::class::test_class;
    use crate::env::test_env;
    use crate::object::test_object;
    use crate::testing::*;
    use crate::vm::test_vm;
    use std::mem;
    use std::ptr;

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
            assert_eq!(METHOD_ARGUMENT0, arguments.0.to_jni());
            assert_eq!(METHOD_ARGUMENT1, arguments.1.to_jni());
            assert_eq!(METHOD_ARGUMENT2, arguments.2.to_jni());
            assert_eq!(METHOD_ARGUMENT3, arguments.3.to_jni());
            assert_eq!(METHOD_ARGUMENT4, arguments.4.to_jni());
            assert_eq!(METHOD_ARGUMENT5, arguments.5.to_jni());
            assert_eq!(METHOD_ARGUMENT7, arguments.6.to_jni());
            assert_eq!(METHOD_ARGUMENT8, arguments.7.to_jni());
            assert_eq!(METHOD_ARGUMENT9, arguments.8.to_jni());
            assert_eq!(METHOD_ARGUMENT10, arguments.9.to_jni());
            assert_eq!(METHOD_ARGUMENT11, arguments.10.to_jni());
            assert_eq!(METHOD_ARGUMENT12, arguments.11.to_jni());
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
