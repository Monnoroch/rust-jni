use java_string::*;
use jni_sys;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr;
use version::{self, JniVersion};

/// Arguments for attaching a thread to the JVM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachArguments<'a> {
    thread_name: Option<&'a str>,
    // TODO(#7): support thread groups.
}

impl<'a> AttachArguments<'a> {
    /// Create attach arguments with the default thread name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn new() -> Self {
        AttachArguments { thread_name: None }
    }

    /// Create attach arguments with a specified thread name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn named(thread_name: &'a str) -> Self {
        AttachArguments {
            thread_name: Some(thread_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AttachArguments;

    #[test]
    fn new() {
        assert_eq!(
            AttachArguments::new(),
            AttachArguments { thread_name: None }
        );
    }

    #[test]
    fn named() {
        assert_eq!(
            AttachArguments::named("test-name"),
            AttachArguments {
                thread_name: Some("test-name")
            }
        );
    }
}

/// A wrapper around `jni_sys::JavaVMAttachArgs` with a lifetime to ensure
/// there's no access to freed memory.
pub struct RawAttachArguments<'a> {
    raw_arguments: jni_sys::JavaVMAttachArgs,
    buffer_len: usize,
    _buffer: PhantomData<&'a Vec<u8>>,
}

/// Convert `AttachArguments` to `jni_sys::JavaVMAttachArgs`. Uses a buffer for storing
/// the Java string with the thread name.
pub fn to_raw<'a>(
    arguments: &AttachArguments,
    version: JniVersion,
    buffer: &'a mut Vec<u8>,
) -> RawAttachArguments<'a> {
    let version = version::to_raw(version);
    let group = ptr::null_mut();
    let raw_arguments = jni_sys::JavaVMAttachArgs {
        name: match arguments.thread_name {
            None => ptr::null_mut(),
            Some(ref thread_name) => {
                *buffer = to_java_string(thread_name);
                buffer.as_ptr() as *mut c_char
            }
        },
        version,
        group,
    };
    RawAttachArguments {
        raw_arguments,
        buffer_len: buffer.len(),
        _buffer: PhantomData::<&'a Vec<u8>>,
    }
}

#[cfg(test)]
mod to_raw_tests {
    use super::*;
    use std::slice;

    #[test]
    fn to_raw() {
        let arguments = AttachArguments::new();
        let mut buffer: Vec<u8> = vec![];
        let raw_arguments = super::to_raw(&arguments, JniVersion::V8, &mut buffer);
        assert_eq!(raw_arguments.raw_arguments.group, ptr::null_mut());
        assert_eq!(raw_arguments.raw_arguments.name, ptr::null_mut());
        assert_eq!(
            raw_arguments.raw_arguments.version,
            version::to_raw(JniVersion::V8)
        );
    }

    #[test]
    fn to_raw_named() {
        let test_name = "test-name";
        let arguments = AttachArguments::named(test_name);
        let mut buffer: Vec<u8> = vec![];
        let raw_arguments = super::to_raw(&arguments, JniVersion::V8, &mut buffer);
        assert_eq!(raw_arguments.raw_arguments.group, ptr::null_mut());
        assert_eq!(
            raw_arguments.raw_arguments.version,
            version::to_raw(JniVersion::V8)
        );
        assert_eq!(
            from_java_string(unsafe {
                slice::from_raw_parts(
                    raw_arguments.raw_arguments.name as *const u8,
                    raw_arguments.buffer_len,
                )
            }).unwrap(),
            test_name
        );
    }
}
