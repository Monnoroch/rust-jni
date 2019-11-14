#[cfg(test)]
mod default_jvm_arguments {
    #[test]
    fn supported_versions() {
        use rust_jni::{InitArguments, JniVersion};
        assert_eq!(
            InitArguments::get_default(JniVersion::V2)
                .unwrap()
                .version(),
            JniVersion::V2
        );
        assert_eq!(
            InitArguments::get_default(JniVersion::V4)
                .unwrap()
                .version(),
            JniVersion::V4
        );
        assert_eq!(
            InitArguments::get_default(JniVersion::V6)
                .unwrap()
                .version(),
            JniVersion::V6
        );
        assert_eq!(
            InitArguments::get_default(JniVersion::V8)
                .unwrap()
                .version(),
            JniVersion::V8
        );
    }
}
