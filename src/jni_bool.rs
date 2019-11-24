use jni_sys;

pub(crate) fn to_rust(value: jni_sys::jboolean) -> bool {
    match value {
        jni_sys::JNI_TRUE => true,
        jni_sys::JNI_FALSE => false,
        value => panic!("Unexpected jboolean value {:?}", value),
    }
}

pub(crate) fn to_jni(value: bool) -> jni_sys::jboolean {
    match value {
        true => jni_sys::JNI_TRUE,
        false => jni_sys::JNI_FALSE,
    }
}

#[cfg(test)]
mod jni_bool_tests {
    use super::*;

    #[test]
    fn test_to_jni() {
        assert_eq!(to_jni(true), jni_sys::JNI_TRUE);
        assert_eq!(to_jni(false), jni_sys::JNI_FALSE);
    }

    #[test]
    fn test_to_rust() {
        assert_eq!(to_rust(jni_sys::JNI_TRUE), true);
        assert_eq!(to_rust(jni_sys::JNI_FALSE), false);
    }

    #[test]
    #[should_panic(expected = "Unexpected jboolean value 10")]
    fn test_to_rust_unknown() {
        to_rust(10);
    }
}
