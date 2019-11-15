use crate::env::JniEnv;
use crate::jni_bool;
use crate::jni_types::JniPrimitiveType;
use crate::traits::{FromJni, ToJni};
use jni_sys;
use std::char;
use std::iter;

/// Make [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) convertible to
/// [`jboolean`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jboolean.html).
impl ToJni for bool {
    #[doc(hidden)]
    type JniType = jni_sys::jboolean;

    #[doc(hidden)]
    fn signature() -> &'static str {
        <Self::JniType as JniPrimitiveType>::signature()
    }
    unsafe fn to_jni(&self) -> Self::JniType {
        jni_bool::to_jni(*self)
    }
}

/// Make [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) convertible from
/// [`jboolean`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jboolean.html).
impl<'env> FromJni<'env> for bool {
    #[doc(hidden)]
    type JniType = jni_sys::jboolean;

    #[doc(hidden)]
    fn signature() -> &'static str {
        <Self::JniType as JniPrimitiveType>::signature()
    }
    unsafe fn from_jni(_: &'env JniEnv<'env>, value: Self::JniType) -> Self {
        jni_bool::to_rust(value)
    }
}

#[cfg(test)]
mod bool_tests {
    use super::*;
    use crate::env::test_env;
    use crate::vm::test_vm;
    use std::ptr;

    #[test]
    fn signature() {
        assert_eq!(<bool as ToJni>::signature(), "Z");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(true.to_jni(), jni_sys::JNI_TRUE);
            assert_eq!(false.to_jni(), jni_sys::JNI_FALSE);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(bool::from_jni(&env, jni_sys::JNI_TRUE), true);
            assert_eq!(bool::from_jni(&env, jni_sys::JNI_FALSE), false);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(bool::from_jni(&env, true.to_jni()), true);
            assert_eq!(bool::from_jni(&env, false.to_jni()), false);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(
                bool::from_jni(&env, jni_sys::JNI_TRUE).to_jni(),
                jni_sys::JNI_TRUE
            );
            assert_eq!(
                bool::from_jni(&env, jni_sys::JNI_FALSE).to_jni(),
                jni_sys::JNI_FALSE
            );
        }
    }
}

/// Make [`char`](https://doc.rust-lang.org/std/primitive.char.html) convertible to
/// [`jchar`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html).
#[doc(hidden)]
impl ToJni for char {
    #[doc(hidden)]
    type JniType = jni_sys::jchar;

    #[doc(hidden)]
    fn signature() -> &'static str {
        <Self::JniType as JniPrimitiveType>::signature()
    }

    unsafe fn to_jni(&self) -> Self::JniType {
        *self as Self::JniType
    }
}

/// Make [`char`](https://doc.rust-lang.org/std/primitive.char.html) convertible from
/// [`jchar`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html).
#[doc(hidden)]
impl<'env> FromJni<'env> for char {
    #[doc(hidden)]
    type JniType = jni_sys::jchar;

    #[doc(hidden)]
    fn signature() -> &'static str {
        <Self::JniType as JniPrimitiveType>::signature()
    }

    unsafe fn from_jni(_: &'env JniEnv<'env>, value: Self::JniType) -> Self {
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
    use crate::env::test_env;
    use crate::vm::test_vm;
    use std::ptr;

    #[test]
    fn signature() {
        assert_eq!(<char as ToJni>::signature(), "C");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!('h'.to_jni(), 'h' as jni_sys::jchar);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(char::from_jni(&env, 'h' as jni_sys::jchar), 'h');
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(char::from_jni(&env, 'h'.to_jni()), 'h');
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(
                char::from_jni(&env, 'h' as jni_sys::jchar).to_jni(),
                'h' as jni_sys::jchar
            );
        }
    }
}

/// A macro for generating [`JavaType`](trait.JavaType.html) implementations for most primitive
/// Rust types.
macro_rules! jni_input_traits {
    ($type:ty, $jni_type:ty, $link:expr, $jni_sys_link:expr) => {
        /// Make
        #[doc = $link]
        /// convertible from
        #[doc = $jni_sys_link]
        ///.
        #[doc(hidden)]
        impl<'env> FromJni<'env> for $type {
            #[doc(hidden)]
            type JniType = $jni_type;

            #[doc(hidden)]
            fn signature() -> &'static str {
                <Self::JniType as JniPrimitiveType>::signature()
            }

            unsafe fn from_jni(_: &'env JniEnv<'env>, value: Self::JniType) -> Self {
                value as Self
            }
        }
    };
}

/// A macro for generating [`JavaType`](trait.JavaType.html) implementations for most primitive
/// Rust types.
macro_rules! jni_io_traits {
    ($type:ty, $jni_type:ty, $link:expr, $jni_sys_link:expr) => {
        jni_input_traits!(
            $type,
            $jni_type,
            $link,
            $jni_sys_link
        );

        /// Make
        #[doc = $link]
        /// convertible to
        #[doc = $jni_sys_link]
        ///.
        #[doc(hidden)]
        impl ToJni for $type {
            #[doc(hidden)]
            type JniType = $jni_type;

            #[doc(hidden)]
            fn signature() -> &'static str {
                <Self::JniType as JniPrimitiveType>::signature()
            }

            unsafe fn to_jni(&self) -> Self::JniType {
                *self as Self::JniType
            }
        }
    };
}

jni_input_traits!(
    (),
    (),
    "[`()`](https://doc.rust-lang.org/std/primitive.unit.html)",
    "[`()`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jchar.html)"
);
jni_io_traits!(
    u8,
    jni_sys::jbyte,
    "[`u8`](https://doc.rust-lang.org/std/primitive.u8.html)",
    "[`jbyte`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jbyte.html)"
);
jni_io_traits!(
    i16,
    jni_sys::jshort,
    "[`i16`](https://doc.rust-lang.org/std/primitive.i16.html)",
    "[`jshort`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jshort.html)"
);
jni_io_traits!(
    i32,
    jni_sys::jint,
    "[`i32`](https://doc.rust-lang.org/std/primitive.i32.html)",
    "[`jint`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jint.html)"
);
jni_io_traits!(
    i64,
    jni_sys::jlong,
    "[`i64`](https://doc.rust-lang.org/std/primitive.i64.html)",
    "[`jlong`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jlong.html)"
);
// For some reason, floats need to be passed as 64-bit floats to JNI.
// When passed as 32-bit numbers, Java recieves `0.0` instead of the passed number.
// TODO(#25): figure out the underlying cause of this.
// native call -> java: f64
// java -> native call: f32
// java -> native method: f64
// native method -> java: f64
// jni_io_traits!(
//     f32,
//     jni_sys::jfloat,
//     "[`f32`](https://doc.rust-lang.org/std/primitive.f32.html)",
//     "[`jfloat`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jfloat.html)"
// );
jni_io_traits!(
    f64,
    jni_sys::jdouble,
    "[`f64`](https://doc.rust-lang.org/std/primitive.f64.html)",
    "[`jdouble`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jdouble.html)"
);

#[cfg(test)]
mod void_tests {
    use super::*;
    use crate::env::test_env;
    use crate::vm::test_vm;
    use std::ptr;

    #[test]
    fn signature() {
        assert_eq!(<() as FromJni>::signature(), "V");
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(<()>::from_jni(&env, ()), ());
        }
    }
}

#[cfg(test)]
mod byte_tests {
    use super::*;
    use crate::env::test_env;
    use crate::vm::test_vm;
    use std::ptr;

    #[test]
    fn signature() {
        assert_eq!(<u8 as ToJni>::signature(), "B");
    }

    #[test]
    fn to_jni() {
        unsafe {
            assert_eq!(217.to_jni(), 217);
        }
    }

    #[test]
    fn from_jni() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(u8::from_jni(&env, 217 as u8 as i8), 217);
        }
    }

    #[test]
    fn to_and_from() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(u8::from_jni(&env, (217 as u8).to_jni()), 217);
        }
    }

    #[test]
    fn from_and_to() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        unsafe {
            assert_eq!(
                u8::from_jni(&env, 217 as u8 as i8).to_jni(),
                217 as u8 as i8
            );
        }
    }
}

macro_rules! generate_primitive_tests {
    ($module:ident, $type:ty, $value:expr, $signature:expr) => {
        #[cfg(test)]
        mod $module {
            use super::*;
            use crate::env::test_env;
            use crate::vm::test_vm;
            use std::ptr;

            #[test]
            fn signature() {
                assert_eq!(<$type as ToJni>::signature(), $signature);
            }

            #[test]
            fn to_jni() {
                unsafe {
                    assert_eq!($value.to_jni(), $value);
                }
            }

            #[test]
            fn from_jni() {
                let vm = test_vm(ptr::null_mut());
                let env = test_env(&vm, ptr::null_mut());
                unsafe {
                    assert_eq!(<$type as FromJni>::from_jni(&env, $value), $value);
                }
            }

            #[test]
            fn to_and_from() {
                let vm = test_vm(ptr::null_mut());
                let env = test_env(&vm, ptr::null_mut());
                unsafe {
                    assert_eq!(
                        <$type as FromJni>::from_jni(&env, ($value as $type).to_jni()),
                        $value
                    );
                }
            }

            #[test]
            fn from_and_to() {
                let vm = test_vm(ptr::null_mut());
                let env = test_env(&vm, ptr::null_mut());
                unsafe {
                    assert_eq!(<$type as FromJni>::from_jni(&env, $value).to_jni(), $value);
                }
            }
        }
    };
}

generate_primitive_tests!(short_tests, i16, 217, "S");
generate_primitive_tests!(int_tests, i32, 217, "I");
generate_primitive_tests!(long_tests, i64, 217, "J");
// TODO(#25): re-enable. See the comment above.
// generate_primitive_tests!(float_tests, f32, 217., "F");
generate_primitive_tests!(double_tests, f64, 217., "D");
