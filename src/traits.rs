use crate::env::JniEnv;
use crate::jni_types::private::{JniArgumentType, JniType};
use crate::object::Object;
use jni_sys;
use std::ptr;

pub trait JavaClassType<'env> {
    /// Compute the signature for this Java type.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    fn signature() -> &'static str;

    /// Construct a value from a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    fn from_object(value: Object<'env>) -> Self;
}

/// A trait for mapping types to their JNI types.
/// This trait has to be implemented for all types that need to be passed as arguments
/// to Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
#[doc(hidden)]
pub trait ToJni {
    type JniType: JniArgumentType;
    fn signature() -> &'static str;
    /// Map the value to a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    unsafe fn to_jni(&self) -> Self::JniType;
}

impl<'a, T> ToJni for Option<T>
where
    T: ToJni<JniType = jni_sys::jobject> + JavaClassType<'a>,
{
    type JniType = jni_sys::jobject;

    fn signature() -> &'static str {
        <T as JavaClassType>::signature()
    }
    /// Map the value to a JNI type value.
    unsafe fn to_jni(&self) -> jni_sys::jobject {
        self.as_ref()
            .map(|value| value.to_jni())
            .unwrap_or(ptr::null_mut())
    }
}

/// Make references mappable to JNI types of their referenced types.
impl<'a, T> ToJni for &'a T
where
    T: ToJni,
{
    type JniType = T::JniType;

    fn signature() -> &'static str {
        T::signature()
    }

    unsafe fn to_jni(&self) -> Self::JniType {
        T::to_jni(self)
    }
}

/// A trait for constructing types from their JNI types and [`JniEnv`](struct.JniEnv.html)
/// references. This trait has to be implemented for all types that the user wants to pass
/// return from Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
#[doc(hidden)]
pub trait FromJni<'env> {
    /// The corresponding JNI type.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    type JniType: JniType;

    /// Compute the signature for this Java type.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    fn signature() -> &'static str;
    /// Construct a value from a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    unsafe fn from_jni(env: &'env JniEnv<'env>, value: Self::JniType) -> Self;
}

impl<'env, T> FromJni<'env> for T
where
    T: JavaClassType<'env>,
{
    type JniType = jni_sys::jobject;

    fn signature() -> &'static str {
        <Self as JavaClassType>::signature()
    }

    unsafe fn from_jni(env: &'env JniEnv<'env>, value: jni_sys::jobject) -> Self {
        <Self as JavaClassType>::from_object(Object::from_raw(env, value))
    }
}

pub(crate) mod private {

    /// A trait that represents Rust function types that are mappable to Java function types.
    /// This trait is separate from `JavaType` because this one doesn't need to be exposed
    /// in the public crate API.
    ///
    /// THIS TRAIT SHOULD NOT BE USED MANUALLY.
    // TODO: reimplement it in a way that it returns `&'static str`.
    // `concat!` doesn't acceps arbitrary expressions of type `&'static str`, so it can't be
    // implemented that way today.
    pub trait JavaMethodSignature<In, Out> {
        /// Get the method's JNI signature.
        ///
        /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
        fn __signature() -> std::string::String;
    }
}

/// A trait for casting Java object types to their superclasses.
pub trait Cast<'env, As: Cast<'env, Object<'env>>>: JavaClassType<'env> {
    /// Cast the object to itself or one of it's superclasses.
    ///
    /// Doesn't actually convert anything, the result is just the same object
    /// interpreted as one of it's superclasses.
    fn cast<'a>(&'a self) -> &'a As;
}
