/// Test that a base class method that accepts a class as an argument can accept any of:
/// - Value
/// - &Value
/// - Option<Value>
/// - Option<&Value>
///
/// Also test that it's the same when the method is called on any subclass.
#[cfg(test)]
mod test {
    use java::lang::Class;
    use rust_jni::*;
    use rust_jni_java_lib::*;
    use std::fs;

    macro_rules! assert_value_with_added_eq {
        ($env:expr, $token:expr, $object:expr, $argument:expr, $value:expr, $expected:expr) => {{
            let new_object = $object
                .combine($token, $argument)
                .or_npe($env, $token)
                .unwrap();
            assert_eq!(
                new_object.value_with_added($token, $value).unwrap(),
                $expected
            );
        }};
    }

    macro_rules! assert_value_with_added_eq_pass_by_all {
        ($env:expr, $token:expr, $object:expr, $argument:expr, $value:expr, $expected:expr) => {
            assert_value_with_added_eq!(
                $env,
                $token,
                $object,
                $argument.unwrap(),
                $value,
                $expected
            );

            assert_value_with_added_eq!(
                $env,
                $token,
                $object,
                &$argument.unwrap(),
                $value,
                $expected
            );

            assert_value_with_added_eq!(
                $env,
                $token,
                $object,
                Some($argument.unwrap()),
                $value,
                $expected
            );

            assert_value_with_added_eq!(
                $env,
                $token,
                $object,
                Some(&$argument.unwrap()),
                $value,
                $expected
            );
        };
    }

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(
            &AttachArguments::new(init_arguments.version()),
            |env, token| {
                let classes = vec!["SimpleClass", "SimpleSubClass", "SimpleSubSubClass"];
                for class_name in classes {
                    Class::define(
                        env,
                        &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                        &token,
                    )
                    .unwrap();
                }

                // Can call a method on it's own class.

                let object = SimpleClass::new(env, &token, 12).unwrap();

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleClass::new(env, &token, 7),
                    5,
                    12 + 7 + 5
                );

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleSubClass::new(env, &token, 7),
                    5,
                    12 + (7 + 1) + 5
                );

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleSubSubClass::new(env, &token, 7),
                    5,
                    12 + (7 + 2) + 5
                );

                // Can call a method on a subclass.

                let object = SimpleSubClass::new(env, &token, 12).unwrap();

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleClass::new(env, &token, 7),
                    5,
                    (12 + 1) + 7 + 5
                );

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleSubClass::new(env, &token, 7),
                    5,
                    (12 + 1) + (7 + 1) + 5
                );

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleSubSubClass::new(env, &token, 7),
                    5,
                    (12 + 1) + (7 + 2) + 5
                );

                // Can call a method on a sub-subclass.

                let object = SimpleSubSubClass::new(env, &token, 12).unwrap();

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleClass::new(env, &token, 7),
                    5,
                    (12 + 2) + 7 + 5
                );

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleSubClass::new(env, &token, 7),
                    5,
                    (12 + 2) + (7 + 1) + 5
                );

                assert_value_with_added_eq_pass_by_all!(
                    env,
                    &token,
                    object,
                    SimpleSubSubClass::new(env, &token, 7),
                    5,
                    (12 + 2) + (7 + 2) + 5
                );

                ((), token)
            },
        )
        .unwrap();
    }
}
