#[cfg(all(test, feature = "libjvm"))]
mod native_method_env {
    use rust_jni::__generator::*;
    use rust_jni::{self, *};
    use std::mem;
    use std::ptr;

    unsafe fn check_exception_message(env: &JniEnv, message: &str, token: &NoException) {
        let raw_env = env.raw_env();
        let exception_occured_fn = (**raw_env).ExceptionOccurred.unwrap();
        let throwable = exception_occured_fn(raw_env);
        assert_ne!(throwable, ptr::null_mut());

        let exception_clear_fn = (**raw_env).ExceptionClear.unwrap();
        exception_clear_fn(raw_env);

        let throwable = java::lang::Throwable::from_jni(&env, throwable);
        let actual_message = throwable.get_message(&token).unwrap();
        assert_eq!(actual_message.as_string(&token), message);
        mem::forget(throwable);
    }

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm
            .attach(&AttachArguments::new(init_arguments.version()))
            .unwrap();
        let token = env.token();

        unsafe {
            // Result in the callback.

            let raw_env = env.raw_env();
            let result = 10;
            let actual_result =
                rust_jni::__generator::native_method_wrapper(raw_env, |native_env, _| {
                    assert_eq!(native_env.raw_jvm(), vm.raw_jvm());
                    assert_eq!(native_env.raw_env(), raw_env);
                    assert_eq!(native_env.version(), init_arguments.version());
                    Ok(result)
                });
            assert_eq!(actual_result, result);

            // Exception in the callback.

            let message = "test-message";
            let actual_result: i32 =
                rust_jni::__generator::native_method_wrapper(raw_env, |native_env, token| {
                    assert_eq!(native_env.raw_jvm(), vm.raw_jvm());
                    assert_eq!(native_env.raw_env(), raw_env);
                    assert_eq!(native_env.version(), init_arguments.version());
                    let message = java::lang::String::new(native_env, message, &token).unwrap();
                    Err(
                        java::lang::Throwable::new_with_message(native_env, &message, &token)
                            .unwrap(),
                    )
                });
            assert_eq!(actual_result, i32::default());
            check_exception_message(&env, message, &token);

            // Panic in the callback.

            let actual_result: i32 =
                rust_jni::__generator::native_method_wrapper(raw_env, |native_env, _| {
                    assert_eq!(native_env.raw_jvm(), vm.raw_jvm());
                    assert_eq!(native_env.raw_env(), raw_env);
                    assert_eq!(native_env.version(), init_arguments.version());
                    panic!(message);
                });
            assert_eq!(actual_result, i32::default());
            check_exception_message(&env, &format!("Rust panic: {}", message), &token);
        }
    }
}
