use crate::env::JniEnv;
use crate::java_class::find_class;
use crate::java_class::JavaClassRef;
use crate::java_methods::JavaArgumentTuple;
use crate::java_methods::JavaArgumentType;
use crate::java_methods::JavaMethodResult;
use crate::java_methods::JniSignature;
use crate::jni_bool;
use crate::jni_methods;
use crate::jni_types::private::JniPrimitiveType;
use crate::native_method::ToJavaNativeArgument;
use crate::result::JavaResult;
use crate::token::NoException;
use std::char;
use std::iter;

pub trait JavaPrimitiveResultType: JniSignature {
    type JniType: JniPrimitiveType;

    fn from_jni(value: Self::JniType) -> Self;
}

macro_rules! jni_signature_trait {
    ($type:ty, $jni_type:ty, $typedoc:expr) => {
        /// Make
        #[doc = $typedoc]
        /// passable to and from Java calls.
        impl JniSignature for $type {
            #[inline(always)]
            fn signature() -> &'static str {
                <$jni_type as JniPrimitiveType>::signature()
            }
        }
    };
}

macro_rules! java_method_result_trait {
    ($type:ty, $jni_type:ty) => {
        impl<'a> JavaMethodResult<'a> for $type {
            type JniType = $jni_type;
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
                T: JavaClassRef<'a>,
                A: JavaArgumentTuple,
            {
                let result: Self::JniType = jni_methods::call_primitive_method(
                    object.as_ref(),
                    token,
                    name,
                    signature,
                    JavaArgumentTuple::to_jni(&arguments),
                )?;
                Ok(Self::from_jni(result))
            }

            #[inline(always)]
            unsafe fn call_static_method<T, A>(
                env: &'a JniEnv<'a>,
                token: &NoException<'a>,
                name: &str,
                signature: &str,
                arguments: A,
            ) -> JavaResult<'a, Self::ResultType>
            where
                T: JavaClassRef<'a>,
                A: JavaArgumentTuple,
            {
                let class = find_class::<T>(env, token)?;
                let result = jni_methods::call_static_primitive_method(
                    &class,
                    token,
                    name,
                    signature,
                    JavaArgumentTuple::to_jni(&arguments),
                )?;
                Ok(Self::from_jni(result))
            }
        }
    };
}

macro_rules! java_primitive_result_type_trait {
    ($type:ty, $jni_type:ty) => {
        impl JavaPrimitiveResultType for $type {
            type JniType = $jni_type;

            #[inline(always)]
            fn from_jni(value: Self::JniType) -> Self {
                value as Self
            }
        }

        java_method_result_trait!($type, $jni_type);
    };
}

macro_rules! jni_primitive_argument_traits {
    ($type:ty, $jni_type:ty, $typedoc:expr) => {
        jni_signature_trait!($type, $jni_type, $typedoc);

        impl JavaArgumentType for $type {
            type JniType = $jni_type;

            #[inline(always)]
            fn to_jni(&self) -> Self::JniType {
                *self as Self::JniType
            }
        }

        impl ToJavaNativeArgument for $type {
            type JniType = <Self as JavaPrimitiveResultType>::JniType;

            unsafe fn from_raw<'a>(_env: &'a JniEnv<'a>, value: Self::JniType) -> Self {
                <Self as JavaPrimitiveResultType>::from_jni(value)
            }
        }
    };
}

macro_rules! java_primitive_traits {
    ($type:ty, $jni_type:ty, $typedoc:expr) => {
        jni_primitive_argument_traits!($type, $jni_type, $typedoc);
        java_primitive_result_type_trait!($type, $jni_type);
    };
}

jni_signature_trait!(
    (),
    (),
    "[`()`](https://doc.rust-lang.org/std/primitive.unit.html)"
);
java_primitive_result_type_trait!((), ());

jni_primitive_argument_traits!(
    bool,
    jni_sys::jboolean,
    "[`bool`](https://doc.rust-lang.org/std/primitive.bool.html)"
);

impl JavaPrimitiveResultType for bool {
    type JniType = jni_sys::jboolean;

    #[inline(always)]
    fn from_jni(value: Self::JniType) -> Self {
        jni_bool::to_rust(value)
    }
}

java_method_result_trait!(bool, jni_sys::jboolean);

jni_primitive_argument_traits!(
    char,
    jni_sys::jchar,
    "[`char`](https://doc.rust-lang.org/std/primitive.char.html)"
);

impl JavaPrimitiveResultType for char {
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
}

java_method_result_trait!(char, jni_sys::jchar);

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
java_primitive_traits!(
    f32,
    jni_sys::jfloat,
    "[`f32`](https://doc.rust-lang.org/std/primitive.f32.html)"
);
java_primitive_traits!(
    f64,
    jni_sys::jdouble,
    "[`f64`](https://doc.rust-lang.org/std/primitive.f64.html)"
);
