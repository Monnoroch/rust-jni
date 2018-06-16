/// A module with tools used in unit tests.

#[cfg(test)]
use jni_sys;
#[cfg(test)]
use std::ptr;

#[cfg(test)]
pub fn empty_raw_java_vm() -> jni_sys::JNIInvokeInterface_ {
    jni_sys::JNIInvokeInterface_ {
        reserved0: ptr::null_mut(),
        reserved1: ptr::null_mut(),
        reserved2: ptr::null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: None,
        AttachCurrentThreadAsDaemon: None,
    }
}
