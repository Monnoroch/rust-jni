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
        let string_class_from_string = java::lang::String::new(&env, "test-string", &token)
            .unwrap()
            .class(&token);
        assert!(string_class.is_same_as(&string_class_from_string, &token));

        let class_class = java::lang::Class::find(&env, "java/lang/Class", &token).unwrap();
        assert!(string_class.is_instance_of(&class_class, &token));

        // TODO: check the message.
        let _exception = java::lang::Class::find(&env, "java/lang/Invalid", &token).unwrap_err();
    }
}
