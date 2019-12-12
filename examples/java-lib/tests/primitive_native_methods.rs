/// Test that calling methods with primitive arguments and results works as expected.
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
            let classes = vec!["ClassWithPrimitiveNativeMethods"];
            for class_name in classes {
                Class::define(
                    &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                    &token,
                )
                .unwrap();
            }

            // Call object methods.

            let object = ClassWithPrimitiveNativeMethods::new(&token).unwrap();

            object.test_function_void(&token).unwrap();
            assert_eq!(object.test_function_bool(&token, true).unwrap(), false);
            assert_eq!(object.test_function_char(&token, '0').unwrap(), '1');
            assert_eq!(object.test_function_u8(&token, 10).unwrap(), 12);
            assert_eq!(object.test_function_i16(&token, 10).unwrap(), 13);
            assert_eq!(object.test_function_i32(&token, 10).unwrap(), 14);
            assert_eq!(object.test_function_i64(&token, 10).unwrap(), 15);
            assert_eq!(object.test_function_f32(&token, 10.).unwrap(), 16.);
            assert_eq!(object.test_function_f64(&token, 10.).unwrap(), 17.);

            // Call static methods.

            ClassWithPrimitiveNativeMethods::test_static_function_void(&token).unwrap();
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_bool(&token, true).unwrap(),
                false
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_char(&token, '0').unwrap(),
                '1'
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_u8(&token, 10).unwrap(),
                12
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_i16(&token, 10).unwrap(),
                13
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_i32(&token, 10).unwrap(),
                14
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_i64(&token, 10).unwrap(),
                15
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_f32(&token, 10.).unwrap(),
                16.
            );
            assert_eq!(
                ClassWithPrimitiveNativeMethods::test_static_function_f64(&token, 10.).unwrap(),
                17.
            );

            ((), token)
        })
        .unwrap();
    }
}
