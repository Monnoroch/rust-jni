mod object;

/// An integration test for the `java::lang::String` type.
#[cfg(all(test, feature = "libjvm"))]
mod string {
    use crate::object;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let string = java::lang::String::new(&env, "test-string", &token).unwrap();
        object::test_object(&string, "java/lang/String", "test-string", &env, &token);

        assert!(string.class(&token).is_same_as(
            &java::lang::String::get_class(&env, &token).unwrap(),
            &token
        ));

        assert_eq!(
            java::lang::String::empty(&env, &token)
                .unwrap()
                .as_string(&token),
            ""
        );

        assert_eq!(
            java::lang::String::new(&env, "", &token)
                .unwrap()
                .as_string(&token),
            ""
        );

        assert_eq!(
            java::lang::String::value_of_int(&env, 17, &token)
                .unwrap()
                .as_string(&token),
            "17"
        );

        let string = java::lang::String::new(&env, "строка", &token).unwrap();
        assert_eq!(string.as_string(&token), "строка");
        assert_eq!(string.len(&token), 6);
        assert_eq!(string.size(&token), 12);
    }
}
