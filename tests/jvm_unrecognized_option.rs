#[cfg(test)]
mod create_jvm {
    use jni_sys;
    use rust_jni::*;

    #[test]
    fn unrecognized_option() {
        assert_eq!(
            JavaVM::create(
                &InitArguments::get_default(JniVersion::V8)
                    .unwrap()
                    .with_option(JvmOption::Unknown("utest".to_owned()))
                    .fail_on_unrecognized_options()
            )
            .unwrap_err(),
            JniError::Unknown(jni_sys::JNI_ERR)
        );
    }
}
