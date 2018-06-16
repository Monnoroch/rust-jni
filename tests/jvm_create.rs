extern crate rust_jni;

#[cfg(test)]
mod create_jvm {
    use rust_jni::*;
    use std::ptr;

    #[test]
    fn create_default() {
        let vm = JavaVM::create(&InitArguments::get_default(JniVersion::V8).unwrap()).unwrap();
        unsafe { assert_ne!(vm.raw_jvm(), ptr::null_mut()) };
    }
}
