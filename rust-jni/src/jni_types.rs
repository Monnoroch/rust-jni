use crate::class::Class;
use crate::object::Object;
use crate::token::NoException;
use jni_sys;
use std::ptr;

include!("call_jni_method.rs");

pub(crate) mod private {
    use super::*;

    /// A trait that represents a JNI type. It's implemented for all JNI primitive types
    /// and [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
    /// Implements Java method calls and provides the default value for this JNI type.
    pub trait JniType {
        fn default() -> Self;

        unsafe fn call_method<In: JniArgumentTypeTuple>(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: In,
        ) -> Self;

        unsafe fn call_static_method<In: JniArgumentTypeTuple>(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: In,
        ) -> Self;
    }

    /// A trait that represents primitive JNI types. It's implemented for all JNI primitive types.
    pub trait JniPrimitiveType: JniType {
        fn signature() -> &'static str;
    }

    /// A trait that represents JNI types that can be passed as arguments to JNI functions.
    /// Implemented for all JNI types except for [`()`](https://doc.rust-lang.org/stable/std/primitive.unit.html).
    ///
    /// Temporarily [not implemented](https://github.com/Monnoroch/rust-jni/issues/25) for
    /// [`jfloat`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jfloat.html).
    pub trait JniArgumentType: JniType {}

    /// A trait that represents JNI types that can be passed as arguments to native Java functions.
    /// Implemented for all JNI types except for [`()`](https://doc.rust-lang.org/stable/std/primitive.unit.html).
    // TODO(#25): remove this trait and replace with JniArgumentType when the float issue is fixed.
    pub trait JniNativeArgumentType: JniType {}

    /// A trait that implements calling JNI variadic functions using a macro to generate
    /// it's instances for tuples of different sizes.
    /// This is essentially the "[`JniType`](trait.JniType.html) for packed argument tuples".
    // TODO: reimplement once Rust has variadic functions or variadic templates.
    pub trait JniArgumentTypeTuple {
        unsafe fn call_constructor(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jobject;

        unsafe fn call_object_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jobject;

        unsafe fn call_static_object_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jobject;

        unsafe fn call_void_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> ();

        unsafe fn call_static_void_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> ();

        unsafe fn call_boolean_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jboolean;

        unsafe fn call_static_boolean_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jboolean;

        unsafe fn call_char_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jchar;

        unsafe fn call_static_char_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jchar;

        unsafe fn call_byte_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jbyte;

        unsafe fn call_static_byte_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jbyte;

        unsafe fn call_short_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jshort;

        unsafe fn call_static_short_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jshort;

        unsafe fn call_int_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jint;

        unsafe fn call_static_int_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jint;

        unsafe fn call_long_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jlong;

        unsafe fn call_static_long_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jlong;

        unsafe fn call_float_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jfloat;

        unsafe fn call_static_float_method(
            token: &NoException,
            class: &Class,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jfloat;

        unsafe fn call_double_method(
            token: &NoException,
            object: &Object,
            method_id: jni_sys::jmethodID,
            arguments: Self,
        ) -> jni_sys::jdouble;

        unsafe fn call_static_double_method(
            token: &NoException,
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
            #[inline(always)]
            fn default() -> Self {
                $default
            }

            #[inline(always)]
            unsafe fn call_method<In: JniArgumentTypeTuple>(
                token: &NoException,
                object: &Object,
                method_id: jni_sys::jmethodID,
                arguments: In,
            ) -> Self {
                In::$method(token, object, method_id, arguments)
            }

            #[inline(always)]
            unsafe fn call_static_method<In: JniArgumentTypeTuple>(
                token: &NoException,
                class: &Class,
                method_id: jni_sys::jmethodID,
                arguments: In,
            ) -> Self {
                In::$static_method(token, class, method_id, arguments)
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
            #[inline(always)]
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

macro_rules! jni_method_call {
    ($name:ident, $type:ty, $method:ident, $return_type:ty, $($argument:ident,)*) => {
        #[inline(always)]
        unsafe fn $name(
            token: &NoException,
            object: &$type,
            method_id: jni_sys::jmethodID,
            arguments: Self
        ) -> $return_type {
            #[allow(non_snake_case)]
            let ($($argument,)*) = arguments;
            call_jni_object_method!(
                token,
                object,
                $method,
                method_id
                $(,$argument)*
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
        impl<'a, $($type),*> JniArgumentTypeTuple for ($($type,)*)
        where
            $($type: JniArgumentType,)*
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

impl JniArgumentType for jni_sys::jboolean {}
impl JniArgumentType for jni_sys::jchar {}
impl JniArgumentType for jni_sys::jbyte {}
impl JniArgumentType for jni_sys::jshort {}
impl JniArgumentType for jni_sys::jint {}
impl JniArgumentType for jni_sys::jlong {}
// TODO(#25): floating point numbers don't work properly.
// impl JniArgumentType for jni_sys::jfloat {}
impl JniArgumentType for jni_sys::jdouble {}
impl JniArgumentType for jni_sys::jobject {}

impl<T> JniNativeArgumentType for T where T: JniArgumentType {}
impl JniNativeArgumentType for jni_sys::jfloat {}

// [`()`](https://doc.rust-lang.org/stable/std/primitive.unit.html)
// can't be passed as an argument to a function.
// impl !JniArgumentType for () {}
