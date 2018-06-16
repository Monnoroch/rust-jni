extern crate rust_jni;

#[cfg(test)]
mod create_envs {
    use rust_jni::*;
    use std::sync::Arc;

    #[test]
    fn test() {
        let init_arguments = InitArguments::get_default(JniVersion::V8).unwrap();
        let vm = Arc::new(JavaVM::create(&init_arguments).unwrap());

        let env = vm.attach(&AttachArguments::new(&init_arguments)).unwrap();
        unsafe { assert_eq!(env.raw_jvm(), vm.raw_jvm()) };

        let child1 = {
            let vm = vm.clone();
            let attach_arguments = AttachArguments::new(&init_arguments);
            ::std::thread::spawn(move || {
                let _ = vm.attach(&attach_arguments).unwrap();
            })
        };

        let child2 = {
            let vm = vm.clone();
            let attach_arguments = AttachArguments::new(&init_arguments);
            ::std::thread::spawn(move || {
                let _ = vm.attach(&attach_arguments).unwrap();
            })
        };

        child1.join().unwrap();
        child2.join().unwrap();
    }
}
