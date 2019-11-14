use crate::init_arguments::InitArguments;
use crate::java_string::*;
use crate::version::JniVersion;
use jni_sys;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr;

/// Arguments for attaching a thread to the JVM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
///
/// # Example
/// ```
/// use rust_jni::{AttachArguments, InitArguments, JniVersion};
///
/// let options = InitArguments::get_default(JniVersion::V8).unwrap();
/// let attach_arguments = AttachArguments::new(&options);
///
/// assert_eq!(attach_arguments.version(), JniVersion::V8);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct AttachArguments<'a> {
    version: JniVersion,
    thread_name: Option<&'a str>,
    // TODO(#7): support thread groups.
}

impl<'a> AttachArguments<'a> {
    /// Create attach arguments with the default thread name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn new(init_arguments: &InitArguments) -> Self {
        AttachArguments {
            thread_name: None,
            version: init_arguments.version(),
        }
    }

    /// Create attach arguments with a specified thread name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn named(init_arguments: &InitArguments, thread_name: &'a str) -> Self {
        AttachArguments {
            thread_name: Some(thread_name),
            version: init_arguments.version(),
        }
    }

    /// Return the JNI version these arguments will request when attaching a thread to a Java VM.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn version(&self) -> JniVersion {
        self.version
    }
}

#[cfg(test)]
mod attach_arguments_tests {
    use super::*;

    #[test]
    fn new() {
        let init_arguments = InitArguments::default().with_version(JniVersion::V4);
        assert_eq!(
            AttachArguments::new(&init_arguments),
            AttachArguments {
                thread_name: None,
                version: JniVersion::V4
            }
        );
    }

    #[test]
    fn named() {
        let init_arguments = InitArguments::default().with_version(JniVersion::V4);
        assert_eq!(
            AttachArguments::named(&init_arguments, "test-name"),
            AttachArguments {
                thread_name: Some("test-name"),
                version: JniVersion::V4,
            }
        );
    }

    #[test]
    fn version() {
        let arguments = AttachArguments {
            version: JniVersion::V4,
            thread_name: None,
        };
        assert_eq!(arguments.version(), JniVersion::V4);
    }
}

/// A wrapper around `jni_sys::JavaVMAttachArgs` with a lifetime to ensure
/// there's no access to freed memory.
pub struct RawAttachArguments<'a> {
    pub raw_arguments: jni_sys::JavaVMAttachArgs,
    #[allow(dead_code)]
    buffer_len: usize,
    _buffer: PhantomData<&'a Vec<u8>>,
}

/// Convert `AttachArguments` to `jni_sys::JavaVMAttachArgs`. Uses a buffer for storing
/// the Java string with the thread name.
pub fn to_raw<'a>(arguments: &AttachArguments, buffer: &'a mut Vec<u8>) -> RawAttachArguments<'a> {
    let version = arguments.version.to_raw();
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
        let init_arguments = InitArguments::default().with_version(JniVersion::V8);
        let arguments = AttachArguments::new(&init_arguments);
        let mut buffer: Vec<u8> = vec![];
        let raw_arguments = super::to_raw(&arguments, &mut buffer);
        assert_eq!(raw_arguments.raw_arguments.group, ptr::null_mut());
        assert_eq!(raw_arguments.raw_arguments.name, ptr::null_mut());
        assert_eq!(raw_arguments.raw_arguments.version, JniVersion::V8.to_raw());
    }

    #[test]
    fn to_raw_named() {
        let init_arguments = InitArguments::default().with_version(JniVersion::V8);
        let test_name = "test-name";
        let arguments = AttachArguments::named(&init_arguments, test_name);
        let mut buffer: Vec<u8> = vec![];
        let raw_arguments = super::to_raw(&arguments, &mut buffer);
        assert_eq!(raw_arguments.raw_arguments.group, ptr::null_mut());
        assert_eq!(raw_arguments.raw_arguments.version, JniVersion::V8.to_raw());
        assert_eq!(
            from_java_string(unsafe {
                slice::from_raw_parts(
                    raw_arguments.raw_arguments.name as *const u8,
                    raw_arguments.buffer_len,
                )
            })
            .unwrap(),
            test_name
        );
    }
}
