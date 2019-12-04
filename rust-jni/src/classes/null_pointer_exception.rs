use crate::classes::exception::Exception;
use crate::env::JniEnv;
use crate::java_methods::call_constructor;
use crate::java_methods::FromObject;
use crate::java_methods::JniSignature;
use crate::object::Object;
use crate::result::JavaResult;
use crate::throwable::Throwable;
use crate::token::NoException;

/// A type representing a Java
/// [`NullPointerException`](https://docs.oracle.com/javase/10/docs/api/java/lang/NullPointerException.html).
#[derive(Debug, Clone)]
pub struct NullPointerException<'env> {
    pub(crate) object: Exception<'env>,
}

impl<'this> NullPointerException<'this> {
    /// Create a new [`NullPointerException`](struct.NullPointerException.html) with a message.
    ///
    /// [`NullPointerException()` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/NullPointerException.html#<init>())
    pub fn new(
        env: &'this JniEnv<'this>,
        token: &NoException<'this>,
    ) -> JavaResult<'this, NullPointerException<'this>> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_constructor::<Self, _, fn()>(&env, token, ()) }
    }
}

/// Allow [`NullPointerException`](struct.NullPointerException.html) to be used in place of an
/// [`Object`](struct.Object.html).
impl<'env> ::std::ops::Deref for NullPointerException<'env> {
    type Target = Object<'env>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'env> AsRef<Object<'env>> for NullPointerException<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'env> {
        &self.object
    }
}

impl<'env> AsRef<Throwable<'env>> for NullPointerException<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Throwable<'env> {
        self.object.as_ref()
    }
}

impl<'env> AsRef<Exception<'env>> for NullPointerException<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Exception<'env> {
        &self.object
    }
}

impl<'env> AsRef<NullPointerException<'env>> for NullPointerException<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &NullPointerException<'env> {
        &*self
    }
}

impl<'a> Into<Exception<'a>> for NullPointerException<'a> {
    fn into(self) -> Exception<'a> {
        self.object
    }
}

impl<'a> Into<Throwable<'a>> for NullPointerException<'a> {
    fn into(self) -> Throwable<'a> {
        self.object.into()
    }
}

impl<'a> Into<Object<'a>> for NullPointerException<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'env> FromObject<'env> for NullPointerException<'env> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'env>) -> Self {
        Self {
            object: Exception::from_object(object),
        }
    }
}

impl JniSignature for NullPointerException<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Ljava/lang/NullPointerException;"
    }
}

/// Allow comparing [`NullPointerException`](struct.NullPointerException.html)
/// to Java objects. Java objects are compared by-reference to preserve
/// original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for NullPointerException<'env>
where
    T: AsRef<Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        Object::as_ref(self).eq(other.as_ref())
    }
}
