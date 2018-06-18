extern crate rust_jni;

#[cfg(test)]
mod create_envs {
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();

        unsafe {
            let native_vm = JavaVM::__for_native_method(env.raw_env()).unwrap();
            assert_eq!(native_vm.raw_jvm(), vm.raw_jvm());

            let native_env = JniEnv::__for_native_method(&native_vm, env.raw_env());
            assert_eq!(native_env.raw_env(), env.raw_env());
            assert_eq!(native_env.version(), env.version());
        }
    }
}
