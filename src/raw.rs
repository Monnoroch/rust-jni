#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[cfg(test)]
use crate::init_arguments::InitArguments;
use jni_sys;
use std::os::raw::c_void;
#[cfg(test)]
use std::slice;

/// This is a module that wraps `jni_sys` in a way that allows mocking it's functions
/// for unit testing. It defines two versions of each function: a prod one that is just
/// a shallow proxy and a test one, that comes with global variables for
/// mocking the result and the arguments. Tests should populate these variables and then
/// execute the code that depends of these functions.

#[cfg(all(not(test), feature = "libjvm"))]
pub unsafe fn JNI_GetDefaultJavaVMInitArgs(arguments: *mut c_void) -> jni_sys::jint {
    jni_sys::JNI_GetDefaultJavaVMInitArgs(arguments)
}

#[cfg(all(not(test), feature = "libjvm"))]
pub unsafe fn JNI_CreateJavaVM(
    java_vm: *mut *mut jni_sys::JavaVM,
    jni_env: *mut *mut c_void,
    arguments: *mut c_void,
) -> jni_sys::jint {
    jni_sys::JNI_CreateJavaVM(java_vm, jni_env, arguments)
}

#[cfg(all(not(test), feature = "libjvm"))]
pub unsafe fn JNI_GetCreatedJavaVMs(
    java_vms: *mut *mut jni_sys::JavaVM,
    buffer_size: jni_sys::jsize,
    vms_count: *mut jni_sys::jsize,
) -> jni_sys::jint {
    jni_sys::JNI_GetCreatedJavaVMs(java_vms, buffer_size, vms_count)
}

/// Some crates might depend on this crate but not call JNI methods themselves.
/// For these crates we provide dummy implementations of these methods that just panic
/// in case they are called by mistake.

#[cfg(all(not(test), not(feature = "libjvm")))]
pub unsafe fn JNI_GetDefaultJavaVMInitArgs(_: *mut c_void) -> jni_sys::jint {
    unimplemented!()
}

#[cfg(all(not(test), not(feature = "libjvm")))]
pub unsafe fn JNI_CreateJavaVM(
    _: *mut *mut jni_sys::JavaVM,
    _: *mut *mut c_void,
    _: *mut c_void,
) -> jni_sys::jint {
    unimplemented!()
}

#[cfg(all(not(test), not(feature = "libjvm")))]
pub unsafe fn JNI_GetCreatedJavaVMs(
    _: *mut *mut jni_sys::JavaVM,
    _: jni_sys::jsize,
    _: *mut jni_sys::jsize,
) -> jni_sys::jint {
    unimplemented!()
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
    // Tests for code that calls `JNI_GetDefaultJavaVMInitArgs`  must be single-threaded
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
        .0
        != ptr::null_mut()
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

#[cfg(test)]
pub struct CreateJavaVMCall {
    input: Option<InitArguments>,
    result: jni_sys::jint,
    set_input: SendPtr<jni_sys::JavaVM>,
}

// Safe for single-threaded tests.
#[cfg(test)]
unsafe impl Send for CreateJavaVMCall {}

#[cfg(test)]
impl CreateJavaVMCall {
    fn empty() -> Self {
        CreateJavaVMCall {
            input: None,
            result: 17,
            set_input: SendPtr(ptr::null_mut()),
        }
    }

    pub fn new(result: jni_sys::jint, set_input: *mut jni_sys::JavaVM) -> Self {
        CreateJavaVMCall {
            input: None,
            result,
            set_input: SendPtr(set_input),
        }
    }
}

#[cfg(test)]
lazy_static! {
    static ref TEST_JNI_CreateJavaVM_Value: Mutex<CreateJavaVMCall> =
        Mutex::new(CreateJavaVMCall::empty());
    static ref TEST_JNI_CreateJavaVM_Lock: Mutex<bool> = Mutex::new(false);
}

#[cfg(test)]
fn create_java_vm_lock() -> MutexGuard<'static, bool> {
    match TEST_JNI_CreateJavaVM_Lock.lock() {
        Ok(lock) => lock,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
/// Mock a call to the `JNI_CreateJavaVM` JNI function.
/// Returns a `MutexGuard` which is used to make tests using this function sequential,
/// which is required because of the use of global mutable variables.
pub fn setup_create_java_vm_call(call: CreateJavaVMCall) -> MutexGuard<'static, bool> {
    // Tests for code that calls `JNI_CreateJavaVM`  must be single-threaded
    // because global mock variables are not thread-safe.
    let lock = create_java_vm_lock();
    *TEST_JNI_CreateJavaVM_Value.lock().unwrap() = call;
    lock
}

#[cfg(test)]
pub fn get_create_java_vm_call_input() -> InitArguments {
    TEST_JNI_CreateJavaVM_Value
        .lock()
        .unwrap()
        .input
        .clone()
        .unwrap()
}

#[cfg(test)]
pub unsafe fn JNI_CreateJavaVM(
    java_vm: *mut *mut jni_sys::JavaVM,
    _jni_env: *mut *mut c_void,
    arguments: *mut c_void,
) -> jni_sys::jint {
    let arguments = arguments as *mut jni_sys::JavaVMInitArgs;
    TEST_JNI_CreateJavaVM_Value.lock().unwrap().input = Some(InitArguments::from_raw(&*arguments));
    if TEST_JNI_CreateJavaVM_Value.lock().unwrap().set_input.0 != ptr::null_mut() {
        let test_value = TEST_JNI_CreateJavaVM_Value.lock().unwrap().set_input.0;
        *java_vm = test_value;
    }
    TEST_JNI_CreateJavaVM_Value.lock().unwrap().result
}

#[cfg(test)]
pub struct GetCreatedJavaVMsCall {
    result_count: jni_sys::jint,
    result_list: jni_sys::jint,
    set_input_count: jni_sys::jsize,
    set_input: SendPtr<*mut jni_sys::JavaVM>,
}

// Safe for single-threaded tests.
#[cfg(test)]
unsafe impl Send for GetCreatedJavaVMsCall {}

#[cfg(test)]
impl GetCreatedJavaVMsCall {
    fn empty() -> Self {
        GetCreatedJavaVMsCall {
            result_count: 17,
            result_list: 17,
            set_input_count: 0,
            set_input: SendPtr(ptr::null_mut()),
        }
    }

    pub fn new(
        result: jni_sys::jint,
        set_input_count: jni_sys::jsize,
        set_input: *mut *mut jni_sys::JavaVM,
    ) -> Self {
        GetCreatedJavaVMsCall {
            result_count: result,
            result_list: result,
            set_input_count,
            set_input: SendPtr(set_input),
        }
    }

    pub fn new_twice(
        result_count: jni_sys::jint,
        result_list: jni_sys::jint,
        set_input_count: jni_sys::jsize,
        set_input: *mut *mut jni_sys::JavaVM,
    ) -> Self {
        GetCreatedJavaVMsCall {
            result_count,
            result_list,
            set_input_count,
            set_input: SendPtr(set_input),
        }
    }
}

#[cfg(test)]
lazy_static! {
    static ref TEST_JNI_GetCreatedJavaVMs: Mutex<GetCreatedJavaVMsCall> =
        Mutex::new(GetCreatedJavaVMsCall::empty());
    static ref TEST_JNI_GetCreatedJavaVMs_Lock: Mutex<bool> = Mutex::new(false);
}

#[cfg(test)]
/// Mock a call to the `JNI_GetCreatedJavaVMs` JNI function.
/// Returns a `MutexGuard` which is used to make tests using this function sequential,
/// which is required because of the use of global mutable variables.
pub fn setup_get_created_java_vms_call(call: GetCreatedJavaVMsCall) -> MutexGuard<'static, bool> {
    // Tests for code that calls `JNI_GetCreatedJavaVMs`  must be single-threaded
    // because global mock variables are not thread-safe.
    let lock = TEST_JNI_GetCreatedJavaVMs_Lock.lock().unwrap();
    *TEST_JNI_GetCreatedJavaVMs.lock().unwrap() = call;
    lock
}

#[cfg(test)]
fn copy_slice<T: Copy>(dst: &mut [T], src: &[T]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = *s;
    }
}

#[cfg(test)]
pub unsafe fn JNI_GetCreatedJavaVMs(
    java_vms: *mut *mut jni_sys::JavaVM,
    buffer_size: jni_sys::jsize,
    vms_count: *mut jni_sys::jsize,
) -> jni_sys::jint {
    if java_vms == ptr::null_mut() {
        let lock = TEST_JNI_GetCreatedJavaVMs.lock().unwrap();
        *vms_count = lock.set_input_count;
        lock.result_count
    } else {
        let lock = TEST_JNI_GetCreatedJavaVMs.lock().unwrap();
        let set_inputs: &mut [*mut jni_sys::JavaVM] =
            slice::from_raw_parts_mut(lock.set_input.0, buffer_size as usize);
        let inputs: &mut [*mut jni_sys::JavaVM] =
            slice::from_raw_parts_mut(java_vms, buffer_size as usize);
        copy_slice(inputs, set_inputs);
        lock.result_list
    }
}
