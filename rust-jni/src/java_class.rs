use crate::class::Class;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use std::ptr::NonNull;

/// A trait to be implemented by all types that can be passed or returned from JNI.
///
/// To pass a type to Java it needs to:
///  1. Be convertible into a type implementing the `JniType` trait: implement `JavaArgumentType` trait
///  2. Provide a JNI signature (see
///     [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/types.html#type-signatures)
///     for more context).
///
/// To return a type from Java a type also needs to:
///  3. Be convertible from a type implementing the `JniType` trait: implement `JavaMethodResult` trait
///
/// [`rust-jni`](index.html) implements all three conditions for for primitive types that can be passed to JNI.
///
/// Implementing those conditions for Java class wrappers requires cooperation with the wrappers author.
/// [`Object`](java/lang/struct.Object.html) is convertible to and from [`jobject`](../jni_sys/type.jobject.html)
/// which implements the `JniType` trait. So for Java class wrappers the conditions above translate into:
///  1. Be convertible into [`Object`](java/lang/struct.Object.html)
///  2. Provide a JNI signature. For Java classes the signature is `L${CLASS_PATH};`
///  3. Be constructable from [`Object`](java/lang/struct.Object.html)
///
///  - To make a Java class wrapper convertible to [`Object`](java/lang/struct.Object.html) author of the wrapper
///    needs to implement [`AsRef<Object>`](https://doc.rust-lang.org/std/convert/trait.AsRef.html) for it
///  - To make a Java class wrapper constructable from [`Object`](java/lang/struct.Object.html) author of the wrapper
///    needs to implement [`FromObject`](trait.FromObject.html) for it
///  - To provide the JNI signature for a Java class wrapper author needs to implement
///    [`JniSignature`](trait.JniSignature.html)
pub trait JniSignature {
    /// Return the JNI signature for `Self`.
    ///
    /// This method is not unsafe. Returning an incorrect signature will result in a panic, not any unsafe
    /// behaviour.
    fn signature() -> &'static str;
}

/// A trait for making Java class wrappers constructible from an [`Object`](java/lang/struct.Object.html).
///
/// See more detailed info for passing values betweed Java and rust in
/// [`JniSignature`](trait.JniSignature.html) documentation.
pub trait FromObject<'a> {
    /// Construct `Self` from an [`Object`](java/lang/struct.Object.html).
    ///
    /// Unsafe because it's possible to pass an object of a different type.
    unsafe fn from_object(object: Object<'a>) -> Self;
}

pub trait JavaClassRef<'a>: JniSignature + AsRef<Object<'a>> {}

impl<'a, T> JavaClassRef<'a> for T where T: JniSignature + AsRef<Object<'a>> {}

pub trait JavaClass<'a>: JavaClassRef<'a> + FromObject<'a> {}

impl<'a, T> JavaClass<'a> for T where T: JavaClassRef<'a> + FromObject<'a> {}

/// Trait with additional methods on Java class wrappers.
pub trait JavaClassExt<'a> {
    /// Clone the object. This is not a deep clone of the Java object,
    /// but a Rust-like clone of the value. Since Java objects are reference counted, this will
    /// increment the reference count.
    ///
    /// This method has a different signature from the one in the
    /// [`Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html) trait because
    /// cloning a Java object is only safe when there is no pending exception and because
    /// cloning a java object cat throw an exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newlocalref)
    fn clone_object(&self, token: &NoException<'a>) -> JavaResult<'a, Self>
    where
        Self: std::marker::Sized;

    /// Get the [`Class`](java/lang/struct.Class.html) for the wrapper type.
    ///
    /// Calls [`Class::find`](java/lang/struct.Class.html#method.find) with the correct
    /// type signature.
    fn class(token: &NoException<'a>) -> JavaResult<'a, Class<'a>>;

    /// Get the raw object pointer with ownership transfer.
    ///
    /// The caller is responsible for managing the Java object's lifecycle ofter calling this.
    ///
    /// This function provides low-level access to the Java object and thus is unsafe.
    unsafe fn take_raw_object(self) -> NonNull<jni_sys::_jobject>
    where
        Self: Into<Object<'a>>;
}

impl<'a, T> JavaClassExt<'a> for T
where
    T: JavaClass<'a>,
{
    #[inline(always)]
    fn clone_object(&self, token: &NoException<'a>) -> JavaResult<'a, Self> {
        let cloned = self.as_ref().clone_object(token)?;
        // We know it's safe as originally we are cloning Self.
        Ok(unsafe { Self::from_object(cloned) })
    }

    #[inline(always)]
    fn class(token: &NoException<'a>) -> JavaResult<'a, Class<'a>> {
        find_class::<Self>(token)
    }

    #[inline(always)]
    unsafe fn take_raw_object(self) -> NonNull<jni_sys::_jobject>
    where
        Self: Into<Object<'a>>,
    {
        Object::take_raw_object(self)
    }
}

#[inline(always)]
pub fn find_class<'a, T: JavaClassRef<'a>>(token: &NoException<'a>) -> JavaResult<'a, Class<'a>> {
    let signature = T::signature();
    // Class signatures are of the form "L${CLASS_NAME};", so to get the class name
    // we remove the first and the last character.
    Class::find(token, &signature[1..signature.len() - 1])
}
