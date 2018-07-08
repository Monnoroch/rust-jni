use java_string::*;
use jni::method_calls::call_static_method;
use jni::*;
use jni_sys;
use std;
use std::fmt;
use std::os::raw::c_char;
use std::ptr;

include!("call_jni_method.rs");
include!("generate_class.rs");

/// A type representing a Java
/// [`String`](https://docs.oracle.com/javase/10/docs/api/java/lang/String.html).
// TODO: examples.
// TODO: custom debug.
#[derive(Debug)]
pub struct String<'env> {
    object: Object<'env>,
}

impl<'env> String<'env> {
    /// Create a new empty string.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newstring)
    pub fn empty<'a>(env: &'a JniEnv<'a>, token: &NoException<'a>) -> JavaResult<'a, String<'a>> {
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewString` throws an exception before returning `null`.
        let raw_string = unsafe {
            call_nullable_jni_method!(env, NewString, token, ptr::null(), 0 as jni_sys::jsize)?
        };
        // Safe because the argument is a valid string reference.
        Ok(unsafe { Self::from_raw(env, raw_string) })
    }

    /// Create a new Java string from a Rust string.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newstringutf)
    pub fn new<'a>(
        env: &'a JniEnv<'a>,
        string: &str,
        token: &NoException<'a>,
    ) -> JavaResult<'a, String<'a>> {
        if string.is_empty() {
            return Self::empty(env, token);
        }

        let buffer = to_java_string(string);
        // Safe because arguments are ensured to be the correct by construction and because
        // `NewStringUTF` throws an exception before returning `null`.
        let raw_string = unsafe {
            call_nullable_jni_method!(env, NewStringUTF, token, buffer.as_ptr() as *const c_char)?
        };
        // Safe because the argument is a valid string reference.
        Ok(unsafe { Self::from_raw(env, raw_string) })
    }

    /// String length (the number of unicode characters).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringlength)
    pub fn len(&self, _token: &NoException) -> usize {
        // Safe because arguments are ensured to be the correct by construction.
        let length = unsafe {
            call_jni_method!(
                self.env(),
                GetStringLength,
                self.raw_object() as jni_sys::jstring
            )
        };
        length as usize
    }

    /// String size (the number of bytes in modified UTF-8).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringutflength)
    pub fn size(&self, _token: &NoException) -> usize {
        // Safe because arguments are ensured to be the correct by construction.
        let size = unsafe {
            call_jni_method!(
                self.env(),
                GetStringUTFLength,
                self.raw_object() as jni_sys::jstring
            )
        };
        size as usize
    }

    /// Convert the Java `String` into a Rust `String`.
    ///
    /// This method has a different signature from the one in the `ToString` trait because
    /// extracting bytes from `String` is only safe when there is no pending exception.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#getstringutfregion)
    pub fn as_string(&self, token: &NoException) -> std::string::String {
        let length = self.len(token);
        if length == 0 {
            return "".to_owned();
        }

        let size = self.size(token) + 1; // +1 for the '\0' byte.
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        // Safe because arguments are ensured to be the correct by construction.
        unsafe {
            call_jni_method!(
                self.env(),
                GetStringUTFRegion,
                self.raw_object() as jni_sys::jstring,
                0 as jni_sys::jsize,
                length as jni_sys::jsize,
                buffer.as_mut_ptr() as *mut c_char
            );
            buffer.set_len(size);
        }
        from_java_string(buffer.as_slice()).unwrap().into_owned()
    }

    /// Unsafe because an incorrect object reference can be passed.
    unsafe fn from_raw<'a>(env: &'a JniEnv<'a>, raw_string: jni_sys::jstring) -> String<'a> {
        String {
            object: Object::__from_jni(env, raw_string as jni_sys::jobject),
        }
    }
}

java_class!(
    String,
    "[`String`](struct.String.html)",
    constructors = (),
    methods = (),
    static_methods = (
        doc = "Get the string value of an integer.",
        link = "[`String::valueOf(int)` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/String.html#valueOf(int))",
        java_name = "valueOf",
        value_of_int(value: i32) -> String<'env>,
    ),
);

#[cfg(test)]
mod string_tests {
    use super::*;
    use jni::testing::*;
    use std::mem;
    use std::ops::Deref;

    fn test_value<'env>(env: &'env JniEnv<'env>, raw_object: jni_sys::jobject) -> String<'env> {
        String {
            object: test_object(env, raw_object),
        }
    }

    generate_tests!(String, "Ljava/lang/String;");

    #[test]
    fn empty() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::NewString(NewString {
            name: ptr::null(),
            size: 0,
            result: RAW_STRING,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = String::empty(&env, &NoException::test()).unwrap();
        calls.assert_eq(&string, RAW_STRING);
    }

    #[test]
    fn empty_exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::NewString(NewString {
                name: ptr::null(),
                size: 0,
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = String::empty(&env, &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn new_empty() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::NewString(NewString {
            name: ptr::null(),
            size: 0,
            result: RAW_STRING,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = String::new(&env, "", &NoException::test()).unwrap();
        calls.assert_eq(&string, RAW_STRING);
    }

    #[test]
    fn new_empty_exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::NewString(NewString {
                name: ptr::null(),
                size: 0,
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = String::new(&env, "", &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn new() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::NewStringUTF(NewStringUTF {
            string: "test-string".to_owned(),
            result: RAW_STRING,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = String::new(&env, "test-string", &NoException::test()).unwrap();
        calls.assert_eq(&string, RAW_STRING);
    }

    #[test]
    fn new_exception() {
        const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
            JniCall::NewStringUTF(NewStringUTF {
                string: "test-string".to_owned(),
                result: ptr::null_mut(),
            }),
            JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
            JniCall::ExceptionClear(ExceptionClear {}),
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let exception = String::new(&env, "test-string", &NoException::test()).unwrap_err();
        calls.assert_eq(&exception, EXCEPTION);
    }

    #[test]
    fn len() {
        const LENGTH: usize = 17;
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetStringLength(GetStringLength {
            string: RAW_STRING,
            result: 17 as jni_sys::jsize,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.len(&NoException::test()), LENGTH);
    }

    #[test]
    fn size() {
        const LENGTH: usize = 17;
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetStringUTFLength(GetStringUTFLength {
            string: RAW_STRING,
            result: 17 as jni_sys::jsize,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.size(&NoException::test()), LENGTH);
    }

    #[test]
    fn as_string() {
        const LENGTH: usize = 5;
        const SIZE: usize = 11; // `"test-string".len()`.
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![
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
        ]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.as_string(&NoException::test()), "test-string");
    }

    #[test]
    fn as_string_empty() {
        const RAW_STRING: jni_sys::jobject = 0x2835 as jni_sys::jobject;
        let calls = test_raw_jni_env!(vec![JniCall::GetStringLength(GetStringLength {
            string: RAW_STRING,
            result: 0,
        })]);
        let vm = test_vm(ptr::null_mut());
        let env = test_env(&vm, calls.env);
        let string = unsafe { String::from_raw(&env, RAW_STRING) };
        assert_eq!(string.as_string(&NoException::test()), "");
    }
}
