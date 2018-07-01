extern crate rust_jni;
extern crate rust_jni_java_lib;

#[cfg(test)]
mod call_methods {
    use rust_jni::{java, AttachArguments, InitArguments, JavaVM, JniVersion};
    use rust_jni_java_lib::rustjni::test::*;
    use std::fs;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();

        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        let classes = vec!["TestMethodsClass"];
        for class_name in classes {
            java::lang::Class::define(
                &env,
                &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                &token,
            ).unwrap();
        }

        let other = TestMethodsClass::init(&env, &token).unwrap();
        let value = TestMethodsClass::init(&env, &token).unwrap();

        // All methods should return their second argument.

        TestMethodsClass::test_static_function_void(&env, &token).unwrap();
        assert_eq!(
            TestMethodsClass::test_static_function_bool(&env, false, true, false, &token).unwrap(),
            true
        );
        assert_eq!(
            TestMethodsClass::test_static_function_bool(&env, true, false, true, &token).unwrap(),
            false
        );
        assert_eq!(
            TestMethodsClass::test_static_function_char(&env, 'a', 'h', 'b', &token).unwrap(),
            'h'
        );
        assert_eq!(
            TestMethodsClass::test_static_function_char(&env, 'ф', 'я', 'ю', &token).unwrap(),
            'я'
        );
        assert_eq!(
            TestMethodsClass::test_static_function_byte(&env, 7, 10, 9, &token).unwrap(),
            10
        );
        assert_eq!(
            TestMethodsClass::test_static_function_short(&env, 7, 10, 9, &token).unwrap(),
            10
        );
        assert_eq!(
            TestMethodsClass::test_static_function_int(&env, 7, 10, 9, &token).unwrap(),
            10
        );
        assert_eq!(
            TestMethodsClass::test_static_function_long(&env, 7, 10, 9, &token).unwrap(),
            10
        );
        assert_eq!(
            TestMethodsClass::test_static_function_float(&env, 7., 10., 9., &token).unwrap(),
            10.
        );
        assert_eq!(
            TestMethodsClass::test_static_function_double(&env, 7., 10., 9., &token).unwrap(),
            10.
        );
        assert_eq!(
            TestMethodsClass::test_static_function_class(&env, &other, &value, &other, &token)
                .unwrap(),
            value
        );

        value.test_function_void(&token).unwrap();
        assert_eq!(
            value
                .test_function_bool(false, true, false, &token)
                .unwrap(),
            true
        );
        assert_eq!(
            value.test_function_bool(true, false, true, &token).unwrap(),
            false
        );
        assert_eq!(
            value.test_function_char('a', 'h', 'b', &token).unwrap(),
            'h'
        );
        assert_eq!(
            value.test_function_char('ф', 'я', 'ю', &token).unwrap(),
            'я'
        );
        assert_eq!(value.test_function_byte(7, 10, 9, &token).unwrap(), 10);
        assert_eq!(value.test_function_short(7, 10, 9, &token).unwrap(), 10);
        assert_eq!(value.test_function_int(7, 10, 9, &token).unwrap(), 10);
        assert_eq!(value.test_function_long(7, 10, 9, &token).unwrap(), 10);
        assert_eq!(value.test_function_float(7., 10., 9., &token).unwrap(), 10.);
        assert_eq!(
            value.test_function_double(7., 10., 9., &token).unwrap(),
            10.
        );
        assert_eq!(
            value
                .test_function_class(&other, &value, &other, &token)
                .unwrap(),
            value
        );
    }
}
