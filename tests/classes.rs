extern crate rust_jni;

#[cfg(test)]
mod classes {
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let string_class = java::lang::Class::find(&env, "java/lang/String", &token).unwrap();
        assert!(
            string_class
                .clone(&token)
                .unwrap()
                .is_same_as(&string_class, &token)
        );
        assert_eq!(string_class.clone(&token).unwrap(), string_class);

        let string_class_from_string = java::lang::String::new(&env, "test-string", &token)
            .unwrap()
            .class(&token);
        assert!(string_class.is_same_as(&string_class_from_string, &token));

        let class_class = java::lang::Class::find(&env, "java/lang/Class", &token).unwrap();
        assert!(string_class.is_instance_of(&class_class, &token));
        assert_eq!(
            string_class.to_string(&token).unwrap().as_string(&token),
            "class java.lang.String"
        );
        assert_eq!(ToString::to_string(&string_class), "class java.lang.String");
        assert!(format!("{:?}", string_class).contains("class java.lang.String"));

        let throwable_class = java::lang::Class::find(&env, "java/lang/Throwable", &token).unwrap();
        let exception_class =
            java::lang::Class::find(&env, "java/lang/RuntimeException", &token).unwrap();
        assert!(exception_class.is_subtype_of(&throwable_class, &token));
        assert!(!throwable_class.is_subtype_of(&exception_class, &token));
        assert!(
            exception_class
                .parent(&token)
                .unwrap()
                .parent(&token)
                .unwrap()
                .is_same_as(&throwable_class, &token)
        );

        // TODO: check the message.
        let _exception = java::lang::Class::find(&env, "java/lang/Invalid", &token).unwrap_err();
    }
}
