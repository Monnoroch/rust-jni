use jni_sys;

pub(crate) fn to_rust(value: jni_sys::jboolean) -> bool {
    match value {
        jni_sys::JNI_TRUE => true,
        jni_sys::JNI_FALSE => false,
        value => panic!("Unexpected jboolean value {}", value),
    }
}

pub(crate) fn to_jni(value: bool) -> jni_sys::jboolean {
    match value {
        true => jni_sys::JNI_TRUE,
        false => jni_sys::JNI_FALSE,
    }
}
