use crate::class::Class;
use crate::java_methods::JavaArgumentTuple;
use crate::java_methods::JavaMethodResult;
use crate::java_methods::JavaMethodSignature;
use crate::java_methods::ToJniTypeTuple;
use crate::jni_methods;
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
///    [`JavaClassSignature`](trait.JavaClassSignature.html)
pub trait JavaClassSignature {
    /// Return the JNI signature for `Self`.
    ///
    /// This method is not unsafe. Returning an incorrect signature will result in a panic, not any unsafe
    /// behaviour.
    fn signature() -> &'static str;
}

/// Make references to [`JavaClassSignature`](trait.JavaClassSignature.html) also implement
/// [`JavaClassSignature`](trait.JavaClassSignature.html)/
impl<T> JavaClassSignature for &'_ T
where
    T: JavaClassSignature,
{
    #[inline(always)]
    fn signature() -> &'static str {
        T::signature()
    }
}

/// Make nullable [`JavaClassSignature`](trait.JavaClassSignature.html) implement
/// [`JavaClassSignature`](trait.JavaClassSignature.html).
impl<'a, T> JavaClassSignature for Option<T>
where
    T: JavaClassSignature,
{
    #[inline(always)]
    fn signature() -> &'static str {
        T::signature()
    }
}

/// A trait for all types that are accepted as arguments to Java methods
/// or returned from native methods.
pub trait JniSignature {
    /// Return the JNI signature for `Self`.
    ///
    /// This method is not unsafe. Returning an incorrect signature will result in a panic, not any unsafe
    /// behaviour.
    fn signature() -> &'static str;
}

/// Implement [`JniSignature`](trait.JniSignature.html) for all types
/// implementing [`JavaClassSignature`](trait.JavaClassSignature.html).
impl<T> JniSignature for T
where
    T: JavaClassSignature,
{
    #[inline(always)]
    fn signature() -> &'static str {
        <T as JavaClassSignature>::signature()
    }
}

/// A trait for making Java class wrappers constructible from an [`Object`](java/lang/struct.Object.html).
///
/// See more detailed info for passing values betweed Java and rust in
/// [`JavaClassSignature`](trait.JavaClassSignature.html) documentation.
///
/// This trait is used instead of [`From<Object>`](https://doc.rust-lang.org/std/convert/trait.From.html)
/// because the construction must be unsafe, since it is possible to call it with an
/// [`Object`](java/lang/struct.Object.html) of a wrong runtime type.
pub trait FromObject<'a> {
    /// Construct `Self` from an [`Object`](java/lang/struct.Object.html).
    ///
    /// Unsafe because it's possible to pass an object of a different type.
    unsafe fn from_object(object: Object<'a>) -> Self;
}

pub trait JavaClassRef<'a>: JavaClassSignature + AsRef<Object<'a>> {}

impl<'a, T> JavaClassRef<'a> for T where T: JavaClassSignature + AsRef<Object<'a>> {}

pub trait JavaClass<'a>: JavaClassRef<'a> + FromObject<'a> + Into<Object<'a>> {}

impl<'a, T> JavaClass<'a> for T where T: JavaClassRef<'a> + FromObject<'a> + Into<Object<'a>> {}

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
    unsafe fn take_raw_object(self) -> NonNull<jni_sys::_jobject>;

    /// Call a Java method.
    ///
    /// The method has four generic parameters:
    ///  - The first one is the class of the object. It doesn't have to be the exact class,
    ///    a subclass can be passed as well. Can be inferred
    ///  - The second one is the type of the arguments tuple. Can be inferred
    ///  - The third one is the Java result type. Can be inferred
    ///  - The fourth one is the signature of the Java method. Must be specified
    ///
    /// As a result, only one generic parameter needs to be specified -- the last one.
    ///
    /// Example:
    /// ```
    /// # use rust_jni::*;
    /// # use rust_jni::java::lang::String;
    /// # use std::ptr;
    /// #
    /// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
    /// let object = String::empty(&token)?;
    /// // Safe because correct arguments are passed and correct return type specified.
    /// // See `Object::hashCode` javadoc:
    /// // https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#hashCode()
    /// let hash_code = unsafe {
    ///     object.call_method::<_, fn() -> i32>(&token, "hashCode\0", ())
    /// }?;
    /// assert_eq!(hash_code, 0);
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
    ///
    /// Note that method name string *must* be null-terminating.
    ///
    /// See more info about how to pass or return types from Java calls in [`JavaClassSignature`](trait.JavaClassSignature.html)
    /// documentation
    ///
    /// This method is unsafe because incorrect parameters can be passed to a method or incorrect return type specified.
    unsafe fn call_method<'b, A, F>(
        &self,
        token: &NoException<'a>,
        name: &str,
        arguments: A::ActualType,
    ) -> JavaResult<
        'a,
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::ResultType,
    >
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A>,
        'a: 'b;

    unsafe fn call_method_new<'b, A, F>(
        &self,
        token: &NoException<'a>,
        name: &str,
        arguments: A::ActualType,
    ) -> JavaResult<
        'a,
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::ResultType,
    >
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A>,
        'a: 'b;

    /// Call a static Java method.
    ///
    /// The method has four generic parameters:
    ///  - The first one is the class of the object. Can be inferred
    ///  - The second one is the type of the arguments tuple. Can be inferred
    ///  - The third one is the Java result type. Can be inferred
    ///  - The fourth one is the signature of the Java method. Must be specified
    ///
    /// As a result, only one generic parameter needs to be specified -- the last one.
    ///
    /// Example:
    /// ```
    /// # use rust_jni::*;
    /// # use rust_jni::java::lang::String;
    /// # use std::ptr;
    /// #
    /// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
    /// // Safe because correct arguments are passed and correct return type specified.
    /// // See `String::valueOf(int)` javadoc:
    /// // https://docs.oracle.com/javase/10/docs/api/java/lang/String.html#valueOf(int)
    /// let string_value = unsafe {
    ///     String::call_static_method::<_, fn(i32) -> String<'a>>(
    ///         &token,
    ///         "valueOf\0",
    ///         (17,),
    ///     )
    /// }
    /// .or_npe(&token)?
    /// .as_string(&token);
    /// assert_eq!(string_value, "17");
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
    ///
    /// Note that method name string must be null-terminating.
    ///
    /// See more info about how to pass or return types from Java calls in [`JavaClassSignature`](trait.JavaClassSignature.html)
    /// documentation
    ///
    /// This method is unsafe because incorrect parameters can be passed to a method or incorrect return type specified.
    unsafe fn call_static_method<'b, A, F>(
        token: &NoException<'a>,
        name: &str,
        arguments: A::ActualType,
    ) -> JavaResult<
        'a,
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::ResultType,
    >
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A>,
        'a: 'b;

    /// Call a Java constructor
    ///
    /// The method has three generic parameters:
    ///  - The first one is the class of the object
    ///  - The second one is the type of the arguments tuple. Can be inferred
    ///  - The third one is the signature of the Java method. Can be inferred
    ///
    /// As a result, only one generic parameter needs to be specified -- the class type.
    ///
    /// Example:
    /// ```
    /// # use rust_jni::*;
    /// # use rust_jni::java::lang::String;
    /// # use std::ptr;
    /// #
    /// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
    /// // Safe because correct arguments are passed.
    /// // See `String()` javadoc:
    /// // https://docs.oracle.com/javase/10/docs/api/java/lang/String.html#<init>()
    /// let empty_string = unsafe {
    ///     String::call_constructor::<_, fn()>(&token, ())
    /// }?
    /// .as_string(&token);
    /// assert_eq!(empty_string, "");
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
    ///
    /// See more info about how to pass or return types from Java calls in [`JavaClassSignature`](trait.JavaClassSignature.html)
    /// documentation
    ///
    /// This method is unsafe because incorrect parameters can be passed to a method.
    unsafe fn call_constructor<'b, A, F>(
        token: &NoException<'a>,
        arguments: A::ActualType,
    ) -> JavaResult<'a, Self>
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A, Out = ()>,
        Self: Sized,
        'a: 'b;
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
    unsafe fn take_raw_object(self) -> NonNull<jni_sys::_jobject> {
        Object::take_raw_object(self)
    }

    #[inline(always)]
    unsafe fn call_method<'b, A, F>(
        &self,
        token: &NoException<'a>,
        name: &str,
        arguments: A::ActualType,
    ) -> JavaResult<
        'a,
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::ResultType,
    >
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A>,
        'a: 'b,
    {
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::call_method::<
            Self,
            <A::ActualType as ToJniTypeTuple>::JniType,
        >(
            self,
            token,
            name,
            &F::method_signature(),
            ToJniTypeTuple::to_jni(&arguments),
        )
    }

    #[inline(always)]
    unsafe fn call_method_new<'b, A, F>(
        &self,
        token: &NoException<'a>,
        name: &str,
        arguments: A::ActualType,
    ) -> JavaResult<
        'a,
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::ResultType,
    >
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A>,
        'a: 'b,
    {
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::call_method::<
            Self,
            <A::ActualType as ToJniTypeTuple>::JniType,
        >(
            self,
            token,
            name,
            &F::method_signature(),
            ToJniTypeTuple::to_jni(&arguments),
        )
    }

    #[inline(always)]
    unsafe fn call_static_method<'b, A, F>(
        token: &NoException<'a>,
        name: &str,
        arguments: A::ActualType,
    ) -> JavaResult<
        'a,
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::ResultType,
    >
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A>,
        'a: 'b,
    {
        <<F as JavaMethodSignature<'b, 'a, A>>::Out as JavaMethodResult<'a>>::call_static_method::<
            Self,
            <A::ActualType as ToJniTypeTuple>::JniType,
        >(
            token,
            name,
            &F::method_signature(),
            ToJniTypeTuple::to_jni(&arguments),
        )
    }

    #[inline(always)]
    unsafe fn call_constructor<'b, A, F>(
        token: &NoException<'a>,
        arguments: A::ActualType,
    ) -> JavaResult<'a, Self>
    where
        A: JavaArgumentTuple<'b, 'a>,
        F: JavaMethodSignature<'b, 'a, A, Out = ()>,
        Self: Sized,
        'a: 'b,
    {
        let class = Self::class(token)?;
        let result = jni_methods::call_constructor(
            &class,
            token,
            &F::method_signature(),
            ToJniTypeTuple::to_jni(&arguments),
        )?;
        Ok(Self::from_object(Object::from_raw(token.env(), result)))
    }
}

#[inline(always)]
pub fn find_class<'a, T: JavaClass<'a>>(token: &NoException<'a>) -> JavaResult<'a, Class<'a>> {
    let signature = T::signature();
    // Class signatures are of the form "L${CLASS_NAME};", so to get the class name
    // we remove the first and the last character.
    Class::find(token, &signature[1..signature.len() - 1])
}
