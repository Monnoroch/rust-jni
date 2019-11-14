mod object;

/// An integration test for the `java::lang::Class` type.
#[cfg(test)]
mod class {
    use crate::object;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let class = java::lang::Class::find(&env, "java/lang/RuntimeException", &token).unwrap();

        object::test_object(
            &class,
            "java/lang/Class",
            "class java.lang.RuntimeException",
            &env,
            &token,
        );

        assert!(class
            .class(&token)
            .is_same_as(&java::lang::Class::get_class(&env, &token).unwrap(), &token));

        let parent_class = java::lang::Class::find(&env, "java/lang/Throwable", &token).unwrap();
        assert!(class.is_subtype_of(&parent_class, &token));
        assert!(!parent_class.is_subtype_of(&class, &token));
        assert!(class
            .parent(&token)
            .unwrap()
            .parent(&token)
            .unwrap()
            .is_same_as(&parent_class, &token));

        let exception = java::lang::Class::find(&env, "java/lang/Invalid", &token).unwrap_err();
        assert_eq!(
            exception.get_message(&token).unwrap().as_string(&token),
            "java/lang/Invalid"
        );
    }
}
