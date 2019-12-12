use crate::env::JniEnv;
use crate::error::JniError;
use crate::java_methods::FromObject;
use crate::java_methods::JavaObjectArgument;
use crate::java_methods::{call_constructor, call_method, JniSignature};
use crate::object::Object;
use crate::result::JavaResult;
use crate::string::String;
use crate::token::{Exception, NoException};
use jni_sys;

use std::ptr::NonNull;

include!("call_jni_method.rs");

/// A type representing a Java
/// [`Throwable`](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html).
// TODO: examples.
#[derive(Debug, Clone)]
pub struct Throwable<'env> {
    object: Object<'env>,
}

impl<'env> Throwable<'env> {
    /// Throw the exception. Transfers ownership of the object to Java.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#throw)
    pub fn throw<'token>(self, token: NoException<'token>) -> Exception<'token>
    where
        'env: 'token,
    {
        // Safe because the argument is ensured to be correct references by construction.
        let error = JniError::from_raw(unsafe { call_jni_object_method!(self, Throw) });
        // Can't really handle failing throwing an exception.
        if error.is_some() {
            panic!(
                "Throwing an exception has failed with status {:?}.",
                error.unwrap()
            );
        }
        // Safe becuase we just threw the exception.
        unsafe { token.exchange() }
    }

    /// Returns a short description of this [`Throwable`](struct.Throwable.html).
    ///
    /// [`Throwable::getMessage` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#getMessage())
    pub fn get_message(&self, token: &NoException<'env>) -> JavaResult<'env, Option<String<'env>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_method::<Self, _, _, fn() -> String<'env>>(self, token, "getMessage\0", ()) }
    }

    /// Returns a short description of this [`Throwable`](struct.Throwable.html).
    ///
    /// [`Throwable::getCause` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#getCause())
    pub fn get_cause(
        &self,
        token: &NoException<'env>,
    ) -> JavaResult<'env, Option<Throwable<'env>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_method::<Self, _, _, fn() -> Throwable<'env>>(self, token, "getCause\0", ()) }
    }

    /// Create a new [`Throwable`](struct.Throwable.html).
    ///
    /// [`Throwable(String)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#<init>())
    pub fn new(token: &NoException<'env>) -> JavaResult<'env, Throwable<'env>> {
        unsafe { call_constructor::<Self, _, fn()>(token, ()) }
    }

    /// Create a new [`Throwable`](struct.Throwable.html) with a message.
    ///
    /// [`Throwable(String)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#<init>(java.lang.String))
    pub fn new_with_message(
        token: &NoException<'env>,
        message: impl JavaObjectArgument<'env, String<'env>>,
    ) -> JavaResult<'env, Throwable<'env>> {
        unsafe {
            call_constructor::<Self, _, fn(Option<&String<'env>>)>(token, (message.as_argument(),))
        }
    }

    /// Create a new [`Throwable`](struct.Throwable.html) with a cause.
    ///
    /// [`Throwable(String)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#<init>(java.lang.Throwable))
    pub fn new_with_cause(
        token: &NoException<'env>,
        cause: impl JavaObjectArgument<'env, Throwable<'env>>,
    ) -> JavaResult<'env, Throwable<'env>> {
        unsafe {
            call_constructor::<Self, _, fn(Option<&Throwable<'env>>)>(token, (cause.as_argument(),))
        }
    }

    /// Create a new [`Throwable`](struct.Throwable.html) with a message and a cause.
    ///
    /// [`Throwable(String)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#<init>(java.lang.String,java.lang.Throwable))
    pub fn new_with_message_and_cause(
        token: &NoException<'env>,
        message: impl JavaObjectArgument<'env, String<'env>>,
        cause: impl JavaObjectArgument<'env, Throwable<'env>>,
    ) -> JavaResult<'env, Throwable<'env>> {
        unsafe {
            call_constructor::<Self, _, fn(Option<&String<'env>>, Option<&Throwable<'env>>)>(
                token,
                (message.as_argument(), cause.as_argument()),
            )
        }
    }

    /// Unsafe because the argument mught not be a valid class reference.
    #[inline(always)]
    pub(crate) unsafe fn from_raw<'a>(
        env: &'a JniEnv<'a>,
        raw_class: NonNull<jni_sys::_jobject>,
    ) -> Throwable<'a> {
        Throwable {
            object: Object::from_raw(env, raw_class.cast()),
        }
    }
}

/// Allow [`Throwable`](struct.Throwable.html) to be used in place of an [`Object`](struct.Object.html).
impl<'env> ::std::ops::Deref for Throwable<'env> {
    type Target = Object<'env>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'env> AsRef<Object<'env>> for Throwable<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'env> {
        &self.object
    }
}

impl<'env> AsRef<Throwable<'env>> for Throwable<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Throwable<'env> {
        &*self
    }
}

impl<'a> Into<Object<'a>> for Throwable<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'env> FromObject<'env> for Throwable<'env> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'env>) -> Self {
        Self { object }
    }
}

impl JniSignature for Throwable<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Ljava/lang/Throwable;"
    }
}

/// Allow comparing [`Throwable`](struct.Throwable.html)
/// to Java objects. Java objects are compared by-reference to preserve
/// original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for Throwable<'env>
where
    T: AsRef<Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        Object::as_ref(self).eq(other.as_ref())
    }
}
