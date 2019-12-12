/// Test that overriden methods are called correctly, including calling methods
/// on objects casted to base classes.
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
                "SimpleClass",
                "SubClassWithMethodOverride",
                "SubSubClassWithMethodOverride",
            ];
            for class_name in classes {
                Class::define(
                    &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                    &token,
                )
                .unwrap();
            }

            // Can call own methods.

            let object = SimpleClass::new(&token, 12).unwrap();
            assert_eq!(object.value_with_added(&token, 5).unwrap(), 12 + 5);

            // Can call super methods.

            let object = SubClassWithMethodOverride::new(&token, 12).unwrap();
            assert_eq!(
                object.value_with_added(&token, 5).unwrap(),
                (12 + 1) + 5 * 2
            );

            let object = SubClassWithMethodOverride::new(&token, 12).unwrap();
            assert_eq!(
                (*object).value_with_added(&token, 5).unwrap(),
                (12 + 1) + 5 * 2
            );

            // Can call super-super methods.

            let object = SubSubClassWithMethodOverride::new(&token, 12).unwrap();
            assert_eq!(
                object.value_with_added(&token, 5).unwrap(),
                (12 + 2) + 5 * 3
            );

            let object = SubSubClassWithMethodOverride::new(&token, 12).unwrap();
            assert_eq!(
                (*object).value_with_added(&token, 5).unwrap(),
                (12 + 2) + 5 * 3
            );

            let object = SubSubClassWithMethodOverride::new(&token, 12).unwrap();
            assert_eq!(
                (**object).value_with_added(&token, 5).unwrap(),
                (12 + 2) + 5 * 3
            );

            ((), token)
        })
        .unwrap();
    }
}
