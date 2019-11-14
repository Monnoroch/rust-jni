use crate::jni::class::Class;
use crate::jni::method_calls::call_constructor;
use crate::jni::method_calls::call_method;
use crate::jni::string::String;
use crate::jni::*;
use jni_sys;
use std::fmt;

include!("call_jni_method.rs");
include!("generate_class.rs");

/// A type representing a Java
/// [`Throwable`](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html).
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct Throwable<'env> {
    object: Object<'env>,
}

impl<'env> Throwable<'env> {
    /// Throw the exception. Transfers ownership of the object to Java.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#throw)
    pub fn throw(self, token: NoException) -> Exception<'env> {
        // Safe because the argument is ensured to be correct references by construction.
        let error = JniError::from_raw(unsafe {
            call_jni_method!(self.env(), Throw, self.raw_object() as jni_sys::jthrowable)
        });
        // Can't really handle failing throwing an exception.
        if error.is_some() {
            panic!(
                "Throwing an exception has failed with status {:?}.",
                error.unwrap()
            );
        }
        // Safe becuase we just threw the exception.
        unsafe { Exception::new(self.env(), token) }
    }
}

java_class!(
    Throwable,
    "[`Throwable`](struct.Throwable.html)",
    constructors = (
        doc = "Create a new [`Throwable`](struct.Throwable.html) with a message.",
        link = "[`Throwable(String)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#<init>(java.lang.String))",
        new(message: &String<'env>),
    ),
    methods = (
        doc = "Get the exception message.",
        link = "[`Throwable::getMessage` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Throwable.html#getMessage()).",
        java_name = "getMessage",
        get_message() -> String<'env>,
    ),
    static_methods = (),
);

#[cfg(test)]
pub fn test_throwable<'env>(
    env: &'env JniEnv<'env>,
    raw_object: jni_sys::jobject,
) -> Throwable<'env> {
    Throwable {
        object: test_object(env, raw_object),
    }
}

#[cfg(test)]
mod throwable_tests {
    use super::*;
    use crate::jni::testing::*;
    use std::mem;
    use std::ops::Deref;
    use std::ptr;

    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Throwable<'env> {
        test_throwable(env, raw_object)
    }

    generate_tests!(Throwable, "Ljava/lang/Throwable;");

    #[test]
    fn throw() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::Throw(Throw {
            object: RAW_OBJECT,
            result: jni_sys::JNI_OK,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        object.throw(NoException::test());
    }

    #[test]
    #[should_panic(expected = "Throwing an exception has failed with status Unknown(-1).")]
    fn throw_failed() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::Throw(Throw {
            object: RAW_OBJECT,
            result: jni_sys::JNI_ERR,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        object.throw(NoException::test());
    }
}
