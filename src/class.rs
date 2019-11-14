use crate::env::JniEnv;
use crate::java_string::*;
#[cfg(test)]
use crate::object::test_object;
use crate::object::Object;
use crate::result::JavaResult;
use crate::string::String;
use crate::token::{from_nullable, NoException};
use crate::traits::{Cast, FromJni, JavaType, ToJni};
use jni_sys;
use std::fmt;
use std::os::raw::c_char;
use std::ptr;

include!("call_jni_method.rs");
include!("generate_class.rs");

/// A type representing a Java
/// [`Class`](https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html).
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct Class<'env> {
    object: Object<'env>,
}

impl<'env> Class<'env> {
    /// Find an existing Java class by it's name. The name is a fully qualified class or array
    /// type name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#findclass)
    pub fn find<'a>(
        env: &'a JniEnv<'a>,
        class_name: &str,
        token: &NoException<'a>,
    ) -> JavaResult<'a, Class<'a>> {
        let class_name = to_java_string(class_name);
        // Safe because the arguments are correct and because `FindClass` throws an exception
        // before returning `null`.
        let raw_class = unsafe {
            call_nullable_jni_method!(env, token, FindClass, class_name.as_ptr() as *const c_char)?
        };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(env, raw_class) })
    }

    /// Define a new Java class from a `.class` file contents.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#defineclass)
    pub fn define<'a>(
        env: &'a JniEnv<'a>,
        bytes: &[u8],
        token: &NoException<'a>,
    ) -> JavaResult<'a, Class<'a>> {
        // Safe because the arguments are correct and because `DefineClass` throws an exception
        // before returning `null`.
        let raw_class = unsafe {
            call_nullable_jni_method!(
                env,
                token,
                DefineClass,
                ptr::null() as *const c_char,
                ptr::null_mut() as jni_sys::jobject,
                bytes.as_ptr() as *const jni_sys::jbyte,
                bytes.len() as jni_sys::jsize
            )?
        };
        // Safe because the argument is a valid class reference.
        Ok(unsafe { Self::from_raw(env, raw_class) })
    }

    /// Get the parent class of this class. Will return
    /// [`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None) for the
    /// [`Object`](struct.Object.html) class or any interface.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getsuperclass)
    pub fn parent(&self, _token: &NoException) -> Option<Class<'env>> {
        // Safe because the argument is ensured to be correct references by construction.
        let raw_java_class =
            unsafe { call_jni_method!(self.env(), GetSuperclass, self.raw_object()) };
        if raw_java_class == ptr::null_mut() {
            None
        } else {
            // Safe because the argument is ensured to be a correct reference.
            Some(unsafe { Self::__from_jni(self.env(), raw_java_class) })
        }
    }

    /// Check if this class is a subtype of the other class.
    ///
    /// In Java a class is a subtype of the other class if that other class is a direct or
    /// an indirect parent of this class or an interface this class or any it's parent is
    /// implementing.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#isassignablefrom)
    pub fn is_subtype_of(&self, class: &Class, _token: &NoException) -> bool {
        // Safe because arguments are ensured to be the correct by construction.
        let assignable = unsafe {
            call_jni_method!(
                self.env(),
                IsAssignableFrom,
                self.raw_object() as jni_sys::jclass,
                class.raw_object() as jni_sys::jclass
            )
        };
        // Safe because `bool` conversion is safe internally.
        unsafe { bool::__from_jni(self.env(), assignable) }
    }

    /// Unsafe because the argument mught not be a valid class reference.
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, raw_class: jni_sys::jclass) -> Class<'a> {
        Class {
            object: Object::__from_jni(env, raw_class as jni_sys::jobject),
        }
    }
}

java_class!(
    Class,
    "[`Class`](struct.Class.html)",
    constructors = (),
    methods = (),
    static_methods = (),
);

#[cfg(test)]
pub fn test_class<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Class<'env> {
    Class {
        object: test_object(env, raw_object),
    }
}

#[cfg(test)]
mod class_tests {
    use super::*;
    use crate::env::test_env;
    use crate::testing::*;
    use crate::vm::test_vm;
    use std::mem;
    use std::ops::Deref;

    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> Class<'env> {
        test_class(env, raw_object)
    }

    generate_tests!(Class, "Ljava/lang/Class;");

    #[test]
    fn find() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::FindClass(FindClass {
            name: "test-class".to_owned(),
            result: RAW_OBJECT,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = Class::find(&env, "test-class", &NoException::test()).unwrap();
        calls.assert_eq(&class, RAW_OBJECT);
    }

    #[test]
    fn find_not_found() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::FindClass(FindClass {
                name: "test-class".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = Class::find(&env, "test-class", &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn define() {
        const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::DefineClass(DefineClass {
            name: ptr::null(),
            loader: ptr::null_mut(),
            buffer: vec![17, (230 as u8) as i8],
            result: RAW_OBJECT,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = Class::define(&env, &[17, 230], &NoException::test()).unwrap();
        calls.assert_eq(&class, RAW_OBJECT);
    }

    #[test]
    fn define_not_found() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::DefineClass(DefineClass {
                name: ptr::null(),
                loader: ptr::null_mut(),
                buffer: vec![17, (230 as u8) as i8],
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = Class::define(&env, &[17, 230], &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn is_subtype_of() {
        const RAW_CLASS1: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const RAW_CLASS2: jni_sys::jobject = 0x294875 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsAssignableFrom(IsAssignableFrom {
            class1: RAW_CLASS1,
            class2: RAW_CLASS2,
            result: jni_sys::JNI_TRUE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class1 = test_class(&env, RAW_CLASS1);
        let class2 = test_class(&env, RAW_CLASS2);
        assert!(class1.is_subtype_of(&class2, &NoException::test()));
    }

    #[test]
    fn is_not_subtype_of() {
        const RAW_CLASS1: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const RAW_CLASS2: jni_sys::jobject = 0x294875 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::IsAssignableFrom(IsAssignableFrom {
            class1: RAW_CLASS1,
            class2: RAW_CLASS2,
            result: jni_sys::JNI_FALSE,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class1 = test_class(&env, RAW_CLASS1);
        let class2 = test_class(&env, RAW_CLASS2);
        assert!(!class1.is_subtype_of(&class2, &NoException::test()));
    }

    #[test]
    fn parent() {
        const RAW_CLASS1: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        const RAW_CLASS2: jni_sys::jobject = 0x294875 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetSuperclass(GetSuperclass {
            class: RAW_CLASS1,
            result: RAW_CLASS2,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = test_class(&env, RAW_CLASS1);
        let parent = class.parent(&NoException::test()).unwrap();
        calls.assert_eq(&parent, RAW_CLASS2);
    }

    #[test]
    fn no_parent() {
        const RAW_CLASS: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetSuperclass(GetSuperclass {
            class: RAW_CLASS,
            result: ptr::null_mut(),
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let class = test_class(&env, RAW_CLASS);
        assert!(class.parent(&NoException::test()).is_none());
    }
}
