use crate::java_methods::FromObject;
use crate::java_methods::JniSignature;
use crate::object::Object;
use crate::throwable::Throwable;

/// A type representing a Java
/// [`Exception`](https://docs.oracle.com/javase/10/docs/api/java/lang/Exception.html).
#[derive(Debug, Clone)]
pub struct Exception<'env> {
    pub(crate) object: Throwable<'env>,
}

/// Allow [`Exception`](struct.Exception.html) to be used in place of an [`Object`](struct.Object.html).
impl<'env> ::std::ops::Deref for Exception<'env> {
    type Target = Object<'env>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'env> AsRef<Object<'env>> for Exception<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'env> {
        &self.object
    }
}

impl<'env> AsRef<Throwable<'env>> for Exception<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Throwable<'env> {
        &self.object
    }
}

impl<'env> AsRef<Exception<'env>> for Exception<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Exception<'env> {
        &*self
    }
}

impl<'a> Into<Throwable<'a>> for Exception<'a> {
    fn into(self) -> Throwable<'a> {
        self.object
    }
}

impl<'a> Into<Object<'a>> for Exception<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'env> FromObject<'env> for Exception<'env> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'env>) -> Self {
        Self {
            object: Throwable::from_object(object),
        }
    }
}

impl JniSignature for Exception<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Ljava/lang/Exception;"
    }
}

/// Allow comparing [`Exception`](struct.Exception.html)
/// to Java objects. Java objects are compared by-reference to preserve
/// original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for Exception<'env>
where
    T: AsRef<Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        Object::as_ref(self).eq(other.as_ref())
    }
}
