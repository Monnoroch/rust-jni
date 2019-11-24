/// An integration test for the `java::lang::Throwable` type.
#[cfg(all(test, feature = "libjvm"))]
mod throwable {
    use rust_jni::java::lang::*;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(
            &AttachArguments::new(init_arguments.version()),
            |env, token| {
                let _throwable = Throwable::new(env, &token).unwrap();

                let throwable = Throwable::new_with_message(
                    env,
                    &token,
                    &String::new(env, &token, "cause").unwrap(),
                )
                .unwrap();

                let _ = Throwable::new_with_cause(env, &token, &throwable).unwrap();

                let throwable = Throwable::new_with_message_and_cause(
                    env,
                    &token,
                    &String::new(env, &token, "message").unwrap(),
                    &throwable,
                )
                .unwrap();

                assert!(throwable
                    .class(&token)
                    .is_same_as(&token, &Throwable::class(env, &token).unwrap()));

                assert_eq!(
                    throwable
                        .get_message(&token)
                        .unwrap()
                        .unwrap()
                        .as_string(&token),
                    "message"
                );

                assert_eq!(
                    throwable
                        .get_cause(&token)
                        .unwrap()
                        .unwrap()
                        .get_message(&token)
                        .unwrap()
                        .unwrap()
                        .as_string(&token),
                    "cause"
                );

                let token = throwable.throw(token);
                let (throwable, token) = token.unwrap();

                assert_eq!(
                    throwable
                        .get_message(&token)
                        .unwrap()
                        .unwrap()
                        .as_string(&token),
                    "message"
                );

                ((), token)
            },
        )
        .unwrap();
    }
}
