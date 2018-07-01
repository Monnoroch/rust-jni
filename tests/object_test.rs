extern crate rust_jni;

mod object;

/// An integration test for the `java::lang::Object` type.
#[cfg(test)]
mod class {
    use object;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let class = java::lang::Class::find(&env, "java/lang/Object", &token).unwrap();
        let object: &java::lang::Object = class.cast();

        object::test_object(
            object,
            "java/lang/Class",
            "class java.lang.Object",
            &env,
            &token,
        );

        assert!(
            object
                .class(&token)
                .is_same_as(&java::lang::Class::get_class(&env, &token).unwrap(), &token)
        );
    }
}
