/// An integration test for the `java::lang::String` type.
#[cfg(all(test, feature = "libjvm"))]
mod string {
    use rust_jni::java::lang::*;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(
            &AttachArguments::new(init_arguments.version()),
            |env, token| {
                let string = String::empty(env, &token).unwrap();

                assert!(string
                    .class(&token)
                    .is_same_as(&token, &String::class(env, &token).unwrap(),));

                assert_eq!(string.len(&token), 0);
                assert_eq!(string.size(&token), 0);
                assert_eq!(string.as_string(&token), "");

                assert_eq!(
                    java::lang::String::new(&env, &token, "")
                        .unwrap()
                        .as_string(&token),
                    ""
                );

                let string = String::new(&env, &token, "строка").unwrap();
                assert_eq!(string.as_string(&token), "строка");
                assert_eq!(string.len(&token), 6);
                assert_eq!(string.size(&token), 12);

                assert_eq!(
                    String::value_of_int(&env, &token, 17)
                        .unwrap()
                        .unwrap()
                        .as_string(&token),
                    "17"
                );

                ((), token)
            },
        )
        .unwrap();
    }
}
