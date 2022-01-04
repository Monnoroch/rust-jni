use crate::env::JniEnv;
use crate::java_class::find_class;
use crate::java_class::JavaClass;
use crate::java_class::JniSignature;
use crate::java_methods::JavaArgumentType;
use crate::java_methods::JavaMethodResult;
use crate::java_methods::ToJniType;
use crate::jni_bool;
use crate::jni_methods;
use crate::jni_types::private::JniArgumentTypeTuple;
use crate::jni_types::private::JniPrimitiveType;
use crate::native_method::ToJavaNativeArgument;
use crate::native_method::ToJavaNativeResult;
use crate::result::JavaResult;
use crate::token::NoException;
use std::char;
use std::iter;

pub trait JavaPrimitiveType: JniSignature {
    type JniType: JniPrimitiveType;

    fn from_jni(value: Self::JniType) -> Self;
    fn to_jni(self) -> Self::JniType;
}

macro_rules! java_signature_trait {
    ($type:ty, $typedoc:expr) => {
        /// Make
        #[doc = $typedoc]
        /// passable to and from Java calls.
        impl JniSignature for $type {
            #[inline(always)]
            fn signature() -> &'static str {
                <<Self as JavaPrimitiveType>::JniType as JniPrimitiveType>::signature()
            }
        }
    };
}

macro_rules! java_primitive_type_trait {
    ($type:ty, $jni_type:ty, $typedoc:expr) => {
        impl JavaPrimitiveType for $type {
            type JniType = $jni_type;

            #[inline(always)]
            fn from_jni(value: Self::JniType) -> Self {
                value as Self
            }

            #[inline(always)]
            fn to_jni(self) -> Self::JniType {
                self as Self::JniType
            }
        }

        java_signature_trait!($type, $typedoc);
    };
}

macro_rules! java_method_result_trait {
    ($type:ty) => {
        impl<'a> JavaMethodResult<'a> for $type {
            type ResultType = Self;

            #[inline(always)]
            unsafe fn call_method<T, A>(
                object: &T,
                token: &NoException<'a>,
                name: &str,
                signature: &str,
                arguments: A,
            ) -> JavaResult<'a, Self::ResultType>
            where
                T: JavaClass<'a>,
                A: JniArgumentTypeTuple,
            {
                let result: <Self as JavaPrimitiveType>::JniType =
                    jni_methods::call_primitive_method(
                        object.as_ref(),
                        token,
                        name,
                        signature,
                        arguments,
                    )?;
                Ok(JavaPrimitiveType::from_jni(result))
            }

            #[inline(always)]
            unsafe fn call_static_method<T, A>(
                token: &NoException<'a>,
                name: &str,
                signature: &str,
                arguments: A,
            ) -> JavaResult<'a, Self::ResultType>
            where
                T: JavaClass<'a>,
                A: JniArgumentTypeTuple,
            {
                let class = find_class::<T>(token)?;
                let result: <Self as JavaPrimitiveType>::JniType =
                    jni_methods::call_static_primitive_method(
                        &class, token, name, signature, arguments,
                    )?;
                Ok(JavaPrimitiveType::from_jni(result))
            }
        }

        impl ToJavaNativeResult for $type {
            type JniType = <Self as JavaPrimitiveType>::JniType;

            #[inline(always)]
            unsafe fn into_java_native_result(self) -> Self::JniType {
                JavaPrimitiveType::to_jni(self)
            }
        }
    };
}

macro_rules! java_primitive_argument_trait {
    ($type:ty) => {
        impl<'a, 'this: 'a> JavaArgumentType<'a, 'this> for $type {
            type ActualType = Self;
        }

        impl ToJniType for $type {
            type JniType = <Self as JavaPrimitiveType>::JniType;

            #[inline(always)]
            unsafe fn to_jni(&self) -> Self::JniType {
                JavaPrimitiveType::to_jni(*self)
            }
        }
    };
}

macro_rules! java_primitive_native_argument_trait {
    ($type:ty) => {
        impl<'this> ToJavaNativeArgument<'this> for $type {
            type JniType = <Self as JavaPrimitiveType>::JniType;
            type ArgumentType = Self;

            #[inline(always)]
            unsafe fn from_raw(_env: &'this JniEnv<'this>, value: Self::JniType) -> Self {
                <Self as JavaPrimitiveType>::from_jni(value)
            }
        }
    };
}

macro_rules! java_primitive_traits {
    ($type:ty, $jni_type:ty, $typedoc:expr) => {
        java_primitive_type_trait!($type, $jni_type, $typedoc);
        java_primitive_argument_trait!($type);
        java_primitive_native_argument_trait!($type);
        java_method_result_trait!($type);
    };
}

java_primitive_type_trait!(
    (),
    (),
    "[`()`](https://doc.rust-lang.org/std/primitive.unit.html)"
);
java_method_result_trait!(());

impl JavaPrimitiveType for bool {
    type JniType = jni_sys::jboolean;

    #[inline(always)]
    fn from_jni(value: Self::JniType) -> Self {
        jni_bool::to_rust(value)
    }

    #[inline(always)]
    fn to_jni(self) -> Self::JniType {
        jni_bool::to_jni(self)
    }
}
java_signature_trait!(
    bool,
    "[`bool`](https://doc.rust-lang.org/std/primitive.bool.html)"
);
java_primitive_argument_trait!(bool);
java_primitive_native_argument_trait!(bool);
java_method_result_trait!(bool);

impl JavaPrimitiveType for char {
    type JniType = jni_sys::jchar;

    #[inline(always)]
    fn from_jni(value: Self::JniType) -> Self {
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

    #[inline(always)]
    fn to_jni(self) -> Self::JniType {
        // TODO: find out if this is correct.
        self as Self::JniType
    }
}
java_signature_trait!(
    char,
    "[`char`](https://doc.rust-lang.org/std/primitive.char.html)"
);
java_primitive_argument_trait!(char);
java_primitive_native_argument_trait!(char);
java_method_result_trait!(char);

java_primitive_traits!(
    u8,
    jni_sys::jbyte,
    "[`u8`](https://doc.rust-lang.org/std/primitive.u8.html)"
);
java_primitive_traits!(
    i16,
    jni_sys::jshort,
    "[`i16`](https://doc.rust-lang.org/std/primitive.i16.html)"
);
java_primitive_traits!(
    i32,
    jni_sys::jint,
    "[`i32`](https://doc.rust-lang.org/std/primitive.i32.html)"
);
java_primitive_traits!(
    i64,
    jni_sys::jlong,
    "[`i64`](https://doc.rust-lang.org/std/primitive.i64.html)"
);

java_primitive_type_trait!(
    f32,
    jni_sys::jfloat,
    "[`f32`](https://doc.rust-lang.org/std/primitive.f32.html)"
);
// TODO(#25): floating point numbers don't work properly.
// java_primitive_argument_trait!(f32);
java_primitive_native_argument_trait!(f32);
java_method_result_trait!(f32);

java_primitive_traits!(
    f64,
    jni_sys::jdouble,
    "[`f64`](https://doc.rust-lang.org/std/primitive.f64.html)"
);
