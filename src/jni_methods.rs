use crate::class::Class;
use crate::java_string::{
    to_java_string_null_terminated, to_java_string_null_terminated_unchecked,
};
use crate::jni_types::private::{JniArgumentTypeTuple, JniPrimitiveType, JniType};
use crate::object::Object;
use crate::result::JavaResult;
use crate::token::{CallOutcome, NoException};
use core::ptr::NonNull;
use std::os::raw::c_char;

include!("call_jni_method.rs");

/// Unsafe because signature must be null-terminated.
unsafe fn get_method_id<'a>(
    class: &Class<'a>,
    token: &NoException<'a>,
    name: &str,
    signature: &str,
) -> JavaResult<'a, NonNull<jni_sys::_jmethodID>> {
    let name = to_java_string_null_terminated(name);
    let signature = to_java_string_null_terminated_unchecked(signature);
    // Safe because arguments are ensured to be the correct by construction and because
    // `GetMethodID` throws an exception before returning `null`.
    #[allow(unused_unsafe)]
    unsafe {
        call_nullable_jni_method!(
            class.env(),
            token,
            GetMethodID,
            class.raw_object().as_ptr(),
            name.as_ptr() as *const c_char,
            signature.as_ptr() as *const c_char
        )
    }
}

/// Unsafe because signature must be null-terminated.
unsafe fn get_static_method_id<'a>(
    class: &Class<'a>,
    token: &NoException<'a>,
    name: &str,
    signature: &str,
) -> JavaResult<'a, NonNull<jni_sys::_jmethodID>> {
    let name = to_java_string_null_terminated(name);
    let signature = to_java_string_null_terminated_unchecked(signature);
    // Safe because arguments are ensured to be the correct by construction and because
    // `GetMethodID` throws an exception before returning `null`.
    #[allow(unused_unsafe)]
    unsafe {
        call_nullable_jni_method!(
            class.env(),
            token,
            GetStaticMethodID,
            class.raw_object().as_ptr(),
            name.as_ptr() as *const c_char,
            signature.as_ptr() as *const c_char
        )
    }
}

/// Call a method on a Java object that returns a primitive value.
///
/// Unsafe because it is possible to pass incorrect arguments or return type.
pub(crate) unsafe fn call_primitive_method<'a, R: JniPrimitiveType>(
    object: &Object<'a>,
    token: &NoException<'a>,
    name: &str,
    signature: &str,
    arguments: impl JniArgumentTypeTuple,
) -> JavaResult<'a, R> {
    let class = object.class(token);
    let method_id = get_method_id(&class, token, name, signature)?;
    token.with_owned(
        class.env(),
        #[inline(always)]
        |_token| CallOutcome::Unknown(R::call_method(object, method_id.as_ptr(), arguments)),
    )
}

/// Call a method on a Java object that returns another object.
///
/// Unsafe because it is possible to pass incorrect arguments or return type.
pub(crate) unsafe fn call_object_method<'a>(
    object: &Object<'a>,
    token: &NoException<'a>,
    name: &str,
    signature: &str,
    arguments: impl JniArgumentTypeTuple,
) -> JavaResult<'a, Option<NonNull<jni_sys::_jobject>>> {
    let class = object.class(token);
    let method_id = get_method_id(&class, token, name, signature)?;
    token.with_owned(
        class.env(),
        #[inline(always)]
        |token| {
            let result = jni_sys::jobject::call_method(object, method_id.as_ptr(), arguments);
            match NonNull::new(result) {
                None => CallOutcome::Unknown(None),
                result => CallOutcome::Ok((result, token)),
            }
        },
    )
}

/// Call a static method on a Java class that returns a primitive value.
///
/// Unsafe because it is possible to pass incorrect arguments or return type.
pub(crate) unsafe fn call_static_primitive_method<'a, R: JniPrimitiveType>(
    class: &Class<'a>,
    token: &NoException<'a>,
    name: &str,
    signature: &str,
    arguments: impl JniArgumentTypeTuple,
) -> JavaResult<'a, R> {
    let method_id = get_static_method_id(&class, token, name, signature)?;
    token.with_owned(
        class.env(),
        #[inline(always)]
        |_token| CallOutcome::Unknown(R::call_static_method(class, method_id.as_ptr(), arguments)),
    )
}

/// Call a static method on a Java object that returns another object.
///
/// Unsafe because it is possible to pass incorrect arguments or return type.
pub(crate) unsafe fn call_static_object_method<'a>(
    class: &Class<'a>,
    token: &NoException<'a>,
    name: &str,
    signature: &str,
    arguments: impl JniArgumentTypeTuple,
) -> JavaResult<'a, Option<NonNull<jni_sys::_jobject>>> {
    let method_id = get_static_method_id(&class, token, name, signature)?;
    token.with_owned(
        class.env(),
        #[inline(always)]
        |token| {
            let result = jni_sys::jobject::call_static_method(class, method_id.as_ptr(), arguments);
            match NonNull::new(result) {
                None => CallOutcome::Unknown(None),
                result => CallOutcome::Ok((result, token)),
            }
        },
    )
}

/// Call a constructor of a Java class.
///
/// Unsafe because it is possible to pass incorrect arguments.
pub(crate) unsafe fn call_constructor<'a, A: JniArgumentTypeTuple>(
    class: &Class<'a>,
    token: &NoException<'a>,
    signature: &str,
    arguments: A,
) -> JavaResult<'a, NonNull<jni_sys::_jobject>> {
    let method_id = get_method_id(&class, token, "<init>\0", signature)?;
    token.with_owned(
        class.env(),
        #[inline(always)]
        |token| {
            let result = A::call_constructor(class, method_id.as_ptr(), arguments);
            match NonNull::new(result) {
                None => CallOutcome::Err(token.exchange(class.env())),
                Some(value) => CallOutcome::Ok((value, token)),
            }
        },
    )
}
