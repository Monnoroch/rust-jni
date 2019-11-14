use crate::class::Class;
use crate::env::JniEnv;
use crate::jni_bool;
use crate::method_calls::call_method;
use crate::result::JavaResult;
use crate::string::String;
use crate::token::{from_nullable, NoException};
use crate::traits::{Cast, FromJni, JavaType, ToJni};
use jni_sys;
use std;
use std::fmt;
use std::ptr;

include!("call_jni_method.rs");
include!("generate_class.rs");

/// A type representing the
/// [`java.lang.Object`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html) class
/// -- the root class of Java's class hierarchy.
///
/// [`Object` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html)
// TODO: examples.
pub struct Object<'env> {
    env: &'env JniEnv<'env>,
    raw_object: jni_sys::jobject,
}

// [`Object`](struct.Object.html) can't be passed between threads.
// TODO(https://github.com/rust-lang/rust/issues/13231): enable when !Send is stable.
// impl<'env> !Send for Object<'env> {}
// impl<'env> !Sync for Object<'env> {}

impl<'env> Object<'env> {
    /// Get the raw object pointer.
    ///
    /// This function provides low-level access to the Java object and thus is unsafe.
    pub unsafe fn raw_object(&self) -> jni_sys::jobject {
        self.raw_object
    }

    /// Get the [`JniEnv`](../../struct.JniEnv.html) this object is bound to.
    pub fn env(&self) -> &'env JniEnv<'env> {
        self.env
    }

    /// Get the object's class.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getobjectclass)
    pub fn class(&self, _token: &NoException) -> Class<'env> {
        // Safe because arguments are ensured to be correct references by construction.
        let raw_java_class = unsafe { call_jni_method!(self.env, GetObjectClass, self.raw_object) };
        if raw_java_class == ptr::null_mut() {
            panic!("Object {:?} doesn't have a class.", self.raw_object);
        }
        // Safe because the argument is ensured to be correct references by construction.
        unsafe { Class::__from_jni(self.env, raw_java_class) }
    }

    /// Compare with another Java object by reference.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#issameobject)
    pub fn is_same_as(&self, other: &Object, _token: &NoException) -> bool {
        // Safe because arguments are ensured to be correct references by construction.
        let same = unsafe {
            call_jni_method!(
                self.env(),
                IsSameObject,
                self.raw_object(),
                other.raw_object()
            )
        };
        jni_bool::to_rust(same)
    }

    /// Check if the object is an instance of the class.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#isinstanceof)
    pub fn is_instance_of(&self, class: &Class, _token: &NoException) -> bool {
        // Safe because arguments are ensured to be correct references by construction.
        let is_instance = unsafe {
            call_jni_method!(
                self.env(),
                IsInstanceOf,
                self.raw_object(),
                class.raw_object()
            )
        };
        jni_bool::to_rust(is_instance)
    }

    /// Clone the [`Object`](struct.Object.html). This is not a deep clone of the Java object,
    /// but a Rust-like clone of the value. Since Java objects are reference counted, this will
    /// increment the reference count.
    ///
    /// This method has a different signature from the one in the
    /// [`Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html) trait because
    /// cloning a Java object is only safe when there is no pending exception and because
    /// cloning a java object cat throw an exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newlocalref)
    pub fn clone(&self, token: &NoException<'env>) -> JavaResult<'env, Object<'env>> {
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewLocalRef` throws an exception before returning `null`.
        let raw_object =
            unsafe { call_nullable_jni_method!(self.env, token, NewLocalRef, self.raw_object)? };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(self.env, raw_object) })
    }

    /// Construct from a raw pointer. Unsafe because an invalid pointer may be passed
    /// as the argument.
    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_raw(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Self {
        Self { env, raw_object }
    }
}

object_java_class!(
    Object,
    "[`Object`](struct.Object.html)",
    constructors = (),
    methods = (
        doc = "Convert the object to a string.",
        link = "[`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())",
        java_name = "toString",
        to_string() -> String<'env>,
        doc = "Compare to another Java object.",
        link = "[`Object::equals`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#equals(java.lang.Object))",
        java_name = "equals",
        equals(other: &Object) -> bool,
    ),
);

/// Make [`Object`](struct.Object.html) convertible from
/// [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
impl<'env> FromJni<'env> for Object<'env> {
    unsafe fn __from_jni(env: &'env JniEnv<'env>, value: Self::__JniType) -> Self {
        Self::from_raw(env, value)
    }
}

/// Make [`Object`](struct.Object.html)-s reference be deleted when the value is dropped.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#deletelocalref)
impl<'env> Drop for Object<'env> {
    fn drop(&mut self) {
        // Safe because the argument is ensured to be correct references by construction.
        unsafe {
            call_jni_method!(self.env, DeleteLocalRef, self.raw_object);
        }
    }
}

/// Allow comparing [`Object`](struct.Object.html) to Java objects. Java objects are compared
/// by-reference to preserve original Java semantics. To compare objects by value, call the
/// [`equals`](struct.Object.html#method.equals) method.
///
/// Will panic if there is a pending exception in the current thread.
///
/// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
/// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env, T> PartialEq<T> for Object<'env>
where
    T: Cast<'env, Object<'env>>,
{
    fn eq(&self, other: &T) -> bool {
        if self.env().has_exception() {
            panic!("Comparing Java objects with a pending exception in the current thread")
        } else {
            // Safe because we checked that there is no pending exception.
            let token = unsafe { NoException::new(self.env()) };
            self.is_same_as(other.cast(), &token)
        }
    }
}

/// Allow displaying Java objects for debug purposes.
///
/// [`Object::toString`](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
///
/// This is mostly a convenience for debugging. Always prefer using
/// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env> fmt::Debug for Object<'env> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.env.has_exception() {
            // Can't call `to_string` with a pending exception.
            write!(
                formatter,
                "Object {{ env: {:?}, object: {:?}, string: \
                 <can't call Object::toString string because of a pending exception in the current thread> }}",
                self.env, self.raw_object
            )
        } else {
            // Safe because we checked that there is no pending exception.
            let token = unsafe { NoException::new(self.env) };
            match self.to_string(&token) {
                Ok(string) => write!(
                    formatter,
                    "Object {{ env: {:?}, object: {:?} string: {} }}",
                    self.env,
                    self.raw_object,
                    string.as_string(&token),
                ),
                Err(exception) => match exception.to_string(&token) {
                    Ok(message) => write!(
                        formatter,
                        "Object {{ env: {:?}, object: {:?}, string: \
                         <Object::toString threw an exception: {:?}> }}",
                        self.env,
                        self.raw_object,
                        message.as_string(&token)
                    ),
                    Err(_) => write!(
                        formatter,
                        "Object {{ env: {:?}, object: {:?}, string: \
                         <Object::toString threw an exception> }}",
                        self.env, self.raw_object
                    ),
                },
            }
        }
    }
}

/// Allow displaying Java objects.
///
/// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
///
/// This is mostly a convenience for debugging. Always prefer using
/// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
/// the former checks for a pending exception in compile-time rather than the run-time.
impl<'env> fmt::Display for Object<'env> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.env.has_exception() {
            panic!("Displaying a Java object with a pending exception in the current thread.");
        } else {
            // Safe because we checked that there is no pending exception.
            let token = unsafe { NoException::new(self.env) };
            match self.to_string(&token) {
                Ok(string) => write!(formatter, "{}", string.as_string(&token)),
                Err(exception) => match exception.to_string(&token) {
                    Ok(message) => write!(
                        formatter,
                        "Object::toString threw an exception: {}",
                        message.as_string(&token)
                    ),
                    Err(_) => write!(
                        formatter,
                        "<Object::toString threw an exception which could not be formatted>"
                    ),
                },
            }
        }
    }
}

#[cfg(test)]
pub fn test_object<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Object<'env> {
    Object { env, raw_object }
}

#[cfg(test)]
mod object_tests {
    use super::*;
    use crate::class::test_class;
    use crate::env::test_env;
    use crate::testing::*;
    use crate::vm::test_vm;
    use std::mem;

    #[cfg(test)]
    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Object<'env> {
        test_object(env, raw_object)
    }

    generate_object_tests!(Object, "Ljava/lang/Object;");

    #[test]
    fn raw_object() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            assert_eq!(object.raw_object(), raw_object);
        }
        mem::forget(object);
    }

    #[test]
    fn env() {
        let vm = test_vm(ptr::null_mut());
        let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
        let env = test_env(&vm, jni_env);
        let raw_object = 0x91011 as jni_sys::jobject;
        let object = test_object(&env, raw_object);
        unsafe {
            assert_eq!(object.env().raw_env(), jni_env);
        }
        mem::forget(object);
    }

    #[test]
    fn cast() {
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, ptr::null_mut());
        let object = test_value(&env, ptr::null_mut());
        assert_eq!(&object as *const _, object.cast() as *const _);
        mem::forget(object);
    }

    #[test]
    fn class() {
        const RAW_OBJECT: jni_sys::jobject = 0x093599 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x347658 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetObjectClass(GetObjectClass {
            object: RAW_OBJECT,
            result: RAW_CLASS,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        let class = object.class(&NoException::test());
        calls.assert_eq(&class, RAW_CLASS);
    }

    #[test]
    #[should_panic(expected = "doesn't have a class")]
    fn class_not_found() {
        const RAW_OBJECT: jni_sys::jobject = 0x093599 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetObjectClass(GetObjectClass {
            object: RAW_OBJECT,
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        object.class(&NoException::test());
    }

    #[test]
    fn is_same_as_same() {
        const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsSameObject(IsSameObject {
            object1: RAW_OBJECT1,
            object2: RAW_OBJECT2,
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object1 = test_value(&env, RAW_OBJECT1);
        let object2 = test_value(&env, RAW_OBJECT2);
        assert!(object1.is_same_as(&object2, &NoException::test()));
    }

    #[test]
    fn is_same_as_not_same() {
        const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsSameObject(IsSameObject {
            object1: RAW_OBJECT1,
            object2: RAW_OBJECT2,
            result: jni_sys::JNI_FALSE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object1 = test_value(&env, RAW_OBJECT1);
        let object2 = test_value(&env, RAW_OBJECT2);
        assert!(!object1.is_same_as(&object2, &NoException::test()));
    }

    #[test]
    fn is_instance_of() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsInstanceOf(IsInstanceOf {
            object: RAW_OBJECT,
            class: RAW_CLASS,
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_object(&env, RAW_OBJECT);
        let class = test_class(&env, RAW_CLASS);
        assert!(object.is_instance_of(&class, &NoException::test()));
    }

    #[test]
    fn is_not_instance_of() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x93486 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsInstanceOf(IsInstanceOf {
            object: RAW_OBJECT,
            class: RAW_CLASS,
            result: jni_sys::JNI_FALSE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_object(&env, RAW_OBJECT);
        let class = test_class(&env, RAW_CLASS);
        assert!(!object.is_instance_of(&class, &NoException::test()));
    }

    #[test]
    fn debug() {
        const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
        const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
        const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
        const LENGTH: usize = 5;
        const SIZE: usize = 11; // `"test-string".len()`.
        static mut METHOD_CALLS: i32 = 0;
        static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        type VariadicFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            ...
        ) -> jni_sys::jstring;
        type TestFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jstring;
        unsafe extern "C" fn method(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jstring {
            assert_eq!(object, RAW_OBJECT);
            assert_eq!(method_id, METHOD_ID);
            METHOD_CALLS += 1;
            METHOD_ENV_ARGUMENT = env;
            RAW_STRING
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ..empty_raw_jni_env()
        };
        let calls = test_raw_jni_env!(
            vec![
                JniCall::ExceptionCheck(ExceptionCheck {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::GetObjectClass(GetObjectClass {
                    object: RAW_OBJECT,
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "toString".to_owned(),
                    signature: "()Ljava/lang/String;".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred {
                    result: ptr::null_mut(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                JniCall::GetStringLength(GetStringLength {
                    string: RAW_STRING,
                    result: LENGTH as jni_sys::jsize,
                }),
                JniCall::GetStringUTFLength(GetStringUTFLength {
                    string: RAW_STRING,
                    result: SIZE as jni_sys::jsize,
                }),
                JniCall::GetStringUTFRegion(GetStringUTFRegion {
                    string: RAW_STRING,
                    start: 0,
                    len: LENGTH as jni_sys::jsize,
                    buffer: "test-string".to_owned(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_STRING }),
            ],
            raw_jni_env
        );
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        assert!(format!("{:?}", object).contains("string: test-string"));
    }

    #[test]
    fn debug_exception_pending() {
        let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, ptr::null_mut());
        assert!(format!("{:?}", object).contains(
            "string: <can't call Object::toString string \
             because of a pending exception in the current thread>",
        ));
    }

    #[test]
    fn debug_exception_thrown() {
        const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
        const RAW_EXCEPTION_CLASS: jni_sys::jobject = 0x912376 as jni_sys::jobject;
        const METHOD_ID: jni_sys::jmethodID = 0x923476 as jni_sys::jmethodID;
        const EXCEPTION_METHOD_ID: jni_sys::jmethodID = 0x8293659 as jni_sys::jmethodID;
        const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const LENGTH: usize = 5;
        const SIZE: usize = 11; // `"test-string".len()`.
        static mut METHOD_CALLS: i32 = 0;
        static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        type VariadicFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            ...
        ) -> jni_sys::jstring;
        type TestFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jstring;
        unsafe extern "C" fn method(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jstring {
            METHOD_CALLS += 1;
            if METHOD_CALLS == 1 {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_ENV_ARGUMENT = env;
            } else {
                assert_eq!(object, EXCEPTION);
                assert_eq!(method_id, EXCEPTION_METHOD_ID);
                assert_eq!(env, METHOD_ENV_ARGUMENT);
            }
            RAW_STRING
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ..empty_raw_jni_env()
        };
        let calls = test_raw_jni_env!(
            vec![
                JniCall::ExceptionCheck(ExceptionCheck {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::GetObjectClass(GetObjectClass {
                    object: RAW_OBJECT,
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "toString".to_owned(),
                    signature: "()Ljava/lang/String;".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                JniCall::GetObjectClass(GetObjectClass {
                    object: EXCEPTION,
                    result: RAW_EXCEPTION_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_EXCEPTION_CLASS,
                    name: "toString".to_owned(),
                    signature: "()Ljava/lang/String;".to_owned(),
                    result: EXCEPTION_METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred {
                    result: ptr::null_mut(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef {
                    object: RAW_EXCEPTION_CLASS,
                }),
                JniCall::GetStringLength(GetStringLength {
                    string: RAW_STRING,
                    result: LENGTH as jni_sys::jsize,
                }),
                JniCall::GetStringUTFLength(GetStringUTFLength {
                    string: RAW_STRING,
                    result: SIZE as jni_sys::jsize,
                }),
                JniCall::GetStringUTFRegion(GetStringUTFRegion {
                    string: RAW_STRING,
                    start: 0,
                    len: LENGTH as jni_sys::jsize,
                    buffer: "test-string".to_owned(),
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_STRING }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION }),
            ],
            raw_jni_env
        );
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        assert!(format!("{:?}", object)
            .contains("string: <Object::toString threw an exception: \"test-string\">"));
    }

    #[test]
    fn debug_exception_thrown_twice() {
        const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
        const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
        const RAW_EXCEPTION_CLASS: jni_sys::jobject = 0x912376 as jni_sys::jobject;
        const METHOD_ID: jni_sys::jmethodID = 0x923476 as jni_sys::jmethodID;
        const EXCEPTION_METHOD_ID: jni_sys::jmethodID = 0x8293659 as jni_sys::jmethodID;
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const EXCEPTION2: jni_sys::jobject = 0x2836 as jni_sys::jobject;
        static mut METHOD_CALLS: i32 = 0;
        static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
        type VariadicFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
            ...
        ) -> jni_sys::jstring;
        type TestFn = unsafe extern "C" fn(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jstring;
        unsafe extern "C" fn method(
            env: *mut jni_sys::JNIEnv,
            object: jni_sys::jobject,
            method_id: jni_sys::jmethodID,
        ) -> jni_sys::jstring {
            METHOD_CALLS += 1;
            if METHOD_CALLS == 1 {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_ENV_ARGUMENT = env;
            } else {
                assert_eq!(object, EXCEPTION);
                assert_eq!(method_id, EXCEPTION_METHOD_ID);
                assert_eq!(env, METHOD_ENV_ARGUMENT);
            }
            ptr::null_mut()
        }
        let raw_jni_env = jni_sys::JNINativeInterface_ {
            CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
            ..empty_raw_jni_env()
        };
        let calls = test_raw_jni_env!(
            vec![
                JniCall::ExceptionCheck(ExceptionCheck {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::GetObjectClass(GetObjectClass {
                    object: RAW_OBJECT,
                    result: RAW_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_CLASS,
                    name: "toString".to_owned(),
                    signature: "()Ljava/lang/String;".to_owned(),
                    result: METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                JniCall::GetObjectClass(GetObjectClass {
                    object: EXCEPTION,
                    result: RAW_EXCEPTION_CLASS,
                }),
                JniCall::GetMethodID(GetMethodID {
                    class: RAW_EXCEPTION_CLASS,
                    name: "toString".to_owned(),
                    signature: "()Ljava/lang/String;".to_owned(),
                    result: EXCEPTION_METHOD_ID,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION2 }),
                JniCall::ExceptionClear(ExceptionClear {}),
                JniCall::DeleteLocalRef(DeleteLocalRef {
                    object: RAW_EXCEPTION_CLASS,
                }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION2 }),
                JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION }),
            ],
            raw_jni_env
        );
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let object = test_value(&env, RAW_OBJECT);
        assert!(format!("{:?}", object).contains("string: <Object::toString threw an exception>"));
    }
}
