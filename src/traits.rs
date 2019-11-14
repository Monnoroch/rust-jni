use crate::class::Class;
use crate::env::JniEnv;
use crate::object::Object;
use crate::primitives::ToJniTuple;
use jni_sys;
use std;

/// A trait that represents a JNI type. It's implemented for all JNI primitive types
/// and [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
/// Implements Java method calls and provides the default value for this JNI type.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented for classes by generated code.
#[doc(hidden)]
pub trait JniType {
    fn default() -> Self;

    unsafe fn call_method<In: ToJniTuple>(
        object: &Object,
        method_id: jni_sys::jmethodID,
        arguments: In,
    ) -> Self;

    unsafe fn call_static_method<In: ToJniTuple>(
        class: &Class,
        method_id: jni_sys::jmethodID,
        arguments: In,
    ) -> Self;
}

/// A trait that represents JNI types that can be passed as arguments to JNI functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
#[doc(hidden)]
pub trait JniArgumentType: JniType {}

/// A trait that represents Rust types that are mappable to JNI types.
/// This trait has to be implemented for all types that need to be passed as arguments
/// to or returned from Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
pub trait JavaType {
    /// The corresponding JNI type.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    type __JniType: JniType;

    /// Compute the signature for this Java type.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    #[doc(hidden)]
    fn __signature() -> &'static str;
}

/// A trait for mapping types to their JNI types.
/// This trait has to be implemented for all types that need to be passed as arguments
/// to Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
#[doc(hidden)]
pub trait ToJni: JavaType {
    /// Map the value to a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    unsafe fn __to_jni(&self) -> Self::__JniType;
}

/// A trait for constructing types from their JNI types and [`JniEnv`](struct.JniEnv.html)
/// references. This trait has to be implemented for all types that the user wants to pass
/// return from Java functions.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
///
/// This trait should only be implemented and used by generated code.
#[doc(hidden)]
pub trait FromJni<'env>: JavaType {
    /// Construct a value from a JNI type value.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    ///
    /// Should only be implemented and used by generated code.
    unsafe fn __from_jni(env: &'env JniEnv<'env>, value: Self::__JniType) -> Self;
}

/// Make references mappable to JNI types of their referenced types.
impl<'a, T> JavaType for &'a T
where
    T: JavaType + ?Sized,
{
    #[doc(hidden)]
    type __JniType = T::__JniType;

    #[doc(hidden)]
    fn __signature() -> &'static str {
        T::__signature()
    }
}

/// Make references mappable from JNI types of their referenced types.
impl<'a, T> ToJni for &'a T
where
    T: ToJni,
{
    unsafe fn __to_jni(&self) -> Self::__JniType {
        T::__to_jni(self)
    }
}

/// A trait that represents Rust function types that are mappable to Java function types.
/// This trait is separate from `JavaType` because this one doesn't need to be exposed
/// in the public crate API.
///
/// THIS TRAIT SHOULD NOT BE USED MANUALLY.
// TODO: reimplement it in a way that it returns `&'static str`.
// `concat!` doesn't acceps arbitrary expressions of type `&'static str`, so it can't be
// implemented that way today.
#[doc(hidden)]
pub trait JavaMethodSignature<In: ?Sized, Out: ?Sized> {
    /// Get the method's JNI signature.
    ///
    /// THIS METHOD SHOULD NOT BE CALLED MANUALLY.
    fn __signature() -> std::string::String;
}

/// A trait for casting Java object types to their superclasses.
pub trait Cast<'env, As: Cast<'env, Object<'env>>>:
    JavaType<__JniType = jni_sys::jobject> + ToJni + FromJni<'env>
{
    /// Cast the object to itself or one of it's superclasses.
    ///
    /// Doesn't actually convert anything, the result is just the same object
    /// interpreted as one of it's superclasses.
    fn cast<'a>(&'a self) -> &'a As;
}
