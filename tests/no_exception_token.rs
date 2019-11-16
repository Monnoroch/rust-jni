#[cfg(all(test, feature = "libjvm"))]
mod create_envs {
    use rust_jni::*;

    fn example_with_attached(vm: &JavaVM, init_arguments: &InitArguments) {
        let _ = vm.with_attached(
            &AttachArguments::new(init_arguments.version()),
            |_env: &JniEnv, token: NoException| ((), token),
        );
    }

    fn example_empty_string_length(vm: &JavaVM, init_arguments: &InitArguments) {
        let empty_string_length = vm
            .with_attached(
                &AttachArguments::new(init_arguments.version()),
                |env, token| {
                    let string = java::lang::String::empty(env, &token).unwrap();
                    (string.len(&token), token)
                },
            )
            .unwrap();
        assert_eq!(empty_string_length, 0);
    }

    fn example_attach_manually(vm: &JavaVM, init_arguments: &InitArguments) {
        let env = vm
            .attach(&AttachArguments::new(init_arguments.version()))
            .unwrap();
        let _token = env.token();
    }

    fn example_throws_exception(vm: &JavaVM, init_arguments: &InitArguments) {
        let _ = vm
            .with_attached(
                &AttachArguments::new(init_arguments.version()),
                |env, token| {
                    let _string = java::lang::Class::find(env, "java/lang/String", &token).unwrap();
                    let _exception = java::lang::Class::find(env, "invalid", &token).unwrap_err();
                    ((), token)
                },
            )
            .unwrap();
    }

    fn example_rethrows_exception(vm: &JavaVM, init_arguments: &InitArguments) {
        let _ = vm
            .with_attached(
                &AttachArguments::new(init_arguments.version()),
                |env, token| {
                    let exception = java::lang::Class::find(env, "invalid", &token).unwrap_err();
                    let exception_token = exception.throw(token);
                    let (_exception, token) = exception_token.unwrap();
                    let _ = java::lang::String::empty(env, &token); // can call Java methods again.
                    ((), token)
                },
            )
            .unwrap();
    }

    #[test]
    fn test() {
        let init_arguments = InitArguments::default();
        let vm = JavaVM::create(&init_arguments).unwrap();
        example_with_attached(&vm, &init_arguments);
        example_empty_string_length(&vm, &init_arguments);
        example_attach_manually(&vm, &init_arguments);
        example_throws_exception(&vm, &init_arguments);
        example_rethrows_exception(&vm, &init_arguments);
    }
}
