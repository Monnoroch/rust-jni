extern crate rust_jni;

#[cfg(test)]
mod classes {
    use rust_jni::*;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = JavaVM::create(&init_arguments).unwrap();
        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        let token = env.token();

        // TODO: compare to the `java::lang::String::class()` result.
        let _class = java::lang::Class::find(&env, "java/lang/String", &token).unwrap();
        // TODO: check the message.
        let _exception = java::lang::Class::find(&env, "java/lang/Invalid", &token).unwrap_err();
    }
}
