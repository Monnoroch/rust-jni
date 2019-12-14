use crate::throwable::Throwable;

/// A type that represents a result of a Java method call. A Java method can either return
/// a result or throw a
/// [`Throwable`](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html)
/// in which case it will be captured in a [`Throwable`](java/lang/struct.Throwable.html) value.
pub type JavaResult<'env, T> = Result<T, Throwable<'env>>;
