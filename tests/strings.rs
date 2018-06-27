extern crate rust_jni;

#[cfg(test)]
mod strings {
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        assert_eq!(
            java::lang::String::empty(&env, &token)
                .unwrap()
                .as_string(&token),
            ""
        );
        assert_eq!(
            java::lang::String::new(&env, "test-string", &token)
                .unwrap()
                .as_string(&token),
            "test-string"
        );
    }
}
