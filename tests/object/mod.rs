#[cfg(all(test, feature = "libjvm"))]
use rust_jni::java::lang::Object;
#[cfg(all(test, feature = "libjvm"))]
use rust_jni::*;

/// An common integration test for types, that inherit from `java::lang::Object`.
#[cfg(all(test, feature = "libjvm"))]
pub fn test_object<'env>(
    object: &Object,
    class_name: &str,
    string_value: &str,
    env: &JniEnv<'env>,
    token: &NoException,
) {
    assert!(!object.is_null());
    assert!(object.is_same_as(object, &token));
    assert_eq!(object, object);
    assert!(object.clone(&token).unwrap().is_same_as(&object, &token));
    assert!(object.is_instance_of(&object.class(&token), &token));

    assert!(object.class(&token).is_same_as(
        &java::lang::Class::find(&env, class_name, &token).unwrap(),
        &token
    ));

    assert_eq!(
        object.to_string(&token).unwrap().as_string(&token),
        string_value
    );
    assert_eq!(format!("{}", object), string_value);
    assert!(format!("{:?}", object).contains(string_value));
}
