use jni_sys;

/// JNI Version enum.
///
/// Maps to the `jni_sys::JNI_VERSION_1_*` constants.
///
/// [JNI documentation](https://docs.oracle.com/javase/9/docs/specs/jni/functions.html#version-constants)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JniVersion {
    /// JNI 1.1.
    V1,
    /// JNI 1.2.
    V2,
    /// JNI 1.4.
    V4,
    /// JNI 1.6.
    V6,
    /// JNI 1.8.
    V8,
    /// Unknown version.
    /// Needed for forward compability and to request a version that has not been added yet.
    Unknown(i32),
}

/// Convert from a raw `jint` version.
pub fn from_raw(version: jni_sys::jint) -> JniVersion {
    match version {
        jni_sys::JNI_VERSION_1_1 => JniVersion::V1,
        jni_sys::JNI_VERSION_1_2 => JniVersion::V2,
        jni_sys::JNI_VERSION_1_4 => JniVersion::V4,
        jni_sys::JNI_VERSION_1_6 => JniVersion::V6,
        jni_sys::JNI_VERSION_1_8 => JniVersion::V8,
        _ => JniVersion::Unknown(version),
    }
}

#[cfg(test)]
mod from_raw_tests {
    use super::*;

    #[test]
    fn from_raw_version() {
        assert_eq!(from_raw(jni_sys::JNI_VERSION_1_1), JniVersion::V1);
        assert_eq!(from_raw(jni_sys::JNI_VERSION_1_2), JniVersion::V2);
        assert_eq!(from_raw(jni_sys::JNI_VERSION_1_4), JniVersion::V4);
        assert_eq!(from_raw(jni_sys::JNI_VERSION_1_6), JniVersion::V6);
        assert_eq!(from_raw(jni_sys::JNI_VERSION_1_8), JniVersion::V8);
    }

    #[test]
    fn from_unknown_raw_version() {
        assert_eq!(from_raw(7), JniVersion::Unknown(7));
    }
}

/// Convert to a raw `jint` version.
pub fn to_raw(version: JniVersion) -> jni_sys::jint {
    match version {
        JniVersion::V1 => jni_sys::JNI_VERSION_1_1,
        JniVersion::V2 => jni_sys::JNI_VERSION_1_2,
        JniVersion::V4 => jni_sys::JNI_VERSION_1_4,
        JniVersion::V6 => jni_sys::JNI_VERSION_1_6,
        JniVersion::V8 => jni_sys::JNI_VERSION_1_8,
        JniVersion::Unknown(version) => version,
    }
}

#[cfg(test)]
mod to_raw_tests {
    use super::*;

    #[test]
    fn to_raw_version() {
        assert_eq!(to_raw(JniVersion::V1), jni_sys::JNI_VERSION_1_1);
        assert_eq!(to_raw(JniVersion::V2), jni_sys::JNI_VERSION_1_2);
        assert_eq!(to_raw(JniVersion::V4), jni_sys::JNI_VERSION_1_4);
        assert_eq!(to_raw(JniVersion::V6), jni_sys::JNI_VERSION_1_6);
        assert_eq!(to_raw(JniVersion::V8), jni_sys::JNI_VERSION_1_8);
    }

    #[test]
    fn to_unknown_raw_version() {
        assert_eq!(to_raw(JniVersion::Unknown(7)), 7);
    }
}
