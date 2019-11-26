use crate::class::Class;
use crate::env::JniEnv;
use crate::java_methods::FromObject;
use crate::java_methods::JniSignature;
use crate::java_methods::{call_constructor, call_method};
use crate::jni_bool;
use crate::result::JavaResult;
use crate::string::String;
use crate::token::{CallOutcome, NoException};
use core::ptr::NonNull;
use jni_sys;
use std::fmt;

include!("call_jni_method.rs");

/// A type representing the
/// [`java.lang.Object`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html) class
/// -- the root class of Java's class hierarchy.
///
/// [`Object` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html)
// TODO: examples.
pub struct Object<'env> {
    env: &'env JniEnv<'env>,
    raw_object: NonNull<jni_sys::_jobject>,
}

// [`Object`](struct.Object.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for Object<'env> {}
// impl<'env> !Sync for Object<'env> {}

impl<'env> Object<'env> {
    /// Get the raw object pointer.
    ///
    /// This function provides low-level access to the Java object and thus is unsafe.
    #[inline(always)]
    pub unsafe fn raw_object(&self) -> NonNull<jni_sys::_jobject> {
        self.raw_object
    }

    /// Get the [`JniEnv`](../../struct.JniEnv.html) this object is bound to.
    #[inline(always)]
    pub fn env(&self) -> &'env JniEnv<'env> {
        self.env
    }

    /// Get the object's class.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getobjectclass)
    pub fn class(&self, _token: &NoException) -> Class<'env> {
        // Safe because arguments are ensured to be correct references by construction.
        let raw_java_class = unsafe { call_jni_object_method!(self, GetObjectClass) };
        NonNull::new(raw_java_class)
            .map(|raw_java_class| {
                // Safe because arguments are ensured to be correct references by construction.
                unsafe { Class::from_raw(self.env, raw_java_class) }
            })
            .unwrap_or_else(|| panic!("Object {:?} doesn't have a class.", self.raw_object))
    }

    /// Compare with another Java object by reference.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#issameobject)
    pub fn is_same_as(&self, _token: &NoException, other: &Object) -> bool {
        // Safe because arguments are ensured to be correct references by construction.
        let same =
            unsafe { call_jni_object_method!(self, IsSameObject, other.raw_object().as_ptr()) };
        jni_bool::to_rust(same)
    }

    /// Check if the object is an instance of the class.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#isinstanceof)
    pub fn is_instance_of(&self, _token: &NoException, class: &Class) -> bool {
        // Safe because arguments are ensured to be correct references by construction.
        let is_instance =
            unsafe { call_jni_object_method!(self, IsInstanceOf, class.raw_object().as_ptr()) };
        jni_bool::to_rust(is_instance)
    }

    /// Clone the [`Object`](struct.Object.html). This is not a deep clone of the Java object,
    /// but a Rust-like clone of the value. Since Java objects are reference counted, this will
    /// increment the reference count.
    ///
    /// This method has a different signature from the one in the
    /// [`Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html) trait because
    /// cloning a Java object is only safe when there is no pending exception and because
    /// cloning a java object cat throw an exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newlocalref)
    pub(crate) fn clone_object(&self, token: &NoException<'env>) -> JavaResult<'env, Object<'env>> {
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewLocalRef` throws an exception before returning `null`.
        let raw_object = unsafe {
            call_nullable_jni_method!(self.env(), token, NewLocalRef, self.raw_object().as_ptr())?
        };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(self.env, raw_object) })
    }

    /// Convert the object to a string.
    ///
    /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
    pub fn to_string(&self, token: &NoException<'env>) -> JavaResult<'env, Option<String<'env>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_method::<Self, _, _, fn() -> String<'env>>(self, token, "toString\0", ()) }
    }

    /// Compare to another Java object.
    ///
    /// [`Object::equals` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#equals(java.lang.Object))
    pub fn equals(
        &self,
        token: &NoException<'env>,
        other: &Object<'env>,
    ) -> JavaResult<'env, bool> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(&Object<'env>) -> bool>(self, token, "equals\0", (other,))
        }
    }

    /// Get the hash code of the [`Object`](struct.Object.html).
    ///
    /// [`Object::hashCode` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#hashCode())
    pub fn hash_code(&self, token: &NoException<'env>) -> JavaResult<'env, i32> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_method::<Self, _, _, fn() -> i32>(self, token, "hashCode\0", ()) }
    }

    /// Create a new [`Object`](struct.Object.html) with a message.
    ///
    /// [`Object()` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#<init>())
    pub fn new(
        env: &'env JniEnv<'env>,
        token: &NoException<'env>,
    ) -> JavaResult<'env, Object<'env>> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_constructor::<Self, _, fn()>(&env, token, ()) }
    }

    /// Construct from a raw pointer. Unsafe because an invalid pointer may be passed
    /// as the argument.
    ///
    /// Unsafe because an incorrect object reference can be passed.
    #[inline(always)]
    pub unsafe fn from_raw<'a>(
        env: &'a JniEnv<'a>,
        raw_object: NonNull<jni_sys::_jobject>,
    ) -> Object<'a> {
        Object { env, raw_object }
    }
}

/// Make [`Object`](struct.Object.html)-s reference be deleted when the value is
/// [`drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html#tymethod.drop)-ed.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#deletelocalref)
impl<'env> Drop for Object<'env> {
    fn drop(&mut self) {
        // Safe because the argument is ensured to be correct references by construction.
        // DeleteLocalRef can handle nulls without any issues.
        unsafe {
            call_jni_object_method!(self, DeleteLocalRef);
        }
    }
}

impl<'env> AsRef<Object<'env>> for Object<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'env> {
        self
    }
}

impl<'env> FromObject<'env> for Object<'env> {
    #[inline(always)]
    // Actually it is safe.
    unsafe fn from_object(object: Object<'env>) -> Self {
        object
    }
}

impl JniSignature for Object<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Ljava/lang/Object;"
    }
}

/// Allow comparing [`Object`](struct.Object.html) to Java objects. Java objects are compared
/// by-reference to preserve original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for Object<'env>
where
    T: AsRef<Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        // Safe because we are not leaking the tokens anywhere.
        unsafe {
            match NoException::check_pending_exception(self.env()) {
                Err(_) => {
                    panic!("Comparing Java objects with a pending exception in the current thread")
                }
                Ok(token) => self.is_same_as(&token, other.as_ref()),
            }
        }
    }
}

fn string_or_null<'a>(string: &'a Option<std::string::String>) -> &'a str {
    string
        .as_ref()
        .map(|string| string.as_ref())
        .unwrap_or("<null>")
}

/// Allow displaying Java objects for debug purposes.
///
/// [`Object::toString`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
///
/// This is mostly a convenience for debugging. Always prefer using
/// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env> fmt::Debug for Object<'env> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // Safe because we are not leaking the tokens anywhere.
        unsafe {
            match NoException::check_pending_exception(self.env()) {
                Err(_) => {
                    // Can't call `to_string` with a pending exception.
                    write!(
                        formatter,
                        "Object {{ env: {:?}, object: {:?}, string: \
                         <can't call Object::toString string because of a pending exception in the current thread> }}",
                        self.env, self.raw_object
                    )
                }
                Ok(token) => match self.to_string(&token) {
                    Ok(string) => write!(
                        formatter,
                        "Object {{ env: {:?}, object: {:?} string: {} }}",
                        self.env,
                        self.raw_object,
                        string_or_null(&string.map(|string| string.as_string(&token)))
                    ),
                    Err(exception) => match exception.to_string(&token) {
                        Ok(message) => write!(
                            formatter,
                            "Object {{ env: {:?}, object: {:?}, string: \
                             <Object::toString threw an exception: {:?}> }}",
                            self.env,
                            self.raw_object,
                            string_or_null(&message.map(|message| message.as_string(&token)))
                        ),
                        Err(_) => write!(
                            formatter,
                            "Object {{ env: {:?}, object: {:?}, string: \
                             <Object::toString threw an exception> }}",
                            self.env, self.raw_object
                        ),
                    },
                },
            }
        }
    }
}

/// Allow displaying Java objects for debug purposes.
///
/// [`Object::toString`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
///
/// This is mostly a convenience for debugging. Always prefer using
/// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'a> Clone for Object<'a> {
    fn clone(&self) -> Self {
        // Safe because we are not leaking the tokens anywhere.
        unsafe {
            match NoException::check_pending_exception(self.env()) {
                Err(_) => {
                    panic!("Cloning a Java object with a pending exception in the current thread")
                }
                Ok(token) => self.clone_object(&token).unwrap(),
            }
        }
    }
}
