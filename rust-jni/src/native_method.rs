use crate::class::Class;
use crate::env::JniEnv;
use crate::error::JniError;
use crate::java_string::to_java_string_null_terminated;
use crate::jni_types::private::JniArgumentTypeTuple;
use crate::jni_types::private::JniType;
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::NoException;
use crate::vm::JavaVMRef;
use jni_sys;
use std::mem;
use std::panic;
use std::ptr::{self, NonNull};

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
/// # use rust_jni::java::lang::String;
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
///     static_native_method_implementation(
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
///             let result = String::value_of_int(env, &token, argument as i32)
///                 .or_npe(env, &token)
///                 .unwrap();
///             assert_eq!(result.as_string(&token), "17");
///             // Safe because we only pass it to Java.
///             let java_result = unsafe { result.raw_object() }.as_ptr();
///             // We don't want to return a deleted reference to Java.
///             mem::forget(result);
///             (Ok(java_result), token)
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
///     static_native_method_implementation(
///         raw_env,
///         raw_class,
///         (raw_argument,),
///         |class, token, (argument,)| {
///             let exception = Throwable::new(class.env(), &token).unwrap();
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
pub unsafe fn static_native_method_implementation<R, A, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    raw_arguments: A,
    callback: F,
) -> R
where
    // TODO(monnoroch): find out how to do this without allocation.
    // See https://stackoverflow.com/questions/59003532/using-higher-ranked-trait-bounds-with-generics.
    for<'a> F: FnOnce(&'a Class<'a>, NoException<'a>, A) -> (JavaResult<'a, R>, NoException<'a>)
        + std::panic::UnwindSafe,
    R: JniType,
    A: JniArgumentTypeTuple + panic::UnwindSafe,
{
    generic_native_method_implementation::<R, A, _>(
        raw_env,
        raw_arguments,
        |env, token, arguments| {
            // Should not panic if the class pointer is valid.
            let class = Class::from_raw(&env, NonNull::new(raw_class).unwrap());
            let (result, token) = callback(&class, token, arguments);
            let result = to_jni_type::<R>(result, token);
            // We don't own the reference.
            mem::forget(class);
            result
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
///     native_method_implementation(
///         raw_env,
///         raw_object,
///         (raw_argument,),
///         |object, token, (argument,)| {
///             let env = object.env();
///             let argument = NonNull::new(argument)
///                 .map(|argument| unsafe {
///                     String::from_object(Object::from_raw(env, argument))
///                 })
///                 .or_npe(env, &token)
///                 .unwrap();
///             let result = object.equals(&token, &argument).unwrap();
///             // We don't own the reference.
///             mem::forget(argument);
///             let java_result = if result {
///                 jni_sys::JNI_TRUE
///             } else {
///                 jni_sys::JNI_FALSE
///             };
///             (Ok(java_result), token)
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
/// # use rust_jni::java::lang::{String, Throwable};
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
///     native_method_implementation(
///         raw_env,
///         raw_object,
///         (raw_argument,),
///         |object, token, (argument,)| {
///             let exception = Throwable::new(object.env(), &token).unwrap();
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
pub unsafe fn native_method_implementation<R, A, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    raw_arguments: A,
    callback: F,
) -> R
where
    // TODO(monnoroch): find out how to do this without allocation.
    // See https://stackoverflow.com/questions/59003532/using-higher-ranked-trait-bounds-with-generics.
    for<'a> F: FnOnce(&'a Object<'a>, NoException<'a>, A) -> (JavaResult<'a, R>, NoException<'a>)
        + std::panic::UnwindSafe,
    R: JniType,
    A: JniArgumentTypeTuple + panic::UnwindSafe,
{
    generic_native_method_implementation::<R, A, _>(
        raw_env,
        raw_arguments,
        |env, token, arguments| {
            // Should not panic if the class pointer is valid.
            let object = Object::from_raw(&env, NonNull::new(raw_object).unwrap());
            let (result, token) = callback(&object, token, arguments);
            let result = to_jni_type::<R>(result, token);
            // We don't own the reference.
            mem::forget(object);
            result
        },
    )
}

fn to_jni_type<'a, R>(result: JavaResult<'a, R>, token: NoException<'a>) -> R
where
    R: JniType + 'a,
{
    match result {
        Ok(result) => {
            // The token is consumed.
            mem::forget(token);
            result
        }
        #[cold]
        Err(exception) => {
            let _ = exception.throw(token);
            R::default()
        }
    }
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
