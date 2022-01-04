use crate::env::JniEnv;
use crate::java_class::JavaClassExt;
use crate::java_class::{FromObject, JavaClassSignature};
use crate::java_string::*;
use crate::jni_bool;
use crate::object::Object;
use crate::result::JavaResult;
use crate::string::String;
use crate::token::{CallOutcome, NoException};
use jni_sys;
use std::os::raw::c_char;
use std::ptr::{self, NonNull};

include!("call_jni_method.rs");

/// A type representing a Java
/// [`Class`](https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html).
// TODO: examples.
#[derive(Debug, Clone)]
pub struct Class<'env> {
    object: Object<'env>,
}

impl<'env> Class<'env> {
    /// Find an existing Java class by it's name. The name is a fully qualified class or array
    /// type name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#findclass)
    pub fn find<'a>(token: &NoException<'a>, class_name: &str) -> JavaResult<'a, Class<'a>> {
        let class_name = to_java_string(class_name);
        // Safe because the arguments are correct and because `FindClass` throws an exception
        // before returning `null`.
        let raw_class = unsafe {
            call_nullable_jni_method!(token, FindClass, class_name.as_ptr() as *const c_char)
        }?;
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(token.env(), raw_class) })
    }

    /// Define a new Java class from a `.class` file contents.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#defineclass)
    pub fn define<'a>(bytes: &[u8], token: &NoException<'a>) -> JavaResult<'a, Class<'a>> {
        // Safe because the arguments are correct and because `DefineClass` throws an exception
        // before returning `null`.
        let raw_class = unsafe {
            call_nullable_jni_method!(
                token,
                DefineClass,
                ptr::null() as *const c_char,
                ptr::null_mut() as jni_sys::jobject,
                bytes.as_ptr() as *const jni_sys::jbyte,
                bytes.len() as jni_sys::jsize
            )?
        };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(token.env(), raw_class) })
    }

    /// Get the parent class of this class. Will return
    /// [`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None) for the
    /// [`Object`](struct.Object.html) class or any interface.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getsuperclass)
    pub fn parent(&self, token: &NoException) -> Option<Class<'env>> {
        // Safe because the argument is ensured to be correct references by construction.
        let raw_java_class = unsafe { call_jni_object_method!(token, self, GetSuperclass) };
        NonNull::new(raw_java_class).map(|raw_java_class| {
            // Safe because the argument is ensured to be a correct reference.
            unsafe { Self::from_raw(self.env(), raw_java_class) }
        })
    }

    /// Check if this class is a subtype of the other class.
    ///
    /// In Java a class is a subtype of the other class if that other class is a direct or
    /// an indirect parent of this class or an interface this class or any it's parent is
    /// implementing.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#isassignablefrom)
    pub fn is_subtype_of<'a>(&self, token: &NoException, class: impl AsRef<Class<'a>>) -> bool {
        // Safe because arguments are ensured to be the correct by construction.
        let assignable = unsafe {
            call_jni_object_method!(
                token,
                self,
                IsAssignableFrom,
                class.as_ref().raw_object().as_ptr() as jni_sys::jclass
            )
        };
        jni_bool::to_rust(assignable)
    }

    /// Get class name
    ///
    /// [`Class::getName` javadoc](https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/lang/Class.html#getName())
    pub fn get_name(&self, token: &NoException<'env>) -> JavaResult<'env, Option<String<'env>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe { self.call_method::<_, fn() -> String<'env>>(token, "getName\0", ()) }
    }

    /// Unsafe because the argument mught not be a valid class reference.
    #[inline(always)]
    pub(crate) unsafe fn from_raw<'a>(
        env: &'a JniEnv<'a>,
        raw_class: NonNull<jni_sys::_jobject>,
    ) -> Class<'a> {
        Class {
            object: Object::from_raw(env, raw_class.cast()),
        }
    }
}

/// Allow [`Class`](struct.Class.html) to be used in place of an [`Object`](struct.Object.html).
impl<'env> ::std::ops::Deref for Class<'env> {
    type Target = Object<'env>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'env> AsRef<Object<'env>> for Class<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'env> {
        &self.object
    }
}

impl<'env> AsRef<Class<'env>> for Class<'env> {
    #[inline(always)]
    fn as_ref(&self) -> &Class<'env> {
        &*self
    }
}

impl<'a> Into<Object<'a>> for Class<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'env> FromObject<'env> for Class<'env> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'env>) -> Self {
        Self { object }
    }
}

impl JavaClassSignature for Class<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Ljava/lang/Class;"
    }
}

/// Allow comparing [`Class`](struct.Class.html)
/// to Java objects. Java objects are compared by-reference to preserve
/// original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for Class<'env>
where
    T: AsRef<Object<'env>>,
{
    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        Object::as_ref(self).eq(other.as_ref())
    }
}
