use crate::class::Class;
use crate::classes::null_pointer_exception::NullPointerException;
use crate::java_methods::FromObject;
use crate::java_methods::JniSignature;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use std::ptr::NonNull;

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

/// Extension trait that adds common helper methods for working with
/// nullable objects to [`JavaClass`](trait.JavaClass.html) types.
pub trait NullableJavaClassExt<'a, R> {
    /// Convert [`Option<T: JavaClass>`](trait.JavaClass.html) into
    /// [`T: JavaClass`](trait.JavaClass.html), returning a
    /// [`NullPointerException`](java/lang/struct.NullPointerException.html)
    /// on [`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    ///
    /// This follows the standard Java semantics of throwing a
    /// [`NullPointerException`](https://docs.oracle.com/javase/10/docs/api/java/lang/NullPointerException.html)
    /// when a method is called on a `null`.
    ///
    /// Example:
    /// ```
    /// # use rust_jni::*;
    /// # use rust_jni::java::lang::Object;
    /// # use std::ptr;
    /// #
    /// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
    /// let object_class = Object::new(&token)?
    ///     .to_string(&token)
    ///     .or_npe(&token)?
    ///     .class(&token)
    ///     .parent(&token)
    ///     .as_ref()
    ///     .or_npe(&token)?;
    /// # Ok(token)
    /// # }
    /// #
    /// # #[cfg(feature = "libjvm")]
    /// # fn main() {
    /// #     let init_arguments = InitArguments::default();
    /// #     let vm = JavaVM::create(&init_arguments).unwrap();
    /// #     let _ = vm.with_attached(
    /// #        &AttachArguments::new(init_arguments.version()),
    /// #        |token: NoException| {
    /// #            ((), jni_main(token).unwrap())
    /// #        },
    /// #     );
    /// # }
    /// #
    /// # #[cfg(not(feature = "libjvm"))]
    /// # fn main() {}
    /// ```
    fn or_npe(self, token: &NoException<'a>) -> JavaResult<'a, R>;
}

/// Add nullable object helper methods from [`NullableJavaClassExt`](trait.NullableJavaClassExt.html)
/// to [`Option<T: JavaClass>`](type.JavaResult.html).
impl<'a, R> NullableJavaClassExt<'a, R> for Option<R>
where
    R: JavaClassRef<'a>,
{
    #[inline(always)]
    fn or_npe(self, token: &NoException<'a>) -> JavaResult<'a, R> {
        match self {
            Some(value) => Ok(value),
            None => {
                let npe = NullPointerException::new(token)?;
                Err(npe.into())
            }
        }
    }
}

#[inline(always)]
pub fn find_class<'a, T: JavaClassRef<'a>>(token: &NoException<'a>) -> JavaResult<'a, Class<'a>> {
    let signature = T::signature();
    // Class signatures are of the form "L${CLASS_NAME};", so to get the class name
    // we remove the first and the last character.
    Class::find(token, &signature[1..signature.len() - 1])
}
