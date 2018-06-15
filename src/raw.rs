#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

/// This is a module that wraps `jni_sys` in a way that allows mocking it's functions
/// for unit testing. It defines two versions of each function: a prod one that is just
/// a shallow proxy and a test one, that comes with global variables for
/// mocking the result and the arguments. Tests should populate these variables and then
/// execute the code that depends of these functions.
use jni_sys;
use std::os::raw::c_void;

#[cfg(not(test))]
pub unsafe fn JNI_GetDefaultJavaVMInitArgs(arguments: *mut c_void) -> jni_sys::jint {
    jni_sys::JNI_GetDefaultJavaVMInitArgs(arguments)
}

#[cfg(test)]
use std::ptr;
#[cfg(test)]
use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
pub struct SendPtr<T>(pub *mut T);

// Safe to share in tests.
#[cfg(test)]
unsafe impl<T> Send for SendPtr<T> {}

#[cfg(test)]
pub struct GetDefaultJavaVMInitArgsCall {
    input: Option<jni_sys::JavaVMInitArgs>,
    result: jni_sys::jint,
    set_input: SendPtr<c_void>,
}

// Safe for single-threaded tests.
#[cfg(test)]
unsafe impl Send for GetDefaultJavaVMInitArgsCall {}

#[cfg(test)]
impl GetDefaultJavaVMInitArgsCall {
    fn empty() -> Self {
        GetDefaultJavaVMInitArgsCall {
            input: None,
            result: 17,
            set_input: SendPtr(ptr::null_mut()),
        }
    }

    pub fn new(set_input: *mut c_void) -> Self {
        GetDefaultJavaVMInitArgsCall {
            input: None,
            result: 17,
            set_input: SendPtr(set_input),
        }
    }
}

#[cfg(test)]
lazy_static! {
    static ref TEST_JNI_GetDefaultJavaVMInitArgs: Mutex<GetDefaultJavaVMInitArgsCall> =
        Mutex::new(GetDefaultJavaVMInitArgsCall::empty());
    static ref TEST_JNI_GetDefaultJavaVMInitArgs_Lock: Mutex<bool> = Mutex::new(false);
}

#[cfg(test)]
/// Mock a call to the `JNI_GetDefaultJavaVMInitArgs` JNI function.
/// Returns a `MutexGuard` which is used to make tests using this function sequential,
/// which is required because of the use of global mutable variables.
pub fn setup_get_default_java_vm_init_args_call(
    call: GetDefaultJavaVMInitArgsCall,
) -> MutexGuard<'static, bool> {
    // Tests for code that calls JNI_GetDefaultJavaVMInitArgs  must be single-threaded
    // because global mock variables are not thread-safe.
    let lock = TEST_JNI_GetDefaultJavaVMInitArgs_Lock.lock().unwrap();
    *TEST_JNI_GetDefaultJavaVMInitArgs.lock().unwrap() = call;
    lock
}

#[cfg(test)]
pub fn get_default_java_vm_init_args_call_input() -> jni_sys::JavaVMInitArgs {
    TEST_JNI_GetDefaultJavaVMInitArgs
        .lock()
        .unwrap()
        .input
        .unwrap()
}

#[cfg(test)]
pub unsafe fn JNI_GetDefaultJavaVMInitArgs(arguments: *mut c_void) -> jni_sys::jint {
    let arguments = arguments as *mut jni_sys::JavaVMInitArgs;
    TEST_JNI_GetDefaultJavaVMInitArgs.lock().unwrap().input = Some(*arguments);
    if TEST_JNI_GetDefaultJavaVMInitArgs
        .lock()
        .unwrap()
        .set_input
        .0 != ptr::null_mut()
    {
        let test_value = TEST_JNI_GetDefaultJavaVMInitArgs
            .lock()
            .unwrap()
            .set_input
            .0 as *mut jni_sys::JavaVMInitArgs;
        *arguments = *test_value;
    }
    TEST_JNI_GetDefaultJavaVMInitArgs.lock().unwrap().result
}
