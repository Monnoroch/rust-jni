use crate::traits::{FromJni, JavaMethodSignature, ToJni};
use std::string;

macro_rules! braces {
    ($name:ident) => {
        "{}"
    };
}

macro_rules! peel_fn_impls {
    () => ();
    ($type:ident, $($other:ident,)*) => (fn_impls! { $($other,)* });
}

/// A macro for generating method signatures.
///
/// Function arguments must be `ToJni` and the result must be `FromJni`.
macro_rules! fn_impls {
    ( $($type:ident,)*) => (
        impl<'a, $($type,)* Out, T> JavaMethodSignature<($($type,)*), Out> for T
            where
                $($type: ToJni,)*
                Out: FromJni<'a>,
                T: FnOnce($($type,)*) -> Out + ?Sized,
        {
            fn __signature() -> string::String {
                format!(concat!("(", $(braces!($type), )* "){}"), $(<$type as ToJni>::signature(),)* Out::signature())
            }
        }

        peel_fn_impls! { $($type,)* }
    );
}

fn_impls! {
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
mod method_signature_tests {
    use super::*;
    use crate::class::Class;
    use crate::object::Object;

    #[test]
    fn no_arguments() {
        assert_eq!(<fn()>::__signature(), "()V");
    }

    #[test]
    fn one_argument() {
        assert_eq!(<fn(i32) -> i64>::__signature(), "(I)J");
    }

    #[test]
    fn non_primitives() {
        assert_eq!(
            <fn(Object) -> Class>::__signature(),
            "(Ljava/lang/Object;)Ljava/lang/Class;"
        );
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
