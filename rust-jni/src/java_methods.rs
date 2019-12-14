use crate::java_class::find_class;
use crate::java_class::JavaClass;
use crate::java_class::JavaClassExt;
use crate::java_class::JavaClassRef;
use crate::java_class::JniSignature;
use crate::jni_methods;
use crate::jni_types::private::JniArgumentType;
use crate::jni_types::private::JniArgumentTypeTuple;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use core::ptr::{self, NonNull};

impl<T> JniSignature for &'_ T
where
    T: JniSignature,
{
    #[inline(always)]
    fn signature() -> &'static str {
        T::signature()
    }
}

/// A trait that needs to be implemented for a type that needs to be passed to Java.
///
/// See more detailed info for passing values betweed Java and rust in
/// [`JniSignature`](trait.JniSignature.html) documentation.
pub trait JavaArgumentType: JniSignature {
    type JniType: JniArgumentType;

    // Unsafe because it returns raw pointers to Java objects.
    unsafe fn to_jni(&self) -> Self::JniType;
}

impl<'a, T> JavaArgumentType for T
where
    T: JavaClassRef<'a>,
{
    type JniType = jni_sys::jobject;

    #[inline(always)]
    unsafe fn to_jni(&self) -> Self::JniType {
        self.as_ref().raw_object().as_ptr()
    }
}

impl<'a, T> JniSignature for Option<T>
where
    T: JavaClassRef<'a>,
{
    #[inline(always)]
    fn signature() -> &'static str {
        T::signature()
    }
}

impl<'a, T> JavaArgumentType for Option<T>
where
    T: JavaClassRef<'a>,
{
    type JniType = jni_sys::jobject;

    #[inline(always)]
    unsafe fn to_jni(&self) -> Self::JniType {
        self.as_ref()
            .map_or(ptr::null_mut(), |value| value.to_jni())
    }
}

/// A helper trait to allow accepting as many types
/// as possible as method arguments in place of Java objects.
///
/// Currently supports:
///  - `T: JavaClassRef`
///  - `Option<T: JavaClassRef>`
///
/// Supporting both `T` and `Option<T>` essentially allow passing nullable class references.
pub trait JavaObjectArgument<'a, AT>
where
    AT: JavaClassRef<'a>,
{
    fn as_argument<'r>(&'r self) -> Option<&'r AT>
    where
        'a: 'r;
}

impl<'a, AT, T> JavaObjectArgument<'a, AT> for T
where
    T: JavaClassRef<'a> + AsRef<AT>,
    AT: JavaClassRef<'a>,
{
    #[inline(always)]
    fn as_argument<'r>(&'r self) -> Option<&'r AT>
    where
        'a: 'r,
    {
        Some(self.as_ref())
    }
}

impl<'a, AT, T> JavaObjectArgument<'a, AT> for Option<T>
where
    T: JavaClassRef<'a> + AsRef<AT>,
    AT: JavaClassRef<'a>,
{
    #[inline(always)]
    fn as_argument<'r>(&'r self) -> Option<&'r AT>
    where
        'a: 'r,
    {
        self.as_ref().map(|value| value.as_ref())
    }
}

pub trait JavaArgumentTuple {
    type JniType: JniArgumentTypeTuple;

    // Unsafe because it returns raw pointers to Java objects.
    unsafe fn to_jni(&self) -> Self::JniType;
}

pub trait JavaMethodSignature<In, Out>
where
    In: JavaArgumentTuple,
{
    fn method_signature() -> std::string::String;
}

macro_rules! braces {
    ($name:ident) => {
        "{}"
    };
}

macro_rules! peel_java_argument_type_impls {
    () => ();
    ($type:ident, $($other:ident,)*) => (java_argument_type_impls! { $($other,)* });
}

macro_rules! java_argument_type_impls {
    ( $($type:ident,)*) => (
        impl<'a, $($type),*> JavaArgumentTuple for ($($type,)*)
        where
            $($type: JavaArgumentType,)*
        {
            type JniType = ($($type::JniType,)*);

            #[inline(always)]
            unsafe fn to_jni(&self) -> Self::JniType {
                #[allow(non_snake_case)]
                let ($($type,)*) = self;
                ($($type.to_jni(),)*)
            }
        }

        impl<'a, $($type,)* Out, F> JavaMethodSignature<($($type,)*), Out> for F
            where
                $($type: JavaArgumentType,)*
                Out: JniSignature,
                F: FnOnce($($type,)*) -> Out + ?Sized,
        {
            #[inline(always)]
            fn method_signature() -> std::string::String {
                format!(
                    concat!("(", $(braces!($type), )* "){}\0"),
                    $(<$type as JniSignature>::signature(),)*
                    <Out as JniSignature>::signature(),
                )
            }
        }

        peel_java_argument_type_impls! { $($type,)* }
    );
}

java_argument_type_impls! {
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11,
}

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
///     call_method::<_, _, _, fn() -> i32>(&object, &token, "hashCode\0", ())
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
/// See more info about how to pass or return types from Java calls in [`JniSignature`](trait.JniSignature.html)
/// documentation
///
/// This method is unsafe because incorrect parameters can be passed to a method or incorrect return type specified.
pub unsafe fn call_method<'a, T, A, R, F>(
    object: &T,
    token: &NoException<'a>,
    name: &str,
    arguments: A,
) -> JavaResult<'a, R::ResultType>
where
    T: JavaClassRef<'a>,
    A: JavaArgumentTuple,
    R: JavaMethodResult<'a>,
    F: JavaMethodSignature<A, R>,
{
    R::call_method::<T, A>(object, token, name, &F::method_signature(), arguments)
}

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
///     call_static_method::<String<'a>, _, _, fn(i32) -> String<'a>>(
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
/// See more info about how to pass or return types from Java calls in [`JniSignature`](trait.JniSignature.html)
/// documentation
///
/// This method is unsafe because incorrect parameters can be passed to a method or incorrect return type specified.
pub unsafe fn call_static_method<'a, T, A, R, F>(
    token: &NoException<'a>,
    name: &str,
    arguments: A,
) -> JavaResult<'a, R::ResultType>
where
    T: JavaClassRef<'a>,
    A: JavaArgumentTuple,
    R: JavaMethodResult<'a>,
    F: JavaMethodSignature<A, R>,
{
    R::call_static_method::<T, A>(token, name, &F::method_signature(), arguments)
}

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
///     call_constructor::<String<'a>, _, fn()>(&token, ())
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
/// See more info about how to pass or return types from Java calls in [`JniSignature`](trait.JniSignature.html)
/// documentation
///
/// This method is unsafe because incorrect parameters can be passed to a method.
pub unsafe fn call_constructor<'a, R, A, F>(
    token: &NoException<'a>,
    arguments: A,
) -> JavaResult<'a, R>
where
    A: JavaArgumentTuple,
    R: JavaClass<'a>,
    F: JavaMethodSignature<A, ()>,
{
    let class = R::class(token)?;
    let result = jni_methods::call_constructor(
        &class,
        token,
        &F::method_signature(),
        JavaArgumentTuple::to_jni(&arguments),
    )?;
    Ok(R::from_object(Object::from_raw(token.env(), result)))
}

pub trait JavaMethodResult<'a> {
    type JniType;
    type ResultType: 'a;

    unsafe fn call_method<T, A>(
        object: &T,
        token: &NoException<'a>,
        name: &str,
        signature: &str,
        arguments: A,
    ) -> JavaResult<'a, Self::ResultType>
    where
        T: JavaClassRef<'a>,
        A: JavaArgumentTuple;

    unsafe fn call_static_method<T, A>(
        token: &NoException<'a>,
        name: &str,
        signature: &str,
        arguments: A,
    ) -> JavaResult<'a, Self::ResultType>
    where
        T: JavaClassRef<'a>,
        A: JavaArgumentTuple;
}

impl<'a, S> JavaMethodResult<'a> for S
where
    S: JavaClass<'a> + 'a,
{
    type JniType = Option<NonNull<jni_sys::_jobject>>;
    type ResultType = Option<Self>;

    #[inline(always)]
    unsafe fn call_method<T, A>(
        object: &T,
        token: &NoException<'a>,
        name: &str,
        signature: &str,
        arguments: A,
    ) -> JavaResult<'a, Self::ResultType>
    where
        T: JavaClassRef<'a>,
        A: JavaArgumentTuple,
    {
        let result = jni_methods::call_object_method(
            object.as_ref(),
            token,
            name,
            signature,
            JavaArgumentTuple::to_jni(&arguments),
        )?;
        Ok(result.map(
            #[inline(always)]
            |result| Self::from_object(Object::from_raw(object.as_ref().env(), result)),
        ))
    }

    #[inline(always)]
    unsafe fn call_static_method<T, A>(
        token: &NoException<'a>,
        name: &str,
        signature: &str,
        arguments: A,
    ) -> JavaResult<'a, Self::ResultType>
    where
        T: JavaClassRef<'a>,
        A: JavaArgumentTuple,
    {
        let class = find_class::<T>(token)?;
        let result = jni_methods::call_static_object_method(
            &class,
            token,
            name,
            signature,
            JavaArgumentTuple::to_jni(&arguments),
        )?;
        Ok(result.map(
            #[inline(always)]
            |result| Self::from_object(Object::from_raw(token.env(), result)),
        ))
    }
}
