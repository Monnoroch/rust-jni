/// Test calling aliased methods: methods with the same name as in the base class, but not
/// overrides.
///
/// In Rust a caller must explicitly dereference an object to call the superclass method.
/// To call the method on the super-super class a caller must explicitly dereference the object twice, etc.
#[cfg(test)]
mod test {
    use java::lang::Class;
    use rust_jni::*;
    use rust_jni_java_lib::*;
    use std::fs;

    macro_rules! assert_value_with_added_eq {
        ($token:expr, $object:expr, $argument:expr, $value:expr, $expected:expr) => {{
            let new_object = $object
                .combine($token, $argument.unwrap())
                .or_npe($token)
                .unwrap();
            assert_eq!(
                new_object.value_with_added($token, $value).unwrap(),
                $expected
            );
        }};
    }

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        vm.with_attached(&AttachArguments::new(init_arguments.version()), |token| {
            let classes = vec![
                "SimpleClass",
                "SubClassWithMethodAlias",
                "SubSubClassWithMethodAlias",
            ];
            for class_name in classes {
                Class::define(
                    &fs::read(format!("./java/rustjni/test/{}.class", class_name)).unwrap(),
                    &token,
                )
                .unwrap();
            }

            // Base class -- no aliasing.

            let object = SimpleClass::new(&token, 12).unwrap();

            assert_value_with_added_eq!(&token, object, SimpleClass::new(&token, 7), 5, 12 + 7 + 5);

            assert_value_with_added_eq!(
                &token,
                object,
                SubClassWithMethodAlias::new(&token, 7),
                5,
                12 + (7 + 1) + 5
            );

            assert_value_with_added_eq!(
                &token,
                object,
                SubSubClassWithMethodAlias::new(&token, 7),
                5,
                12 + (7 + 2) + 5
            );

            let object = SubClassWithMethodAlias::new(&token, 12).unwrap();

            // Subclass method works.

            assert_value_with_added_eq!(
                &token,
                object,
                SubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 1) + (7 + 1) * 2 + 5
            );

            assert_value_with_added_eq!(
                &token,
                object,
                SubSubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 1) + (7 + 2) * 2 + 5
            );

            // Aliased method works.

            assert_value_with_added_eq!(
                &token,
                *object,
                SimpleClass::new(&token, 7),
                5,
                (12 + 1) + 7 + 5
            );

            assert_value_with_added_eq!(
                &token,
                *object,
                SubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 1) + (7 + 1) + 5
            );

            assert_value_with_added_eq!(
                &token,
                *object,
                SubSubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 1) + (7 + 2) + 5
            );

            let object = SubSubClassWithMethodAlias::new(&token, 12).unwrap();

            // Subsubclass method works.

            assert_value_with_added_eq!(
                &token,
                object,
                SubSubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 2) + (7 + 2) * 3 + 5
            );

            // Aliased subclass method works.

            assert_value_with_added_eq!(
                &token,
                *object,
                SubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 2) + (7 + 1) * 2 + 5
            );

            assert_value_with_added_eq!(
                &token,
                *object,
                SubSubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 2) + (7 + 2) * 2 + 5
            );

            // Aliased method works.

            assert_value_with_added_eq!(
                &token,
                **object,
                SimpleClass::new(&token, 7),
                5,
                (12 + 2) + 7 + 5
            );

            assert_value_with_added_eq!(
                &token,
                **object,
                SubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 2) + (7 + 1) + 5
            );

            assert_value_with_added_eq!(
                &token,
                **object,
                SubSubClassWithMethodAlias::new(&token, 7),
                5,
                (12 + 2) + (7 + 2) + 5
            );

            ((), token)
        })
        .unwrap();
    }
}
