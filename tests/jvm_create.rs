#[cfg(all(test, feature = "libjvm"))]
mod create_jvm {
    use rust_jni::*;
    use std::ptr;

    #[test]
    fn create_default() {
        let vm = JavaVM::create(&InitArguments::default()).unwrap();
        unsafe { assert_ne!(vm.raw_jvm().as_ptr(), ptr::null_mut()) };

        let vms = JavaVM::list().unwrap();
        assert_eq!(vms.len(), 1);
        unsafe {
            assert_eq!(vms[0].raw_jvm(), vm.raw_jvm());
        }
    }
}
