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
/// # #[cfg(feature = "libjvm")]
/// # fn main() {
/// use rust_jni::{AttachArguments, JniVersion};
///
/// let attach_arguments = AttachArguments::new(JniVersion::V8);
///
/// assert_eq!(attach_arguments.version(), JniVersion::V8);
/// # }
/// #
/// # #[cfg(not(feature = "libjvm"))]
/// # fn main() {}
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct AttachArguments {
    version: JniVersion,
    thread_name: Option<String>,
    // TODO(#7): support thread groups.
}

impl AttachArguments {
    /// Create attach arguments with the default thread name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn new(version: JniVersion) -> Self {
        AttachArguments {
            thread_name: None,
            version: version,
        }
    }

    /// Create attach arguments with a specified thread name.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn named(version: JniVersion, thread_name: impl Into<String>) -> Self {
        AttachArguments {
            thread_name: Some(thread_name.into()),
            version: version,
        }
    }

    /// Return the JNI version to request when attaching a thread to a Java VM.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn version(&self) -> JniVersion {
        self.version
    }

    /// Return the JNI thread name to request when attaching a thread to a Java VM.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#attachcurrentthread)
    pub fn thread_name(&self) -> &Option<String> {
        &self.thread_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert_eq!(
            AttachArguments::new(JniVersion::V4),
            AttachArguments {
                thread_name: None,
                version: JniVersion::V4
            }
        );
    }

    #[test]
    fn named() {
        assert_eq!(
            AttachArguments::named(JniVersion::V4, "test-name"),
            AttachArguments {
                thread_name: Some("test-name".into()),
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

    #[test]
    fn thread_name() {
        let arguments = AttachArguments {
            version: JniVersion::V4,
            thread_name: Some("test-name".into()),
        };
        assert_eq!(arguments.thread_name(), &Some("test-name".to_owned()));
    }

    #[test]
    fn no_thread_name() {
        let arguments = AttachArguments {
            version: JniVersion::V4,
            thread_name: None,
        };
        assert_eq!(arguments.thread_name(), &None);
    }
}

/// A wrapper around `jni_sys::JavaVMAttachArgs` with a lifetime to ensure
/// there's no access to freed memory.
pub(crate) struct RawAttachArguments<'a> {
    pub raw_arguments: jni_sys::JavaVMAttachArgs,
    // Used by JNI.
    #[allow(dead_code)]
    buffer_len: usize,
    _buffer: PhantomData<&'a Vec<u8>>,
}

impl AttachArguments {
    /// Convert `AttachArguments` to `jni_sys::JavaVMAttachArgs`. Uses a buffer for storing
    /// the Java string with the thread name.
    pub(crate) fn to_raw<'a>(&self, buffer: &'a mut Vec<u8>) -> RawAttachArguments<'a> {
        let version = self.version().to_raw();
        let group = ptr::null_mut();
        let raw_arguments = jni_sys::JavaVMAttachArgs {
            name: match self.thread_name() {
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
}

#[cfg(test)]
mod to_raw_tests {
    use super::*;
    use std::slice;

    #[test]
    fn to_raw() {
        let arguments = AttachArguments::new(JniVersion::V8);
        let mut buffer: Vec<u8> = vec![];
        let raw_arguments = arguments.to_raw(&mut buffer);
        assert_eq!(raw_arguments.raw_arguments.group, ptr::null_mut());
        assert_eq!(raw_arguments.raw_arguments.name, ptr::null_mut());
        assert_eq!(raw_arguments.raw_arguments.version, JniVersion::V8.to_raw());
    }

    #[test]
    fn to_raw_named() {
        let test_name = "test-name";
        let arguments = AttachArguments::named(JniVersion::V8, test_name);
        let mut buffer: Vec<u8> = vec![];
        let raw_arguments = arguments.to_raw(&mut buffer);
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
