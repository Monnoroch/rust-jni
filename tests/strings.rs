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

        let string = java::lang::String::new(&env, "test-string", &token).unwrap();
        assert_eq!(string.as_string(&token), "test-string");

        assert!(string.clone(&token).unwrap().is_same_as(&string, &token));
        assert!(string.class(&token).is_same_as(
            &java::lang::Class::find(&env, "java/lang/String", &token).unwrap(),
            &token
        ));
        assert_eq!(
            string.class(&token),
            java::lang::Class::find(&env, "java/lang/String", &token).unwrap()
        );
        assert_eq!(
            string.to_string(&token).unwrap().as_string(&token),
            "test-string"
        );

        assert_eq!(
            java::lang::String::value_of_int(&env, 17, &token)
                .unwrap()
                .as_string(&token),
            "17"
        );

        assert_eq!(ToString::to_string(&string), "test-string");
        assert!(format!("{:?}", string).contains("string: test-string"));
    }
}
