use crate::class::Class;
use crate::env::JniEnv;
use crate::error::JniError;
use crate::java_class::JavaClass;
use crate::java_methods::FromObject;
use crate::java_methods::JavaArgumentType;
use crate::java_string::to_java_string_null_terminated;
use crate::jni_types::private::JniArgumentType;
use crate::jni_types::private::JniArgumentTypeTuple;
use crate::jni_types::private::JniType;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use crate::vm::JavaVMRef;
use jni_sys;
use std::alloc;
use std::mem;
use std::panic;
use std::ptr::{self, NonNull};

/// A trait representing types that can be returned from a native Java method wrapper.
///
/// These are types that can be passed to Java method wrappers as arguments plus
/// [`()`](https://doc.rust-lang.org/std/primitive.unit.html).
pub trait ToJavaNativeResult {
    type JniType: JniType;

    fn to_java_native_result(&self) -> Self::JniType;
}

impl<T> ToJavaNativeResult for T
where
    T: JavaArgumentType,
{
    type JniType = <T as JavaArgumentType>::JniType;

    #[inline(always)]
    fn to_java_native_result(&self) -> Self::JniType {
        <T as JavaArgumentType>::to_jni(self)
    }
}

impl ToJavaNativeResult for () {
    type JniType = ();

    #[inline(always)]
    fn to_java_native_result(&self) -> Self::JniType {
        ()
    }
}

/// A trait representing types that can be passed to native Java method wrappers
/// as arguments.
///
/// These are either primitive types convertible to JNI types or
/// [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html)-s of Java class wrappers.
pub trait ToJavaNativeArgument {
    type JniType: JniArgumentType;

    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, value: Self::JniType) -> Self;
}

impl<'b, T> ToJavaNativeArgument for Option<T>
where
    T: JavaClass<'b>,
{
    type JniType = jni_sys::jobject;

    #[inline(always)]
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, value: Self::JniType) -> Self {
        // We use extend_lifetime_object() to satisfy the "T: JavaClass<'b>"
        // condition on the trait impl. This is safe as we then shrink the
        // resutl's lifetime to a proper one before passing the value to user code.
        // This is needed because we really don't want to paramentize
        // ToJavaNativeArgument and ToJavaNativeArgumentTuple with lifetimes
        // as then it's impossible to use these traits as bounds
        // on HKTB-ed closure type in native method wrappers below.
        // TODO(monnoroch): clean this up once TODOs below are resolved.
        NonNull::new(value).map(|value| {
            <T as FromObject>::from_object(extend_lifetime_object(Object::from_raw(env, value)))
        })
    }
}

pub trait ToJavaNativeArgumentTuple {
    type JniType: JniArgumentTypeTuple;

    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, value: Self::JniType) -> Self;
}

macro_rules! peel_java_argument_type_impls {
    () => ();
    ($type:ident, $($other:ident,)*) => (java_argument_type_impls! { $($other,)* });
}

macro_rules! java_argument_type_impls {
    ( $($type:ident,)*) => (
        impl<$($type),*> ToJavaNativeArgumentTuple for ($($type,)*)
        where
            $($type: ToJavaNativeArgument,)*
        {
            type JniType = ($(<$type as ToJavaNativeArgument>::JniType,)*);

            #[allow(unused)]
            #[inline(always)]
            unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, value: Self::JniType) -> Self {
                #[allow(non_snake_case)]
                let ($($type,)*) = value;
                ($(<$type as ToJavaNativeArgument>::from_raw(env, $type),)*)
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
/// # fn main() {
/// #     let init_arguments = InitArguments::default();
/// #     let vm = JavaVM::create(&init_arguments).unwrap();
/// #     let _ = vm.with_attached(
/// #        &AttachArguments::new(init_arguments.version()),
/// #        |env: &JniEnv, token: NoException| {
/// #            ((), jni_main(env, token).unwrap())
/// #        },
/// #     );
/// # }
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
///             let env = class.env();
///             assert_eq!(
///                 class
///                     .get_name(&token)
///                     .or_npe(env, &token)
///                     .unwrap()
///                     .as_string(&token),
///                 "java.lang.String",
///             );
///             let result = String::value_of_int(env, &token, *argument)
///                 .or_npe(env, &token)
///                 .unwrap();
///             assert_eq!(result.as_string(&token), "17");
///             (Ok(Box::new(result)), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(env: &'a JniEnv<'a>, token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string_class = String::empty(env, &token)?.class(&token);
/// let string = Java_java_lang_String_valueOf__I(
///     env.raw_env().as_ptr(),
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
/// # fn main() {
/// #     let init_arguments = InitArguments::default();
/// #     let vm = JavaVM::create(&init_arguments).unwrap();
/// #     let _ = vm.with_attached(
/// #        &AttachArguments::new(init_arguments.version()),
/// #        |env: &JniEnv, token: NoException| {
/// #            ((), jni_main(env, token).unwrap())
/// #        },
/// #     );
/// # }
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
///             let env = class.env();
///             assert_eq!(
///                 class
///                     .get_name(&token)
///                     .or_npe(env, &token)
///                     .unwrap()
///                     .as_string(&token),
///                 "java.lang.String",
///             );
///             let result = String::value_of_int(env, &token, *argument)
///                 .unwrap();
///             if result.as_ref().unwrap().as_string(&token) == "17" {
///                 (Ok(Box::new(result)), token)
///             } else {
///                 (Ok(Box::new(None as Option<String>)), token)
///             }
///         }
///     )
/// }
///
/// # fn jni_main<'a>(env: &'a JniEnv<'a>, token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string_class = String::empty(env, &token)?.class(&token);
/// let string = Java_java_lang_String_valueOf__I(
///     env.raw_env().as_ptr(),
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
/// # fn main() {
/// #     let init_arguments = InitArguments::default();
/// #     let vm = JavaVM::create(&init_arguments).unwrap();
/// #     let _ = vm.with_attached(
/// #        &AttachArguments::new(init_arguments.version()),
/// #        |env: &JniEnv, token: NoException| {
/// #            ((), jni_main(env, token).unwrap())
/// #        },
/// #     );
/// # }
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
///             let exception = Throwable::new(class.env(), &token).unwrap();
///             let token = exception.throw(token);
///             let (exception, token) = token.unwrap();
///             (Err(exception), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(env: &'a JniEnv<'a>, token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string_class = String::empty(env, &token)?.class(&token);
/// let string = Java_java_lang_String_valueOf__I(
///     env.raw_env().as_ptr(),
///     string_class.raw_object().as_ptr(),
///     17 as jni_sys::jint,
/// );
/// assert_eq!(string, ptr::null_mut());
///
/// let raw_env = env.raw_env().as_ptr();
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
pub unsafe fn static_native_method_implementation<A, R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    raw_arguments: A::JniType,
    callback: F,
) -> R::JniType
where
    for<'a> F: FnOnce(
        &'a Class<'a>,
        NoException<'a>,
        &'a A,
    ) -> (
        JavaResult<'a, Box<dyn ToJavaNativeResult<JniType = R::JniType> + 'a>>,
        NoException<'a>,
    ),
    F: panic::UnwindSafe,
    // TODO(monnoroch): this should be + 'a for the 'a in the HKTB above.
    A: ToJavaNativeArgumentTuple,
    A::JniType: panic::UnwindSafe,
    R: ToJavaNativeResult,
{
    generic_native_method_implementation::<R::JniType, A::JniType, _>(
        raw_env,
        raw_arguments,
        |env, token, arguments| {
            // Should not panic if the class pointer is valid.
            let class = Class::from_raw(env, NonNull::new(raw_class).unwrap());
            let arguments = <A as ToJavaNativeArgumentTuple>::from_raw(env, arguments);
            let (result, token) = callback(&class, token, &arguments);
            let java_result = to_jni_type::<R>(result, token);
            // We don't own the reference.
            mem::forget(arguments);
            // We don't own the reference.
            mem::forget(class);
            java_result
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
/// # fn main() {
/// #     let init_arguments = InitArguments::default();
/// #     let vm = JavaVM::create(&init_arguments).unwrap();
/// #     let _ = vm.with_attached(
/// #        &AttachArguments::new(init_arguments.version()),
/// #        |env: &JniEnv, token: NoException| {
/// #            ((), jni_main(env, token).unwrap())
/// #        },
/// #     );
/// # }
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_object: jni_sys::jobject,
///     raw_argument: jni_sys::jobject,
/// ) -> jni_sys::jboolean {
///     native_method_implementation::<(Option<Object>,), bool, _>(
///         raw_env,
///         raw_object,
///         (raw_argument,),
///         |object, token, (argument,)| {
///             let env = object.env();
///             let argument = argument
///                 .as_ref()
///                 .or_npe(env, &token)
///                 .unwrap();
///             let result = object.equals(&token, &argument).unwrap();
///             (Ok(Box::new(result)), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(env: &'a JniEnv<'a>, token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string1 = String::empty(env, &token)?;
/// let string2 = String::new(env, &token, "")?;
/// let equals = Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     env.raw_env().as_ptr(),
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
/// # fn main() {
/// #     let init_arguments = InitArguments::default();
/// #     let vm = JavaVM::create(&init_arguments).unwrap();
/// #     let _ = vm.with_attached(
/// #        &AttachArguments::new(init_arguments.version()),
/// #        |env: &JniEnv, token: NoException| {
/// #            ((), jni_main(env, token).unwrap())
/// #        },
/// #     );
/// # }
/// #[no_mangle]
/// unsafe extern "C" fn Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_object: jni_sys::jobject,
///     raw_argument: jni_sys::jobject,
/// ) -> jni_sys::jboolean {
///     native_method_implementation::<(Option<Object>,), bool, _>(
///         raw_env,
///         raw_object,
///         (raw_argument,),
///         |object, token, (argument,)| {
///             let exception = Throwable::new(object.env(), &token).unwrap();
///             let token = exception.throw(token);
///             let (exception, token) = token.unwrap();
///             (Err(exception), token)
///         }
///     )
/// }
///
/// # fn jni_main<'a>(env: &'a JniEnv<'a>, token: NoException<'a>) -> JavaResult<'a, NoException<'a>> {
/// # unsafe {
/// let string1 = String::empty(env, &token)?;
/// let string2 = String::new(env, &token, "")?;
/// let equals = Java_java_lang_Object_equals__Ljava_lang_Object_2(
///     env.raw_env().as_ptr(),
///     string1.raw_object().as_ptr(),
///     string2.raw_object().as_ptr(),
/// );
/// assert_eq!(equals, jni_sys::JNI_FALSE);
///
/// let raw_env = env.raw_env().as_ptr();
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
pub unsafe fn native_method_implementation<A, R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    raw_arguments: A::JniType,
    callback: F,
) -> R::JniType
where
    for<'a> F: FnOnce(
        &'a Object<'a>,
        NoException<'a>,
        &'a A,
    ) -> (
        JavaResult<'a, Box<dyn ToJavaNativeResult<JniType = R::JniType> + 'a>>,
        NoException<'a>,
    ),
    F: panic::UnwindSafe,
    // TODO(monnoroch): this should be + 'a for the 'a in the HKTB above.
    A: ToJavaNativeArgumentTuple,
    A::JniType: panic::UnwindSafe,
    R: ToJavaNativeResult,
{
    generic_native_method_implementation::<R::JniType, A::JniType, _>(
        raw_env,
        raw_arguments,
        |env, token, arguments| {
            // Should not panic if the object pointer is valid.
            let object = Object::from_raw(env, NonNull::new(raw_object).unwrap());
            let arguments = <A as ToJavaNativeArgumentTuple>::from_raw(env, arguments);
            let (result, token) = callback(&object, token, &arguments);
            let java_result = to_jni_type::<R>(result, token);
            // We don't own the reference.
            mem::forget(arguments);
            // We don't own the reference.
            mem::forget(object);
            java_result
        },
    )
}

fn to_jni_type<'a, R>(
    result: JavaResult<'a, Box<dyn ToJavaNativeResult<JniType = R::JniType> + 'a>>,
    token: NoException<'a>,
) -> R::JniType
where
    R: ToJavaNativeResult + 'a,
{
    match result {
        Ok(result) => {
            mem::forget(token);
            let java_result = result.to_java_native_result();
            // Here we want to free memory of the Box, but don't want to run the destructor of the boxed value.
            // Running the destructor for primitive types won't do anything, but running the destructor
            // for a Java class wrapper will delete it's reference, which will make Java delete the object.
            // Here we could use mem::forget(result), but that would leak the Box-es memory, which we don't want.
            let result = Box::into_raw(result);
            // Safe because we just took ownership of this memory.
            unsafe { alloc::dealloc(result as *mut u8, alloc::Layout::for_value(&*result)) };
            java_result
        }
        #[cold]
        Err(exception) => {
            let _ = exception.throw(token);
            R::JniType::default()
        }
    }
}

unsafe fn extend_lifetime_object<'b>(r: Object<'b>) -> Object<'static> {
    std::mem::transmute::<Object<'b>, Object<'static>>(r)
}

/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer.
unsafe fn generic_native_method_implementation<R, A, F>(
    raw_env: *mut jni_sys::JNIEnv,
    arguments: A,
    callback: F,
) -> R
where
    for<'a> F: FnOnce(&'a JniEnv<'a>, NoException<'a>, A) -> R + panic::UnwindSafe,
    R: JniType,
    A: JniArgumentTypeTuple + panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| {
        let mut java_vm: *mut jni_sys::JavaVM = ptr::null_mut();
        let get_java_vm_fn = ((**raw_env).GetJavaVM).unwrap();
        let error = JniError::from_raw(get_java_vm_fn(
            raw_env,
            (&mut java_vm) as *mut *mut jni_sys::JavaVM,
        ));
        if error.is_some() {
            #[cold]
            panic!(format!(
                "Could not get Java VM. Status: {:?}",
                error.unwrap()
            ));
        }

        // Safe because we pass a valid `java_vm` pointer.
        // Will not panic because JNI guarantees that pointers are not null.
        #[allow(unused_unsafe)]
        let vm = unsafe { JavaVMRef::from_ptr(NonNull::new(java_vm).unwrap()) };
        // Safe because we pass a valid `raw_env` pointer.
        // Will not panic because JNI guarantees that pointers are not null.
        #[allow(unused_unsafe)]
        let env = unsafe { JniEnv::native(&vm, NonNull::new(raw_env).unwrap()) };
        let token = env.token();
        let result = callback(&env, token, arguments);
        // We don't own the reference.
        mem::forget(env);
        result
    });
    match result {
        Ok(result) => result,
        #[cold]
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
        #[cold]
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
