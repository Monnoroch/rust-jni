use crate::env::JniEnv;
use crate::java_methods::FromObject;
use crate::java_methods::{call_static_method, JniSignature};
use crate::java_string::{from_java_string, to_java_string};
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::{CallOutcome, NoException};
use core::ptr::NonNull;
use jni_sys;
use std;
use std::os::raw::c_char;
use std::ptr;

include!("call_jni_method.rs");

/// A type representing a Java
/// [`String`](https://docs.oracle.com/javase/10/docs/api/java/lang/String.html).
// TODO: examples.
#[derive(Debug, Clone)]
pub struct String<'env> {
    object: Object<'env>,
}

impl<'env> String<'env> {
    /// Create a new empty string.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newstring)
    pub fn empty<'a>(token: &NoException<'a>) -> JavaResult<'a, String<'a>> {
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewString` throws an exception before returning `null`.
        let raw_string = unsafe {
            call_nullable_jni_method!(token, NewString, ptr::null(), 0 as jni_sys::jsize)
        }?;
        // Safe because the argument is a valid string reference.
        Ok(unsafe { Self::from_raw(token.env(), raw_string) })
    }

    /// Create a new Java string from a Rust string.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newstringutf)
    pub fn new<'a>(token: &NoException<'a>, string: &str) -> JavaResult<'a, String<'a>> {
        if string.is_empty() {
            return Self::empty(token);
        }

        let buffer = to_java_string(string);
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewStringUTF` throws an exception before returning `null`.
        let raw_string = unsafe {
            call_nullable_jni_method!(token, NewStringUTF, buffer.as_ptr() as *const c_char)
        }?;
        // Safe because the argument is a valid string reference.
        Ok(unsafe { Self::from_raw(token.env(), raw_string) })
    }

    /// String length (the number of unicode characters).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringlength)
    pub fn len(&self, _token: &NoException) -> usize {
        // Safe because arguments are ensured to be the correct by construction.
        let length = unsafe { call_jni_object_method!(self, GetStringLength) };
        length as usize
    }

    /// String size (the number of bytes in modified UTF-8).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringutflength)
    pub fn size(&self, _token: &NoException) -> usize {
        // Safe because arguments are ensured to be the correct by construction.
        let size = unsafe { call_jni_object_method!(self, GetStringUTFLength) };
        size as usize
    }

    /// Convert the Java `String` into a Rust `String`.
    ///
    /// This method has a different signature from the one in the `ToString` trait because
    /// extracting bytes from `String` is only safe when there is no pending exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringutfregion)
    pub fn as_string(&self, token: &NoException) -> std::string::String {
        let length = self.len(token);
        if length == 0 {
            return "".to_owned();
        }

        let size = self.size(token) + 1; // +1 for the '\0' byte.
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        // Safe because arguments are ensured to be the correct by construction.
        unsafe {
            call_jni_object_method!(
                self,
                GetStringUTFRegion,
                0 as jni_sys::jsize,
                length as jni_sys::jsize,
                buffer.as_mut_ptr() as *mut c_char
            );
            buffer.set_len(size);
        }
        // Unwrap should not panic as Java guarantees the string's correctness.
        from_java_string(buffer.as_slice()).unwrap().into_owned()
    }

    /// Get the string value of an integer.
    ///
    /// [`String::valueOf(int)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/String.html#valueOf(int)).
    pub fn value_of_int(
        token: &NoException<'env>,
        value: i32,
    ) -> JavaResult<'env, Option<String<'env>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(i32) -> String<'env>>(token, "valueOf\0", (value,))
        }
    }

    /// Unsafe because an incorrect object reference can be passed.
    #[inline(always)]
    pub(crate) unsafe fn from_raw<'a>(
        env: &'a JniEnv<'a>,
        raw_string: NonNull<jni_sys::_jobject>,
    ) -> String<'a> {
        String {
            object: Object::from_raw(env, raw_string.cast()),
        }
    }
}

/// Allow [`String`](struct.String.html) to be used in place of an [`Object`](struct.Object.html).
impl<'env> ::std::ops::Deref for String<'env> {
    type Target = Object<'env>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'env> AsRef<Object<'env>> for String<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'env> {
        &self.object
    }
}

impl<'env> AsRef<String<'env>> for String<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &String<'env> {
        &*self
    }
}

impl<'a> Into<Object<'a>> for String<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'env> FromObject<'env> for String<'env> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'env>) -> Self {
        Self { object }
    }
}

impl JniSignature for String<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Ljava/lang/String;"
    }
}

/// Allow comparing [`String`](struct.String.html)
/// to Java objects. Java objects are compared by-reference to preserve
/// original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for String<'env>
where
    T: AsRef<Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        Object::as_ref(self).eq(other.as_ref())
    }
}
