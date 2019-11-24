extern crate java;
extern crate rust_jni;
extern crate rust_jni_java_lib;

#[cfg(test)]
mod test_inheritance {
    use java;
    use rust_jni::{AttachArguments, InitArguments, JavaVM, JniVersion, NoException};
    use rust_jni_java_lib::rustjni::test::*;
    use std::fs;

    fn test_object_function<'env>(
        value: &java::lang::Object<'env>,
        token: &NoException<'env>,
    ) -> bool {
        value.equals(value, token).unwrap()
    }

    fn test_interface_function<'env>(
        value: &(impl TestInterface<'env> + ?Sized),
        argument: i32,
        token: &NoException<'env>,
    ) -> i64 {
        value.test_interface_function(argument, token).unwrap()
    }

    fn test_interface_extended_function<'env>(
        value: &(impl TestInterfaceExtended<'env> + ?Sized),
        argument: i32,
        token: &NoException<'env>,
    ) -> i64 {
        value
            .test_interface_extended_function(argument, token)
            .unwrap()
    }

    fn test_class_function<'env>(
        value: &TestClass<'env>,
        argument: i32,
        token: &NoException<'env>,
    ) -> i64 {
        value.test_class_function(argument, token).unwrap()
    }

    fn test_class_extended_function<'env>(
        value: &TestClassExtended<'env>,
        argument: i32,
        token: &NoException<'env>,
    ) -> i64 {
        value.test_class_extended_function(argument, token).unwrap()
    }

    fn test_class_extended_final_function<'env>(
        value: &TestClassExtendedFinal<'env>,
        argument: i32,
        token: &NoException<'env>,
    ) -> i64 {
        value
            .test_class_extended_final_function(argument, token)
            .unwrap()
    }

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();

        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let classes = vec![
            "TestInterface",
            "TestInterfaceExtended",
            "TestClass",
            "TestClassExtended",
            "TestClassExtendedFinal",
        ];
        for class_name in classes {
            java::lang::Class::define(
                &env,
                &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                &token,
            )
            .unwrap();
        }

        let value = TestClass::create(&env, &token).unwrap();
        assert!(test_object_function(&value, &token));
        assert_eq!(test_class_function(&value, 42, &token), 42);
        assert_eq!(test_interface_function(&value, 42, &token), 42);
        assert_eq!(
            test_interface_function(&value as &TestInterface, 42, &token),
            42
        );

        let value = TestClassExtended::create(&env, &token).unwrap();
        assert!(test_object_function(&value, &token));
        assert_eq!(test_class_function(&value, 42, &token), 42);
        assert_eq!(test_class_extended_function(&value, 42, &token), 42);
        assert_eq!(test_interface_function(&value, 42, &token), 42);
        assert_eq!(
            test_interface_function(&value as &TestInterface, 42, &token),
            42
        );
        assert_eq!(test_interface_extended_function(&value, 42, &token), 42);
        assert_eq!(
            test_interface_extended_function(&value as &TestInterfaceExtended, 42, &token),
            42
        );

        let value = TestClassExtendedFinal::init(&env, &token).unwrap();
        assert!(test_object_function(&value, &token));
        assert_eq!(test_class_function(&value, 42, &token), 42);
        assert_eq!(test_class_extended_function(&value, 42, &token), 42);
        assert_eq!(test_class_extended_final_function(&value, 42, &token), 42);
        assert_eq!(test_interface_function(&value, 42, &token), 42);
        assert_eq!(
            test_interface_function(&value as &TestInterface, 42, &token),
            42
        );
        assert_eq!(test_interface_extended_function(&value, 42, &token), 42);
        assert_eq!(
            test_interface_extended_function(&value as &TestInterfaceExtended, 42, &token),
            42
        );
    }
}
