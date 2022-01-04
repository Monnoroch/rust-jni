use crate::class::Class;
use crate::env::JniEnv;
use crate::error::JniError;
use crate::java_class::FromObject;
use crate::java_class::JavaClass;
use crate::java_string::to_java_string_null_terminated;
use crate::jni_types::private::JniNativeArgumentType;
use crate::jni_types::private::JniType;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use crate::vm::JavaVMRef;
use jni_sys;
use std::mem;
use std::mem::ManuallyDrop;
use std::panic;
use std::ptr::{self, NonNull};

/// A trait representing types that can be returned from a native Java method wrapper.
///
/// These are types that can be passed to Java method wrappers as arguments plus
/// [`()`](https://doc.rust-lang.org/std/primitive.unit.html).
pub trait ToJavaNativeResult {
    type JniType: JniType;

    // Unsafe because it returns raw pointers to Java objects.
    unsafe fn into_java_native_result(self) -> Self::JniType;
}

impl<'this, T> ToJavaNativeResult for T
where
    T: JavaClass<'this>,
{
    type JniType = jni_sys::jobject;

    #[inline(always)]
    unsafe fn into_java_native_result(self) -> Self::JniType {
        let result = self.as_ref().raw_object().as_ptr();
        // Transfer ownership of the reference to Java code.
        mem::forget(self);
        result
    }
}

impl<'this, T> ToJavaNativeResult for Option<T>
where
    T: JavaClass<'this>,
{
    type JniType = jni_sys::jobject;

    #[inline(always)]
    unsafe fn into_java_native_result(self) -> Self::JniType {
        self.map_or(ptr::null_mut(), |value| value.into_java_native_result())
    }
}

/// A trait representing types that can be passed to native Java method wrappers
/// as arguments.
///
/// These are either primitive types convertible to JNI types or
/// [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html)-s of Java class wrappers.
pub trait ToJavaNativeArgument<'this> {
    type JniType: JniNativeArgumentType;
    type ArgumentType;

    unsafe fn from_raw(env: &'this JniEnv<'this>, value: Self::JniType) -> Self::ArgumentType;
}

impl<'this, T> ToJavaNativeArgument<'this> for T
where
    T: JavaClass<'this>,
{
    type JniType = jni_sys::jobject;
    type ArgumentType = Option<T>;

    #[inline(always)]
    unsafe fn from_raw(env: &'this JniEnv<'this>, value: Self::JniType) -> Self::ArgumentType {
        NonNull::new(value)
            .map(|value| <T as FromObject<'this>>::from_object(Object::from_raw(env, value)))
    }
}

pub trait ToJavaNativeArgumentTuple<'this> {
    type JniType;
    type ArgumentType;

    unsafe fn from_raw(env: &'this JniEnv<'this>, value: Self::JniType) -> Self::ArgumentType;
}

macro_rules! peel_java_argument_type_impls {
    () => ();
    ($type:ident, $($other:ident,)*) => (java_argument_type_impls! { $($other,)* });
}

macro_rules! java_argument_type_impls {
    ( $($type:ident,)*) => (
        impl<'this, $($type),*> ToJavaNativeArgumentTuple<'this> for ($($type,)*)
        where
            $($type: ToJavaNativeArgument<'this>,)*
        {
            type JniType = ($(<$type as ToJavaNativeArgument<'this>>::JniType,)*);
            type ArgumentType = ($(<$type as ToJavaNativeArgument<'this>>::ArgumentType,)*);

            #[allow(unused)]
            #[inline(always)]
            unsafe fn from_raw(env: &'this JniEnv<'this>, value: Self::JniType) -> Self::ArgumentType {
                #[allow(non_snake_case)]
                let ($($type,)*) = value;
                ($(<$type as ToJavaNativeArgument<'this>>::from_raw(env, $type),)*)
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

/// Implementation of a static native Java method.
///
/// This function provides everything needed to execute normal safe [`rust-jni`](index.html) code.
/// It accepts a [`*mut JNIEnv`](../jni_sys/type.JNIEnv.html) and a [`jclass`](../jni_sys/type.jclass.html)
/// that JNI passes to a native method and a callback that accepts a [`&Class`](java/lang/struct.Class.html)
/// and a [`NoException`](struct.NoException.html) token and returns a [`JavaResult`](type.JavaResult.html)
/// with any type that can be returned to Java and the [`NoException`](struct.NoException.html) that proves
/// that there is no pending exception.
///
/// Example:
/// ```
/// # use rust_jni::*;
/// # use rust_jni::java::lang::{Object, String};
/// # use std::ptr;
/// # use std::mem;
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
/// #
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_String_valueOf__I(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_class: jni_sys::jclass,
///     raw_argument: jni_sys::jint,
/// ) -> jni_sys::jstring {
///     static_native_method_implementation::<(i32,), String, _>(
///         raw_env,
///         raw_class,
///         (raw_argument,),
///         |class, token, (argument,)| {
///             assert_eq!(
///                 class
///                     .get_name(&token)
///                     .or_npe(&token)
///                     .unwrap()
///                     .as_string(&token),
///                 "java.lang.String",
///             );
///             let result = String::value_of_int(&token, *argument)
///                 .or_npe(&token)
///                 .unwrap();
///             assert_eq!(result.as_string(&token), "17");
///             (Ok(result), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string_class = String::empty(&token)?.class(&token);
/// let string = Java_java_lang_String_valueOf__I(
///     token.env().raw_env().as_ptr(),
///     string_class.raw_object().as_ptr(),
///     17 as jni_sys::jint,
/// );
/// assert_ne!(string, ptr::null_mut());
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// Example returning an [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html):
/// ```
/// # use rust_jni::*;
/// # use rust_jni::java::lang::{Object, String};
/// # use std::ptr;
/// # use std::mem;
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
/// #
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_String_valueOf__I(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_class: jni_sys::jclass,
///     raw_argument: jni_sys::jint,
/// ) -> jni_sys::jstring {
///     static_native_method_implementation::<(i32,), Option<String>, _>(
///         raw_env,
///         raw_class,
///         (raw_argument,),
///         |class, token, (argument,)| {
///             assert_eq!(
///                 class
///                     .get_name(&token)
///                     .or_npe(&token)
///                     .unwrap()
///                     .as_string(&token),
///                 "java.lang.String",
///             );
///             let result = String::value_of_int(&token, *argument)
///                 .unwrap();
///             if result.as_ref().unwrap().as_string(&token) == "17" {
///                 (Ok(result), token)
///             } else {
///                 (Ok(None as Option<String>), token)
///             }
///         }
///     )
/// }
///
/// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string_class = String::empty(&token)?.class(&token);
/// let string = Java_java_lang_String_valueOf__I(
///     token.env().raw_env().as_ptr(),
///     string_class.raw_object().as_ptr(),
///     17 as jni_sys::jint,
/// );
/// assert_ne!(string, ptr::null_mut());
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// Example with an exeption:
/// ```
/// # use rust_jni::*;
/// # use rust_jni::java::lang::{String, Throwable};
/// # use std::ptr;
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
/// #
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_String_valueOf__I(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_class: jni_sys::jclass,
///     raw_argument: jni_sys::jint,
/// ) -> jni_sys::jstring {
///     static_native_method_implementation::<(i32,), String, _>(
///         raw_env,
///         raw_class,
///         (raw_argument,),
///         |class, token, (argument,)| {
///             let exception = Throwable::new(&token).unwrap();
///             let token = exception.throw(token);
///             let (exception, token) = token.unwrap();
///             (Err(exception), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string_class = String::empty(&token)?.class(&token);
/// let string = Java_java_lang_String_valueOf__I(
///     token.env().raw_env().as_ptr(),
///     string_class.raw_object().as_ptr(),
///     17 as jni_sys::jint,
/// );
/// assert_eq!(string, ptr::null_mut());
///
/// let raw_env = token.env().raw_env().as_ptr();
/// let jni_fn = ((**raw_env).ExceptionOccurred).unwrap();
/// let throwable = jni_fn(raw_env);
/// assert_ne!(throwable, ptr::null_mut());
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer or an invalid [`jclass`](../jni_sys/type.jclass.html).
pub unsafe fn static_native_method_implementation<'this, A, R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    raw_arguments: A::JniType,
    callback: F,
) -> R::JniType
where
    F: FnOnce(
        &Class<'this>,
        NoException<'this>,
        &A::ArgumentType,
    ) -> (JavaResult<'this, R>, NoException<'this>),
    F: panic::UnwindSafe,
    A: ToJavaNativeArgumentTuple<'this>,
    A::JniType: panic::UnwindSafe,
    R: ToJavaNativeResult,
{
    generic_native_method_implementation::<R::JniType, A::JniType, _>(
        raw_env,
        raw_arguments,
        |token, arguments| {
            // Safe because even though the `token` value is elevated from a local lifetime 'a
            // to caller lifetime 'this (which is wider), it is only used for the duration of 'a.
            // The `token` value is `drop`-ed when 'a is over and so it does not leak into
            // the rest of 'this.
            // This is a hack to make the compiler happy. Because the trait bound
            // for `A` on this method that requires a lifetime: `A: ToJavaNativeArgumentTuple<'this>`
            // we had to introduce a lifetime parameter 'this to be used both in that bound and in
            // the trait bound for `F`. If that was not needed we would use a
            // `for<'a> F: FnOnce(...)` [HKTB](https://doc.rust-lang.org/nomicon/hrtb.html) instead.
            // Then the caller lifetime 'this would not leak into this fuction and the hack
            // could be removed.
            let token: NoException<'this> = unsafe { extend_token_lifetime(token) };
            // Should not panic if the class pointer is valid.
            let class = ManuallyDrop::new(Class::from_raw(
                token.env(),
                NonNull::new(raw_class).unwrap(),
            ));
            let arguments = ManuallyDrop::new(<A as ToJavaNativeArgumentTuple<'this>>::from_raw(
                token.env(),
                arguments,
            ));
            let (result, token) = callback(&class, token, &arguments);
            to_jni_type::<R>(result, token)
        },
    )
}

/// Implementation of a native Java method.
///
/// This function provides everything needed to execute normal safe [`rust-jni`](index.html) code.
/// It accepts a [`*mut JNIEnv`](../jni_sys/type.JNIEnv.html) and a [`jobject`](../jni_sys/type.jobject.html)
/// that JNI passes to a native method and a callback that accepts a [`&Object`](java/lang/struct.Object.html)
/// and a [`NoException`](struct.NoException.html) token and returns a [`JavaResult`](type.JavaResult.html)
/// with any type that can be returned to Java and the [`NoException`](struct.NoException.html) that proves
/// that there is no pending exception.
///
/// Example:
/// ```
/// # use rust_jni::*;
/// # use rust_jni::java::lang::{Object, String};
/// # use std::ptr::{self, NonNull};
/// # use std::mem;
/// # use jni_sys;
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
/// #
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_object: jni_sys::jobject,
///     raw_argument: jni_sys::jobject,
/// ) -> jni_sys::jboolean {
///     native_method_implementation::<(Object,), bool, _>(
///         raw_env,
///         raw_object,
///         (raw_argument,),
///         |object, token, (argument,)| {
///             let argument = argument
///                 .as_ref()
///                 .or_npe(&token)
///                 .unwrap();
///             let result = object.equals(&token, &argument).unwrap();
///             (Ok(result), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string1 = String::empty(&token)?;
/// let string2 = String::new(&token, "")?;
/// let equals = Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     token.env().raw_env().as_ptr(),
///     string1.raw_object().as_ptr(),
///     string2.raw_object().as_ptr(),
/// );
/// assert_eq!(equals, jni_sys::JNI_TRUE);
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// Example with an exception:
/// ```
/// # use rust_jni::*;
/// # use rust_jni::java::lang::{Object, String, Throwable};
/// # use jni_sys;
/// # use std::ptr;
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
/// #
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_object: jni_sys::jobject,
///     raw_argument: jni_sys::jobject,
/// ) -> jni_sys::jboolean {
///     native_method_implementation::<(Object,), bool, _>(
///         raw_env,
///         raw_object,
///         (raw_argument,),
///         |object, token, (argument,)| {
///             let exception = Throwable::new(&token).unwrap();
///             let token = exception.throw(token);
///             let (exception, token) = token.unwrap();
///             (Err(exception), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string1 = String::empty(&token)?;
/// let string2 = String::new(&token, "")?;
/// let equals = Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     token.env().raw_env().as_ptr(),
///     string1.raw_object().as_ptr(),
///     string2.raw_object().as_ptr(),
/// );
/// assert_eq!(equals, jni_sys::JNI_FALSE);
///
/// let raw_env = token.env().raw_env().as_ptr();
/// let jni_fn = ((**raw_env).ExceptionOccurred).unwrap();
/// let throwable = jni_fn(raw_env);
/// assert_ne!(throwable, ptr::null_mut());
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer or an invalid [`jobject`](../jni_sys/type.jobject.html).
pub unsafe fn native_method_implementation<'this, A, R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    raw_arguments: A::JniType,
    callback: F,
) -> R::JniType
where
    F: FnOnce(
        &Object<'this>,
        NoException<'this>,
        &A::ArgumentType,
    ) -> (JavaResult<'this, R>, NoException<'this>),
    F: panic::UnwindSafe,
    A: ToJavaNativeArgumentTuple<'this>,
    A::JniType: panic::UnwindSafe,
    R: ToJavaNativeResult,
{
    generic_native_method_implementation::<R::JniType, A::JniType, _>(
        raw_env,
        raw_arguments,
        |token, arguments| {
            // Safe because even though the `token` value is elevated from a local lifetime 'a
            // to caller lifetime 'this (which is wider), it is only used for the duration of 'a.
            // The `token` value is `drop`-ed when 'a is over and so it does not leak into
            // the rest of 'this.
            // This is a hack to make the compiler happy. Because the trait bound
            // for `A` on this method that requires a lifetime: `A: ToJavaNativeArgumentTuple<'this>`
            // we had to introduce a lifetime parameter 'this to be used both in that bound and in
            // the trait bound for `F`. If that was not needed we would use a
            // `for<'a> F: FnOnce(...)` [HKTB](https://doc.rust-lang.org/nomicon/hrtb.html) instead.
            // Then the caller lifetime 'this would not leak into this fuction and the hack
            // could be removed.
            let token: NoException<'this> = unsafe { extend_token_lifetime(token) };
            // Should not panic if the object pointer is valid.
            let object = ManuallyDrop::new(Object::from_raw(
                token.env(),
                NonNull::new(raw_object).unwrap(),
            ));
            let arguments = ManuallyDrop::new(<A as ToJavaNativeArgumentTuple<'this>>::from_raw(
                token.env(),
                arguments,
            ));
            let (result, token) = callback(&object, token, &arguments);
            to_jni_type::<R>(result, token)
        },
    )
}

pub unsafe fn native_method_implementation_new<'this, S, A, R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    raw_arguments: A::JniType,
    callback: F,
) -> R::JniType
where
    F: FnOnce(
        &S,
        NoException<'this>,
        &A::ArgumentType,
    ) -> (JavaResult<'this, R>, NoException<'this>),
    F: panic::UnwindSafe,
    A: ToJavaNativeArgumentTuple<'this>,
    A::JniType: panic::UnwindSafe,
    R: ToJavaNativeResult,
    S: JavaClass<'this>,
{
    generic_native_method_implementation::<R::JniType, A::JniType, _>(
        raw_env,
        raw_arguments,
        |token, arguments| {
            // Safe because even though the `token` value is elevated from a local lifetime 'a
            // to caller lifetime 'this (which is wider), it is only used for the duration of 'a.
            // The `token` value is `drop`-ed when 'a is over and so it does not leak into
            // the rest of 'this.
            // This is a hack to make the compiler happy. Because the trait bound
            // for `A` on this method that requires a lifetime: `A: ToJavaNativeArgumentTuple<'this>`
            // we had to introduce a lifetime parameter 'this to be used both in that bound and in
            // the trait bound for `F`. If that was not needed we would use a
            // `for<'a> F: FnOnce(...)` [HKTB](https://doc.rust-lang.org/nomicon/hrtb.html) instead.
            // Then the caller lifetime 'this would not leak into this fuction and the hack
            // could be removed.
            let token: NoException<'this> = unsafe { extend_token_lifetime(token) };
            // Should not panic if the object pointer is valid.
            let object = ManuallyDrop::new(<S as FromObject<'this>>::from_object(
                Object::from_raw(token.env(), NonNull::new(raw_object).unwrap()),
            ));
            let arguments = ManuallyDrop::new(<A as ToJavaNativeArgumentTuple<'this>>::from_raw(
                token.env(),
                arguments,
            ));
            let (result, token) = callback(&object, token, &arguments);
            to_jni_type::<R>(result, token)
        },
    )
}

unsafe fn to_jni_type<'a, R>(result: JavaResult<'a, R>, token: NoException<'a>) -> R::JniType
where
    R: ToJavaNativeResult,
{
    match result {
        Ok(result) => {
            mem::forget(token);
            result.into_java_native_result()
        }
        Err(exception) => {
            let _ = exception.throw(token);
            R::JniType::default()
        }
    }
}

// Unsafe because it extends a lifetime of a value.
unsafe fn extend_token_lifetime<'b, 's>(r: NoException<'b>) -> NoException<'s> {
    std::mem::transmute::<NoException<'b>, NoException<'s>>(r)
}

/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer.
unsafe fn generic_native_method_implementation<R, A, F>(
    raw_env: *mut jni_sys::JNIEnv,
    arguments: A,
    callback: F,
) -> R
where
    for<'a> F: FnOnce(NoException<'a>, A) -> R + panic::UnwindSafe,
    R: JniType,
    A: panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| {
        let mut java_vm: *mut jni_sys::JavaVM = ptr::null_mut();
        let get_java_vm_fn = ((**raw_env).GetJavaVM).unwrap();
        let error = JniError::from_raw(get_java_vm_fn(
            raw_env,
            (&mut java_vm) as *mut *mut jni_sys::JavaVM,
        ));
        if error.is_some() {
            panic!("Could not get Java VM. Status: {:?}", error.unwrap());
        }

        // Safe because we pass a valid `java_vm` pointer.
        // Will not panic because JNI guarantees that pointers are not null.
        #[allow(unused_unsafe)]
        let vm = unsafe { JavaVMRef::from_ptr(NonNull::new(java_vm).unwrap()) };
        // Safe because we pass a valid `raw_env` pointer.
        // Will not panic because JNI guarantees that pointers are not null.
        #[allow(unused_unsafe)]
        let env = ManuallyDrop::new(unsafe { JniEnv::new(&vm, NonNull::new(raw_env).unwrap()) });
        let token = env.token();
        callback(token, arguments)
    });
    match result {
        Ok(result) => result,
        Err(error) => {
            if let Some(string) = error.downcast_ref::<std::string::String>() {
                // Safe because we pass a correct `raw_env` pointer.
                #[allow(unused_unsafe)]
                unsafe {
                    throw_new_runtime_exception(raw_env, format!("Rust panic: {}\0", string))
                };
            } else if let Some(string) = error.downcast_ref::<&str>() {
                // Safe because we pass a correct `raw_env` pointer.
                #[allow(unused_unsafe)]
                unsafe {
                    throw_new_runtime_exception(raw_env, format!("Rust panic: {}\0", string))
                };
            } else {
                // Safe because we pass a correct `raw_env` pointer.
                #[allow(unused_unsafe)]
                unsafe {
                    throw_new_runtime_exception(raw_env, "Rust panic: generic panic.\0")
                };
            }
            R::default()
        }
    }
}

/// Unsafe because an incorrect pointer can be passed as an argument.
unsafe fn throw_new_runtime_exception(raw_env: *mut jni_sys::JNIEnv, message: impl AsRef<str>) {
    let message = to_java_string_null_terminated(message.as_ref());
    let class_name = to_java_string_null_terminated("java/lang/RuntimeException\0");
    let find_class = (**raw_env).FindClass.unwrap();
    let class = find_class(raw_env, class_name.as_ptr() as *const i8);
    if class == ptr::null_mut() {
        panic!(
            "Could not find the java.lang.RuntimeException class on panic, aborting the program."
        );
    } else {
        let throw_new_fn = (**raw_env).ThrowNew.unwrap();
        let error = JniError::from_raw(throw_new_fn(raw_env, class, message.as_ptr() as *const i8));
        if error.is_some() {
            panic!("Could not throw a new runtime exception on panic, status {:?}, aborting the program.", error.unwrap());
        }
    }
}
