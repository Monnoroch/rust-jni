#[cfg(test)]
mod default_jvm_arguments {
    #[test]
    fn supported_versions() {
        use rust_jni::{InitArguments, JniVersion};
        assert_eq!(
            InitArguments::get_default_or_closest_supported(JniVersion::V1).version(),
            JniVersion::V2
        );
        assert_eq!(
            InitArguments::get_default_or_closest_supported(JniVersion::V2).version(),
            JniVersion::V2
        );
        assert_eq!(
            InitArguments::get_default_or_closest_supported(JniVersion::V4).version(),
            JniVersion::V4
        );
        assert_eq!(
            InitArguments::get_default_or_closest_supported(JniVersion::V6).version(),
            JniVersion::V6
        );
        assert_eq!(
            InitArguments::get_default_or_closest_supported(JniVersion::V8).version(),
            JniVersion::V8
        );
    }

    #[test]
    fn latest_version() {
        use rust_jni::{InitArguments, JniVersion};
        assert_eq!(
            InitArguments::get_latest_default().version(),
            JniVersion::V8
        );
    }
}
