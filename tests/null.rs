/// An integration test for null values.
#[cfg(all(test, feature = "libjvm"))]
mod null {
    use rust_jni::java::lang::Object;
    use rust_jni::*;
    use std::ptr;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm
            .attach(&AttachArguments::new(init_arguments.version()))
            .unwrap();
        let token = env.token();

        let null = Object::null(&env);
        unsafe {
            assert_eq!(null.env().raw_env(), env.raw_env());
            assert_eq!(null.raw_object(), ptr::null_mut());
        }
        assert!(null.is_null());
        assert!(null.is_same_as(&Object::null(&env), &token));

        let null_pointer_exception = null.to_string(&token).unwrap_err();
    }
}
