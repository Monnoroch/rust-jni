use crate::traits::{FromJni, JavaMethodSignature, JniArgumentType, ToJni};
use std::string;

macro_rules! braces {
    ($name:ident) => {
        "{}"
    };
}

macro_rules! peel_fn_impls {
    () => ();
    ($type:ident, $jni_type:ident, $($other:ident,)*) => (fn_impls! { $($other,)* });
}

/// A macro for generating method signatures.
///
/// Function arguments must be `ToJni` with `ToJni::__JniType: JniArgumentType`
/// and the result must be `FromJni`.
macro_rules! fn_impls {
    ( $($type:ident, $jni_type:ident,)*) => (
        impl<'a, $($type, $jni_type,)* Out, T> JavaMethodSignature<($($type,)*), Out> for T
            where
                $($type: ToJni<__JniType = $jni_type>,)*
                $($jni_type: JniArgumentType,)*
                Out: FromJni<'a>,
                T: FnOnce($($type,)*) -> Out + ?Sized,
        {
            fn __signature() -> string::String {
                format!(concat!("(", $(braces!($type), )* "){}"), $(<$type>::__signature(),)* Out::__signature())
            }
        }

        peel_fn_impls! { $($type, $jni_type,)* }
    );
}

fn_impls! {
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
mod method_signature_tests {
    use super::*;

    #[test]
    fn no_arguments() {
        assert_eq!(<fn()>::__signature(), "()V");
    }

    #[test]
    fn one_argument() {
        assert_eq!(<fn(i32) -> i64>::__signature(), "(I)J");
    }

    #[test]
    fn many_arguments() {
        assert_eq!(
            <fn(i32, f64, u8, f64, bool, i16, i64, i32, i32, i32, i32, char) -> bool>::__signature(
            ),
            "(IDBDZSJIIIIC)Z"
        );
    }
}
