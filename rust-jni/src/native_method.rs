use crate::class::Class;
use crate::env::JniEnv;
use crate::error::JniError;
use crate::java_methods::JavaArgumentType;
use crate::java_string::to_java_string_null_terminated;
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

/// A trait implemented for all types that can be results of native Java methods.
///
/// These are all primitive JNI types, [`()`](https://doc.rust-lang.org/stable/std/primitive.unit.html) and
/// Java class wrappers.
pub trait NativeMethodResult {
    type JniType: JniType;

    fn to_jni(&self) -> Self::JniType;
}

impl<T> NativeMethodResult for T
where
    T: JavaArgumentType,
{
    type JniType = <Self as JavaArgumentType>::JniType;

    fn to_jni(&self) -> Self::JniType {
        JavaArgumentType::to_jni(self)
    }
}

impl NativeMethodResult for () {
    type JniType = ();

    fn to_jni(&self) -> Self::JniType {
        *self
    }
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
/// # use rust_jni::java::lang::String;
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
/// unsafe extern "C" fn Java_java_lang_String_valueOf__I(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_class: jni_sys::jclass,
///     argument: jni_sys::jint,
/// ) -> jni_sys::jstring {
///     static_native_method_implementation::<String, _>(
///         raw_env,
///         raw_class,
///         |class, token| {
///             let env = class.env();
///             assert_eq!(
///                 class
///                     .get_name(&token)
///                     .or_npe(env, &token)
///                     .unwrap()
///                     .as_string(&token),
///                 "java.lang.String",
///             );
///             let result = String::value_of_int(env, &token, argument)
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
///     string_class.env().raw_env().as_ptr(),
///     string_class.raw_object().as_ptr(),
///     17,
/// );
/// assert_ne!(string, ptr::null_mut());
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer or an invalid [`jclass`](../jni_sys/type.jclass.html).
pub unsafe fn static_native_method_implementation<R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    callback: F,
) -> R::JniType
where
    // TODO(monnoroch): find out how to do this without allocation.
    // See https://stackoverflow.com/questions/59003532/using-higher-ranked-trait-bounds-with-generics.
    for<'a> F: FnOnce(
            &'a Class<'a>,
            NoException<'a>,
        ) -> (
            JavaResult<'a, Box<dyn NativeMethodResult<JniType = R::JniType> + 'a>>,
            NoException<'a>,
        ) + std::panic::UnwindSafe,
    R: NativeMethodResult,
{
    generic_native_method_implementation::<R, _>(raw_env, |env, token| {
        // Should not panic if the class pointer is valid.
        let class = Class::from_raw(&env, NonNull::new(raw_class).unwrap());
        let (result, token) = callback(&class, token);
        let result = to_jni_type::<R>(result, token);
        // We don't own the reference.
        mem::forget(class);
        result
    })
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
/// # use rust_jni::java::lang::String;
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
/// unsafe extern "C" fn Java_java_lang_String_valueOf__I(
///     raw_env: *mut jni_sys::JNIEnv,
///     raw_object: jni_sys::jobject,
///     argument: jni_sys::jint,
/// ) -> jni_sys::jstring {
///     native_method_implementation::<String, _>(
///         raw_env,
///         raw_object,
///         |object, token| {
///             let env = object.env();
///             assert_eq!(
///                 object
///                     .class(&token)
///                     .get_name(&token)
///                     .or_npe(env, &token)
///                     .unwrap()
///                     .as_string(&token),
///                 "java.lang.String",
///             );
///             let result = String::value_of_int(env, &token, argument)
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
/// let string = String::empty(env, &token)?;
/// let string = Java_java_lang_String_valueOf__I(
///     string.env().raw_env().as_ptr(),
///     string.raw_object().as_ptr(),
///     17,
/// );
/// assert_ne!(string, ptr::null_mut());
/// # }
/// # Ok(token)
/// # }
/// ```
///
/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer or an invalid [`jobject`](../jni_sys/type.jobject.html).
pub unsafe fn native_method_implementation<R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    callback: F,
) -> R::JniType
where
    // TODO(monnoroch): find out how to do this without allocation.
    // See https://stackoverflow.com/questions/59003532/using-higher-ranked-trait-bounds-with-generics.
    for<'a> F: FnOnce(
            &'a Object<'a>,
            NoException<'a>,
        ) -> (
            JavaResult<'a, Box<dyn NativeMethodResult<JniType = R::JniType> + 'a>>,
            NoException<'a>,
        ) + std::panic::UnwindSafe,
    R: NativeMethodResult,
{
    generic_native_method_implementation::<R, _>(raw_env, |env, token| {
        // Should not panic if the class pointer is valid.
        let object = Object::from_raw(&env, NonNull::new(raw_object).unwrap());
        let (result, token) = callback(&object, token);
        let result = to_jni_type::<R>(result, token);
        // We don't own the reference.
        mem::forget(object);
        result
    })
}

/// This function is unsafe because it is possible to pass an invalid [`JNIEnv`](../jni_sys/type.JNIEnv.html)
/// pointer.
unsafe fn generic_native_method_implementation<R, F>(
    raw_env: *mut jni_sys::JNIEnv,
    callback: F,
) -> R::JniType
where
    for<'a> F: FnOnce(&'a JniEnv<'a>, NoException<'a>) -> R::JniType + std::panic::UnwindSafe,
    R: NativeMethodResult,
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
        let result = callback(&env, token);
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
            R::JniType::default()
        }
    }
}

fn to_jni_type<'a, R>(
    result: JavaResult<'a, Box<dyn NativeMethodResult<JniType = R::JniType> + 'a>>,
    token: NoException<'a>,
) -> R::JniType
where
    R: NativeMethodResult + 'a,
{
    match result {
        Ok(result) => {
            mem::forget(token);
            let java_result = result.to_jni();
            // Here we want to free memory of the Box, but don't want to run the destructor of the boxed value.
            // Running the destructor for primitive types won't do anything, but running the destructor
            // for a Java class wrapper will delete it's reference, which will make Java delete the object.
            // Here we could use mem:;forget(result), but that would leak the Box-es memory, which we don't want.
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