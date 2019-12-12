/// An integration test for the `java::lang::Class` type.
#[cfg(all(test, feature = "libjvm"))]
mod class {
    use rust_jni::java::lang::*;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(&AttachArguments::new(init_arguments.version()), |token| {
            let class = Class::find(&token, "java/lang/RuntimeException").unwrap();

            assert!(class
                .class(&token)
                .is_same_as(&token, &Class::class(&token).unwrap(),));

            let parent_class = Throwable::class(&token).unwrap();

            assert!(class.is_subtype_of(&token, &parent_class));
            assert!(!parent_class.is_subtype_of(&token, &class));

            assert!(class
                .parent(&token)
                .unwrap()
                .parent(&token)
                .unwrap()
                .is_same_as(&token, &parent_class));

            let exception = Class::find(&token, "java/lang/Invalid").unwrap_err();
            assert_eq!(
                exception
                    .get_message(&token)
                    .or_npe(&token)
                    .unwrap()
                    .as_string(&token),
                "java/lang/Invalid"
            );

            ((), token)
        })
        .unwrap();
    }
}
