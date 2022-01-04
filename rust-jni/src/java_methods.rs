use crate::java_class::find_class;
use crate::java_class::JavaClass;
use crate::java_class::JavaClassRef;
use crate::java_class::JniSignature;
use crate::jni_methods;
use crate::jni_types::private::JniArgumentType;
use crate::jni_types::private::JniArgumentTypeTuple;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use core::ptr;

/// A helper trait to allow accepting as many types
/// as possible as method arguments in place of Java objects.
///
/// Currently supports:
///  - `T: JavaClassRef`
///  - `Option<T: JavaClassRef>`
///
/// Supporting both `T` and `Option<T>` essentially allow passing nullable class references.
pub trait JavaObjectArgument<AT> {
    fn as_argument<'b>(&'b self) -> Option<&'b AT>;
}

impl<'a, AT, T> JavaObjectArgument<AT> for T
where
    T: JavaClassRef<'a> + AsRef<AT>,
    AT: JavaClass<'a>,
{
    #[inline(always)]
    fn as_argument<'b>(&'b self) -> Option<&'b AT> {
        Some(self.as_ref())
    }
}

impl<'a, AT, T> JavaObjectArgument<AT> for Option<T>
where
    T: JavaObjectArgument<AT>,
{
    #[inline(always)]
    fn as_argument<'b>(&'b self) -> Option<&'b AT> {
        self.as_ref().and_then(|value| value.as_argument())
    }
}

pub trait ToJniType {
    type JniType: JniArgumentType;

    // Unsafe because it returns raw pointers to Java objects.
    unsafe fn to_jni(&self) -> Self::JniType;
}

impl<'a, 'this: 'a, T> ToJniType for Option<&'a T>
where
    T: JavaClass<'this> + 'a,
{
    type JniType = jni_sys::jobject;

    #[inline(always)]
    unsafe fn to_jni(&self) -> Self::JniType {
        self.map_or(ptr::null_mut(), |value| {
            value.as_ref().raw_object().as_ptr()
        })
    }
}

pub trait ToJniTypeTuple {
    type JniType: JniArgumentTypeTuple;

    // Unsafe because it returns raw pointers to Java objects.
    unsafe fn to_jni(&self) -> Self::JniType;
}

/// A trait that needs to be implemented for a type that needs to be passed to Java.
///
/// See more detailed info for passing values betweed Java and rust in
/// [`JavaClassSignature`](trait.JavaClassSignature.html) documentation.
pub trait JavaArgumentType<'a, 'this: 'a>: JniSignature {
    type ActualType: ToJniType;
}

/// Make Java class wrappers passable to Java methods as arguments.
impl<'a, 'this: 'a, T> JavaArgumentType<'a, 'this> for &'a T
where
    T: JavaClass<'this> + 'a,
{
    type ActualType = Option<&'a T>;
}

/// Make nullable Java class wrappers passable to Java methods as arguments.
impl<'a, 'this: 'a, T> JavaArgumentType<'a, 'this> for Option<&'a T>
where
    T: JavaClass<'this> + 'a,
{
    type ActualType = Option<&'a T>;
}

pub trait JavaArgumentTuple<'a, 'this: 'a> {
    type ActualType: ToJniTypeTuple;
}

pub trait JavaMethodSignature<'a, 'this: 'a, In>
where
    In: JavaArgumentTuple<'a, 'this>,
{
    type Out: JavaMethodResult<'this>;

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
        impl<$($type),*> ToJniTypeTuple for ($($type,)*)
        where
            $($type: ToJniType,)*
        {
            type JniType = ($($type::JniType,)*);

            #[inline(always)]
            unsafe fn to_jni(&self) -> Self::JniType {
                #[allow(non_snake_case)]
                let ($($type,)*) = self;
                ($($type.to_jni(),)*)
            }
        }

        impl<'a, 'this: 'a, $($type),*> JavaArgumentTuple<'a, 'this> for ($($type,)*)
        where
            $($type: JavaArgumentType<'a, 'this>,)*
        {
            type ActualType = ($($type::ActualType,)*);
        }

        impl<'a, 'this: 'a, $($type,)* Out, F> JavaMethodSignature<'a, 'this, ($($type,)*)> for F
            where
                $($type: JavaArgumentType<'a, 'this>,)*
                Out: JavaMethodResult<'this>,
                F: FnOnce($($type,)*) -> Out + ?Sized,
        {
            type Out = Out;

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

pub trait JavaMethodResult<'a>: JniSignature {
    type ResultType;

    unsafe fn call_method<T, A>(
        object: &T,
        token: &NoException<'a>,
        name: &str,
        signature: &str,
        arguments: A,
    ) -> JavaResult<'a, Self::ResultType>
    where
        T: JavaClass<'a>,
        A: JniArgumentTypeTuple;

    unsafe fn call_static_method<T, A>(
        token: &NoException<'a>,
        name: &str,
        signature: &str,
        arguments: A,
    ) -> JavaResult<'a, Self::ResultType>
    where
        T: JavaClass<'a>,
        A: JniArgumentTypeTuple;
}

impl<'a, S> JavaMethodResult<'a> for S
where
    S: JavaClass<'a>,
{
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
        T: JavaClass<'a>,
        A: JniArgumentTypeTuple,
    {
        let result =
            jni_methods::call_object_method(object.as_ref(), token, name, signature, arguments)?;
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
        T: JavaClass<'a>,
        A: JniArgumentTypeTuple,
    {
        let class = find_class::<T>(token)?;
        let result =
            jni_methods::call_static_object_method(&class, token, name, signature, arguments)?;
        Ok(result.map(
            #[inline(always)]
            |result| Self::from_object(Object::from_raw(token.env(), result)),
        ))
    }
}
