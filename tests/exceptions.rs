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

        let string = java::lang::String::new(&env, "test-string", &token).unwrap();
        let exception = java::lang::Throwable::new(&env, string, &token).unwrap();
        let token = exception.throw(token);
        let (exception, token) = token.unwrap();
        assert_eq!(
            exception.to_string(&token).unwrap().as_string(&token),
            "java.lang.Throwable: test-string"
        );
    }
}
