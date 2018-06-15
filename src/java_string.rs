/// Java uses
/// [modified UTF-8 strings](https://docs.oracle.com/javase/10/docs/specs/jni/types.html#modified-utf-8-strings).
/// JNI in addition uses null-terminated modified UTF-8 strings.
/// The `cesu8` crate provides tools for regular CESU-8 strings, not null-terminated. This module
/// uses the `cesu8` crate to provide tools for mapping Rust UTF-8 strings and
/// JNI null-terminated CESU-8 strings.
use cesu8::{self, Cesu8DecodingError};
use std::borrow::Cow;
use std::slice;

/// Convert a Rust UTF-8 string into a buffer with a Java modified UTF-8 string.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/types.html#modified-utf-8-strings)
pub fn to_java_string(string: &str) -> Vec<u8> {
    let mut buffer = cesu8::to_java_cesu8(string).into_owned();
    buffer.push(0);
    buffer
}

#[cfg(test)]
mod to_java_string_tests {
    use super::*;

    #[test]
    fn to() {
        assert_eq!(
            to_java_string("test"),
            vec!['t' as u8, 'e' as u8, 's' as u8, 't' as u8, 0]
        );
    }
}

/// Convert a buffer with a Java modified UTF-8 string into a Rust UTF-8 string.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/types.html#modified-utf-8-strings)
pub fn from_java_string(buffer: &[u8]) -> Result<Cow<str>, Cesu8DecodingError> {
    cesu8::from_java_cesu8(unsafe {
        slice::from_raw_parts(buffer.as_ptr() as *const u8, buffer.len() - 1)
    })
}

#[cfg(test)]
mod from_java_string_tests {
    use super::*;

    #[test]
    fn from() {
        let buffer = vec!['t' as u8, 'e' as u8, 's' as u8, 't' as u8, 0];
        assert_eq!(from_java_string(&buffer).unwrap(), "test");
    }

    #[test]
    fn back_and_forth() {
        let string = vec!['t' as u8, 'e' as u8, 's' as u8, 't' as u8, 0];
        assert_eq!(to_java_string(&*from_java_string(&string).unwrap()), string);
    }

    #[test]
    fn forth_and_back() {
        let string = "test";
        assert_eq!(from_java_string(&to_java_string(string)).unwrap(), string);
    }
}
