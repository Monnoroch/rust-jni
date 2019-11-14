mod object;

/// An integration test for the `java::lang::Throwable` type.
#[cfg(test)]
mod throwable {
    use crate::object;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let throwable = java::lang::Throwable::new(
            &env,
            &java::lang::String::new(&env, "test-string", &token).unwrap(),
            &token,
        )
        .unwrap();

        object::test_object(
            &throwable,
            "java/lang/Throwable",
            "java.lang.Throwable: test-string",
            &env,
            &token,
        );
        assert!(throwable.class(&token).is_same_as(
            &java::lang::Throwable::get_class(&env, &token).unwrap(),
            &token
        ));

        assert_eq!(
            throwable.get_message(&token).unwrap().as_string(&token),
            "test-string"
        );

        let token = throwable.throw(token);
        let (exception, token) = token.unwrap();
        assert_eq!(
            exception.to_string(&token).unwrap().as_string(&token),
            "java.lang.Throwable: test-string"
        );
    }
}
