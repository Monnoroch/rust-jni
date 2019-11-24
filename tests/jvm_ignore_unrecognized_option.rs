#[cfg(all(test, feature = "libjvm"))]
mod create_jvm {
    use rust_jni::*;
    use std::ptr;

    #[test]
    fn test() {
        let vm = JavaVM::create(
            &InitArguments::get_default(JniVersion::V8)
                .unwrap()
                .with_option(JvmOption::Unknown("utest".to_owned()))
                .ignore_unrecognized_options(),
        )
        .unwrap();
        unsafe { assert_ne!(vm.raw_jvm().as_ptr(), ptr::null_mut()) };
    }
}
