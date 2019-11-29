use crate::env::JniEnv;
use crate::java_class::JavaClassRef;
use crate::java_class::NullableJavaClassExt;
use crate::throwable::Throwable;
use crate::token::NoException;

/// A type that represents a result of a Java method call. A Java method can either return
/// a result or throw a
/// [`Throwable`](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html)
/// in which case it will be captured in a [`Throwable`](java/lang/struct.Throwable.html) value.
pub type JavaResult<'env, T> = Result<T, Throwable<'env>>;

/// Add nullable object helper methods from [`NullableJavaClassExt`](trait.NullableJavaClassExt.html)
/// to [`JavaResult<Option<T: JavaClassRef>>`](type.JavaResult.html).
impl<'a, R> NullableJavaClassExt<'a, R> for JavaResult<'a, Option<R>>
where
    R: JavaClassRef<'a>,
{
    #[inline(always)]
    fn or_npe(self, env: &'a JniEnv<'a>, token: &NoException<'a>) -> JavaResult<'a, R> {
        let result = self?;
        result.or_npe(env, token)
    }
}
