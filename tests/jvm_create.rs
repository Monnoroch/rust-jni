#[cfg(all(test, feature = "libjvm"))]
mod create_jvm {
    use rust_jni::*;
    use std::ptr;

    #[test]
    fn create_default() {
        let vm = JavaVM::create(&InitArguments::get_default(JniVersion::V8).unwrap()).unwrap();
        unsafe { assert_ne!(vm.raw_jvm(), ptr::null_mut()) };

        let vms = JavaVM::list().unwrap();
        assert_eq!(vms.len(), 1);
        unsafe {
            assert_eq!(vms[0].raw_jvm(), vm.raw_jvm());
        }
    }
}
