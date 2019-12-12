/// Test that calling methods with object arguments and results works as expected including when
/// aruments and resutls are subclasses.
#[cfg(test)]
mod test {
    use java::lang::Class;
    use rust_jni::*;
    use rust_jni_java_lib::*;
    use std::fs;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(&AttachArguments::new(init_arguments.version()), |token| {
            let classes = vec![
                "ClassWithObjectNativeMethods",
                "SimpleClass",
                "SimpleSubClass",
            ];
            for class_name in classes {
                Class::define(
                    &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                    &token,
                )
                .unwrap();
            }

            let test_object1 = SimpleClass::new(&token, 12).unwrap();
            let test_object2 = SimpleSubClass::new(&token, 12).unwrap();

            // Call object methods.

            let object = ClassWithObjectNativeMethods::new(&token).unwrap();

            assert!(object
                .test_function_object(&token, &test_object1)
                .or_npe(&token)
                .unwrap()
                .is_same_as(&token, &test_object1));

            assert!(object
                .test_function_object(&token, &test_object2)
                .or_npe(&token)
                .unwrap()
                .is_same_as(&token, &test_object2));

            // Call static methods.

            assert!(ClassWithObjectNativeMethods::test_static_function_object(
                &token,
                &test_object1
            )
            .or_npe(&token)
            .unwrap()
            .is_same_as(&token, &test_object1));

            assert!(ClassWithObjectNativeMethods::test_static_function_object(
                &token,
                &test_object2
            )
            .or_npe(&token)
            .unwrap()
            .is_same_as(&token, &test_object2));

            ((), token)
        })
        .unwrap();
    }
}
