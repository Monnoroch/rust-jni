use jni_sys;

/// JDK/JRE version enum.
///
/// Maps to the `jni_sys::JNI_VERSION_1_*` constants.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#version-constants)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JniVersion {
    /// JDK/JRE 1.1.
    V1,
    /// JDK/JRE 1.2.
    V2,
    /// JDK/JRE 1.4.
    V4,
    /// JDK/JRE 1.6.
    V6,
    /// JDK/JRE 1.8.
    V8,
    /// JDK/JRE 9.
    V9,
    /// JDK/JRE 10.
    V10,
    /// Unknown version.
    /// Needed for forward compability and to request a version that has not been added yet.
    Unknown(i32),
}

// TODO(monnoroch): contribute these to `jni_sys` crate.
const JNI_VERSION_9: jni_sys::jint = 0x00090000;
const JNI_VERSION_10: jni_sys::jint = 0x000a0000;

impl JniVersion {
    /// Convert from a raw `jint` version.
    pub(crate) fn from_raw(version: jni_sys::jint) -> JniVersion {
        match version {
            jni_sys::JNI_VERSION_1_1 => JniVersion::V1,
            jni_sys::JNI_VERSION_1_2 => JniVersion::V2,
            jni_sys::JNI_VERSION_1_4 => JniVersion::V4,
            jni_sys::JNI_VERSION_1_6 => JniVersion::V6,
            jni_sys::JNI_VERSION_1_8 => JniVersion::V8,
            JNI_VERSION_9 => JniVersion::V9,
            JNI_VERSION_10 => JniVersion::V10,
            _ => JniVersion::Unknown(version),
        }
    }
}

#[cfg(test)]
mod from_raw_tests {
    use super::*;

    #[test]
    fn from_raw_version() {
        assert_eq!(
            JniVersion::from_raw(jni_sys::JNI_VERSION_1_1),
            JniVersion::V1
        );
        assert_eq!(
            JniVersion::from_raw(jni_sys::JNI_VERSION_1_2),
            JniVersion::V2
        );
        assert_eq!(
            JniVersion::from_raw(jni_sys::JNI_VERSION_1_4),
            JniVersion::V4
        );
        assert_eq!(
            JniVersion::from_raw(jni_sys::JNI_VERSION_1_6),
            JniVersion::V6
        );
        assert_eq!(
            JniVersion::from_raw(jni_sys::JNI_VERSION_1_8),
            JniVersion::V8
        );
        assert_eq!(JniVersion::from_raw(JNI_VERSION_9), JniVersion::V9);
        assert_eq!(JniVersion::from_raw(JNI_VERSION_10), JniVersion::V10);
    }

    #[test]
    fn from_unknown_raw_version() {
        assert_eq!(JniVersion::from_raw(7), JniVersion::Unknown(7));
    }
}

impl JniVersion {
    /// Convert to a raw `jint` version.
    pub(crate) fn to_raw(self) -> jni_sys::jint {
        match self {
            JniVersion::V1 => jni_sys::JNI_VERSION_1_1,
            JniVersion::V2 => jni_sys::JNI_VERSION_1_2,
            JniVersion::V4 => jni_sys::JNI_VERSION_1_4,
            JniVersion::V6 => jni_sys::JNI_VERSION_1_6,
            JniVersion::V8 => jni_sys::JNI_VERSION_1_8,
            JniVersion::V9 => JNI_VERSION_9,
            JniVersion::V10 => JNI_VERSION_10,
            JniVersion::Unknown(version) => version,
        }
    }
}

#[cfg(test)]
mod to_raw_tests {
    use super::*;

    #[test]
    fn to_raw_version() {
        assert_eq!(JniVersion::V1.to_raw(), jni_sys::JNI_VERSION_1_1);
        assert_eq!(JniVersion::V2.to_raw(), jni_sys::JNI_VERSION_1_2);
        assert_eq!(JniVersion::V4.to_raw(), jni_sys::JNI_VERSION_1_4);
        assert_eq!(JniVersion::V6.to_raw(), jni_sys::JNI_VERSION_1_6);
        assert_eq!(JniVersion::V8.to_raw(), jni_sys::JNI_VERSION_1_8);
        assert_eq!(JniVersion::V9.to_raw(), JNI_VERSION_9);
        assert_eq!(JniVersion::V10.to_raw(), JNI_VERSION_10);
    }

    #[test]
    fn to_unknown_raw_version() {
        assert_eq!(JniVersion::Unknown(7).to_raw(), 7);
    }
}
