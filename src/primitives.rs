use jni::*;
use jni_sys;
use std::char;
use std::iter;
#[cfg(test)]
use std::ptr;

/// A macro for generating [`JniType`](trait.JniType.html) implementation for primitive types.
macro_rules! jni_type_trait {
    ($type:ty) => {
        impl JniType for $type {}
    };
}

jni_type_trait!(jni_sys::jobject);
jni_type_trait!(());
jni_type_trait!(jni_sys::jboolean);
jni_type_trait!(jni_sys::jchar);
jni_type_trait!(jni_sys::jbyte);
jni_type_trait!(jni_sys::jshort);
jni_type_trait!(jni_sys::jint);
jni_type_trait!(jni_sys::jlong);
jni_type_trait!(jni_sys::jfloat);
jni_type_trait!(jni_sys::jdouble);

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
