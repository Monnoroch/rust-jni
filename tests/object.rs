/// An integration test for the `java::lang::Object` type.
#[cfg(all(test, feature = "libjvm"))]
mod class {
    use rust_jni::java::lang::*;
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(
            &AttachArguments::new(init_arguments.version()),
            |env, token| {
                let object = Object::new(env, &token).unwrap();

                assert!(object.class(&token).is_same_as(
                    &token,
                    &Class::find(env, &token, "java/lang/Object").unwrap(),
                ));

                assert!(object.is_same_as(&token, &object));
                assert!(object.is_instance_of(
                    &token,
                    &Class::find(env, &token, "java/lang/Object").unwrap()
                ));

                assert!(object
                    .clone_object(&token)
                    .unwrap()
                    .is_same_as(&token, &object));
                assert!(object
                    .clone_object(&token)
                    .unwrap()
                    .equals(&token, &object)
                    .unwrap());

                assert_eq!(object.clone(), object);

                let string1 = String::new(env, &token, "test").unwrap();
                let string2 = string1.clone_object(&token).unwrap();
                let string3 = String::new(env, &token, "test").unwrap();

                assert!(string1.is_same_as(&token, &string1));
                assert!(string1.is_same_as(&token, &string2));
                assert!(!string1.is_same_as(&token, &string3));

                assert_eq!(string1, string1);
                assert_eq!(string1, string2);
                assert_ne!(string1, string3);

                assert!(string1.equals(&token, &string1).unwrap());
                assert!(string1.equals(&token, &string2).unwrap());
                assert!(string1.equals(&token, &string3).unwrap());

                assert_eq!(
                    object.to_string(&token).unwrap().unwrap().as_string(&token),
                    format!("java.lang.Object@{:x}", object.hash_code(&token).unwrap())
                );

                assert!(format!("{:?}", object).contains("java.lang.Object@"));

                ((), token)
            },
        )
        .unwrap();
    }
}
