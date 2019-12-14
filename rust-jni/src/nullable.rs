use crate::classes::null_pointer_exception::NullPointerException;
use crate::java_class::JavaClassRef;
use crate::result::JavaResult;
use crate::token::NoException;

/// Extension trait that adds common helper methods for working with
/// nullable objects to [`JavaClass`](trait.JavaClass.html) types.
pub trait NullableJavaClassExt<'a, R> {
    /// Convert [`Option<T: JavaClass>`](trait.JavaClass.html) into
    /// [`T: JavaClass`](trait.JavaClass.html), returning a
    /// [`NullPointerException`](java/lang/struct.NullPointerException.html)
    /// on [`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    ///
    /// This follows the standard Java semantics of throwing a
    /// [`NullPointerException`](https://docs.oracle.com/javase/10/docs/api/java/lang/NullPointerException.html)
    /// when a method is called on a `null`.
    ///
    /// Example:
    /// ```
    /// # use rust_jni::*;
    /// # use rust_jni::java::lang::Object;
    /// # use std::ptr;
    /// #
    /// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
    /// let object_class = Object::new(&token)?
    ///     .to_string(&token)
    ///     .or_npe(&token)?
    ///     .class(&token)
    ///     .parent(&token)
    ///     .as_ref()
    ///     .or_npe(&token)?;
    /// # Ok(token)
    /// # }
    /// #
    /// # #[cfg(feature = "libjvm")]
    /// # fn main() {
    /// #     let init_arguments = InitArguments::default();
    /// #     let vm = JavaVM::create(&init_arguments).unwrap();
    /// #     let _ = vm.with_attached(
    /// #        &AttachArguments::new(init_arguments.version()),
    /// #        |token: NoException| {
    /// #            ((), jni_main(token).unwrap())
    /// #        },
    /// #     );
    /// # }
    /// #
    /// # #[cfg(not(feature = "libjvm"))]
    /// # fn main() {}
    /// ```
    fn or_npe(self, token: &NoException<'a>) -> JavaResult<'a, R>;
}

/// Add nullable object helper methods from [`NullableJavaClassExt`](trait.NullableJavaClassExt.html)
/// to [`Option<T: JavaClass>`](type.JavaResult.html).
impl<'a, R> NullableJavaClassExt<'a, R> for Option<R>
where
    R: JavaClassRef<'a>,
{
    #[inline(always)]
    fn or_npe(self, token: &NoException<'a>) -> JavaResult<'a, R> {
        match self {
            Some(value) => Ok(value),
            None => {
                let npe = NullPointerException::new(token)?;
                Err(npe.into())
            }
        }
    }
}

/// Add nullable object helper methods from [`NullableJavaClassExt`](trait.NullableJavaClassExt.html)
/// to [`JavaResult<Option<T: JavaClassRef>>`](type.JavaResult.html).
impl<'a, R> NullableJavaClassExt<'a, R> for JavaResult<'a, Option<R>>
where
    R: JavaClassRef<'a>,
{
    #[inline(always)]
    fn or_npe(self, token: &NoException<'a>) -> JavaResult<'a, R> {
        let result = self?;
        result.or_npe(token)
    }
}
