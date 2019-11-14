/// Errors returned by JNI function.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#return-codes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JniError {
    /// Unknown error.
    /// Needed for forward compability.
    Unknown(i32),
    /// Returned when the currect thread is not attached to a Java VM.
    ThreadDetached,
    /// Returned when requesting a VM with an unsupported version.
    UnsupportedVersion,
    /// Returned when there isn't enough memory for the operation.
    NotEnoughMemory,
    /// Returned when trying to create a new Java VM when
    /// one already exists in the current process.
    /// Creating multiple Java VMs in a single process is not supported.Unknown
    /// See [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    /// for more details.
    VmExists,
    /// Returned when passing invalid arguments to JNI calls.
    InvalidArguments,
}

impl JniError {
    /// Convert from a raw `jint` error.
    pub(crate) fn from_raw(error: jni_sys::jint) -> Option<JniError> {
        match error {
            jni_sys::JNI_OK => None,
            jni_sys::JNI_EDETACHED => Some(JniError::ThreadDetached),
            jni_sys::JNI_EVERSION => Some(JniError::UnsupportedVersion),
            jni_sys::JNI_ENOMEM => Some(JniError::NotEnoughMemory),
            jni_sys::JNI_EEXIST => Some(JniError::VmExists),
            jni_sys::JNI_EINVAL => Some(JniError::InvalidArguments),
            error => Some(JniError::Unknown(error)),
        }
    }
}

#[cfg(test)]
mod from_raw_tests {
    use super::*;

    #[test]
    fn from_raw_ok() {
        assert_eq!(JniError::from_raw(jni_sys::JNI_OK), None);
    }

    #[test]
    fn from_raw_error() {
        assert_eq!(
            JniError::from_raw(jni_sys::JNI_EDETACHED),
            Some(JniError::ThreadDetached)
        );
        assert_eq!(
            JniError::from_raw(jni_sys::JNI_EVERSION),
            Some(JniError::UnsupportedVersion)
        );
        assert_eq!(
            JniError::from_raw(jni_sys::JNI_ENOMEM),
            Some(JniError::NotEnoughMemory)
        );
        assert_eq!(
            JniError::from_raw(jni_sys::JNI_EEXIST),
            Some(JniError::VmExists)
        );
        assert_eq!(
            JniError::from_raw(jni_sys::JNI_EINVAL),
            Some(JniError::InvalidArguments)
        );
    }

    #[test]
    fn from_raw_unknown_error() {
        assert_eq!(JniError::from_raw(7), Some(JniError::Unknown(7)));
    }
}
