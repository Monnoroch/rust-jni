use jni_sys;
use std::ptr;

/// Create an empty Java VM interface control structure for testing purposes.
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

/// Create an empty JNI interface control structure for testing purposes.
pub fn empty_raw_jni_env() -> jni_sys::JNINativeInterface_ {
    jni_sys::JNINativeInterface_ {
        reserved0: ptr::null_mut(),
        reserved1: ptr::null_mut(),
        reserved2: ptr::null_mut(),
        reserved3: ptr::null_mut(),
        GetVersion: None,
        DefineClass: None,
        FindClass: None,
        FromReflectedMethod: None,
        FromReflectedField: None,
        ToReflectedMethod: None,
        GetSuperclass: None,
        IsAssignableFrom: None,
        ToReflectedField: None,
        Throw: None,
        ThrowNew: None,
        ExceptionOccurred: None,
        ExceptionDescribe: None,
        ExceptionClear: None,
        FatalError: None,
        PushLocalFrame: None,
        PopLocalFrame: None,
        NewGlobalRef: None,
        DeleteGlobalRef: None,
        DeleteLocalRef: None,
        IsSameObject: None,
        NewLocalRef: None,
        EnsureLocalCapacity: None,
        AllocObject: None,
        NewObject: None,
        NewObjectV: None,
        NewObjectA: None,
        GetObjectClass: None,
        IsInstanceOf: None,
        GetMethodID: None,
        CallObjectMethod: None,
        CallObjectMethodV: None,
        CallObjectMethodA: None,
        CallBooleanMethod: None,
        CallBooleanMethodV: None,
        CallBooleanMethodA: None,
        CallByteMethod: None,
        CallByteMethodV: None,
        CallByteMethodA: None,
        CallCharMethod: None,
        CallCharMethodV: None,
        CallCharMethodA: None,
        CallShortMethod: None,
        CallShortMethodV: None,
        CallShortMethodA: None,
        CallIntMethod: None,
        CallIntMethodV: None,
        CallIntMethodA: None,
        CallLongMethod: None,
        CallLongMethodV: None,
        CallLongMethodA: None,
        CallFloatMethod: None,
        CallFloatMethodV: None,
        CallFloatMethodA: None,
        CallDoubleMethod: None,
        CallDoubleMethodV: None,
        CallDoubleMethodA: None,
        CallVoidMethod: None,
        CallVoidMethodV: None,
        CallVoidMethodA: None,
        CallNonvirtualObjectMethod: None,
        CallNonvirtualObjectMethodV: None,
        CallNonvirtualObjectMethodA: None,
        CallNonvirtualBooleanMethod: None,
        CallNonvirtualBooleanMethodV: None,
        CallNonvirtualBooleanMethodA: None,
        CallNonvirtualByteMethod: None,
        CallNonvirtualByteMethodV: None,
        CallNonvirtualByteMethodA: None,
        CallNonvirtualCharMethod: None,
        CallNonvirtualCharMethodV: None,
        CallNonvirtualCharMethodA: None,
        CallNonvirtualShortMethod: None,
        CallNonvirtualShortMethodV: None,
        CallNonvirtualShortMethodA: None,
        CallNonvirtualIntMethod: None,
        CallNonvirtualIntMethodV: None,
        CallNonvirtualIntMethodA: None,
        CallNonvirtualLongMethod: None,
        CallNonvirtualLongMethodV: None,
        CallNonvirtualLongMethodA: None,
        CallNonvirtualFloatMethod: None,
        CallNonvirtualFloatMethodV: None,
        CallNonvirtualFloatMethodA: None,
        CallNonvirtualDoubleMethod: None,
        CallNonvirtualDoubleMethodV: None,
        CallNonvirtualDoubleMethodA: None,
        CallNonvirtualVoidMethod: None,
        CallNonvirtualVoidMethodV: None,
        CallNonvirtualVoidMethodA: None,
        GetFieldID: None,
        GetObjectField: None,
        GetBooleanField: None,
        GetByteField: None,
        GetCharField: None,
        GetShortField: None,
        GetIntField: None,
        GetLongField: None,
        GetFloatField: None,
        GetDoubleField: None,
        SetObjectField: None,
        SetBooleanField: None,
        SetByteField: None,
        SetCharField: None,
        SetShortField: None,
        SetIntField: None,
        SetLongField: None,
        SetFloatField: None,
        SetDoubleField: None,
        GetStaticMethodID: None,
        CallStaticObjectMethod: None,
        CallStaticObjectMethodV: None,
        CallStaticObjectMethodA: None,
        CallStaticBooleanMethod: None,
        CallStaticBooleanMethodV: None,
        CallStaticBooleanMethodA: None,
        CallStaticByteMethod: None,
        CallStaticByteMethodV: None,
        CallStaticByteMethodA: None,
        CallStaticCharMethod: None,
        CallStaticCharMethodV: None,
        CallStaticCharMethodA: None,
        CallStaticShortMethod: None,
        CallStaticShortMethodV: None,
        CallStaticShortMethodA: None,
        CallStaticIntMethod: None,
        CallStaticIntMethodV: None,
        CallStaticIntMethodA: None,
        CallStaticLongMethod: None,
        CallStaticLongMethodV: None,
        CallStaticLongMethodA: None,
        CallStaticFloatMethod: None,
        CallStaticFloatMethodV: None,
        CallStaticFloatMethodA: None,
        CallStaticDoubleMethod: None,
        CallStaticDoubleMethodV: None,
        CallStaticDoubleMethodA: None,
        CallStaticVoidMethod: None,
        CallStaticVoidMethodV: None,
        CallStaticVoidMethodA: None,
        GetStaticFieldID: None,
        GetStaticObjectField: None,
        GetStaticBooleanField: None,
        GetStaticByteField: None,
        GetStaticCharField: None,
        GetStaticShortField: None,
        GetStaticIntField: None,
        GetStaticLongField: None,
        GetStaticFloatField: None,
        GetStaticDoubleField: None,
        SetStaticObjectField: None,
        SetStaticBooleanField: None,
        SetStaticByteField: None,
        SetStaticCharField: None,
        SetStaticShortField: None,
        SetStaticIntField: None,
        SetStaticLongField: None,
        SetStaticFloatField: None,
        SetStaticDoubleField: None,
        NewString: None,
        GetStringLength: None,
        GetStringChars: None,
        ReleaseStringChars: None,
        NewStringUTF: None,
        GetStringUTFLength: None,
        GetStringUTFChars: None,
        ReleaseStringUTFChars: None,
        GetArrayLength: None,
        NewObjectArray: None,
        GetObjectArrayElement: None,
        SetObjectArrayElement: None,
        NewBooleanArray: None,
        NewByteArray: None,
        NewCharArray: None,
        NewShortArray: None,
        NewIntArray: None,
        NewLongArray: None,
        NewFloatArray: None,
        NewDoubleArray: None,
        GetBooleanArrayElements: None,
        GetByteArrayElements: None,
        GetCharArrayElements: None,
        GetShortArrayElements: None,
        GetIntArrayElements: None,
        GetLongArrayElements: None,
        GetFloatArrayElements: None,
        GetDoubleArrayElements: None,
        ReleaseBooleanArrayElements: None,
        ReleaseByteArrayElements: None,
        ReleaseCharArrayElements: None,
        ReleaseShortArrayElements: None,
        ReleaseIntArrayElements: None,
        ReleaseLongArrayElements: None,
        ReleaseFloatArrayElements: None,
        ReleaseDoubleArrayElements: None,
        GetBooleanArrayRegion: None,
        GetByteArrayRegion: None,
        GetCharArrayRegion: None,
        GetShortArrayRegion: None,
        GetIntArrayRegion: None,
        GetLongArrayRegion: None,
        GetFloatArrayRegion: None,
        GetDoubleArrayRegion: None,
        SetBooleanArrayRegion: None,
        SetByteArrayRegion: None,
        SetCharArrayRegion: None,
        SetShortArrayRegion: None,
        SetIntArrayRegion: None,
        SetLongArrayRegion: None,
        SetFloatArrayRegion: None,
        SetDoubleArrayRegion: None,
        RegisterNatives: None,
        UnregisterNatives: None,
        MonitorEnter: None,
        MonitorExit: None,
        GetJavaVM: None,
        GetStringRegion: None,
        GetStringUTFRegion: None,
        GetPrimitiveArrayCritical: None,
        ReleasePrimitiveArrayCritical: None,
        GetStringCritical: None,
        ReleaseStringCritical: None,
        NewWeakGlobalRef: None,
        DeleteWeakGlobalRef: None,
        ExceptionCheck: None,
        NewDirectByteBuffer: None,
        GetDirectBufferAddress: None,
        GetDirectBufferCapacity: None,
        GetObjectRefType: None,
    }
}

/// A macro to generate a mock of global JNI functions.
/// The macro creates global variables and thus needs to be a macro, not a single definition.
#[macro_export]
macro_rules! generate_jni_functions_mock {
    ($module:ident) => {
        // JNI API.
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        mod $module {
            use mockall::*;

            #[automock]
            pub mod ffi {
                extern "C" {
                    pub fn JNI_CreateJavaVM(
                        java_vm: *mut *mut jni_sys::JavaVM,
                        jni_env: *mut *mut ::std::os::raw::c_void,
                        arguments: *mut ::std::os::raw::c_void,
                    ) -> jni_sys::jint;

                    pub fn JNI_GetCreatedJavaVMs(
                        java_vms: *mut *mut jni_sys::JavaVM,
                        buffer_size: jni_sys::jsize,
                        vms_count: *mut jni_sys::jsize,
                    ) -> jni_sys::jint;

                    pub fn JNI_GetDefaultJavaVMInitArgs(
                        arguments: *mut ::std::os::raw::c_void,
                    ) -> jni_sys::jint;
                }
            }

            pub use mock_ffi::*;
        }

        // Re-import to use in production code.

        #[allow(unused_imports)]
        use self::$module::mock_ffi::JNI_CreateJavaVM;
        #[allow(unused_imports)]
        use self::$module::mock_ffi::JNI_GetCreatedJavaVMs;
        #[allow(unused_imports)]
        use self::$module::mock_ffi::JNI_GetDefaultJavaVMInitArgs;
    };
}

/// A macro to generate a mock of a Java VM.
/// The macro creates global variables and thus needs to be a macro, not a single definition.
#[macro_export]
macro_rules! generate_java_vm_mock {
    ($module:ident) => {
        // We're not using the non-test function.
        #[allow(dead_code)]
        mod $module {
            use mockall::*;
            use std;
            use std::ffi::c_void;

            #[automock]
            pub mod ffi {
                extern "Rust" {
                    pub fn destroy_vm(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint;

                    pub fn detach_thread(java_vm: *mut jni_sys::JavaVM) -> jni_sys::jint;

                    pub fn get_env(
                        java_vm: *mut jni_sys::JavaVM,
                        jni_env: *mut *mut std::ffi::c_void,
                        version: jni_sys::jint,
                    ) -> jni_sys::jint;

                    pub fn attach_current_thread(
                        java_vm: *mut jni_sys::JavaVM,
                        jni_env: *mut *mut std::ffi::c_void,
                        argument: *mut std::ffi::c_void,
                    ) -> jni_sys::jint;

                    pub fn attach_current_thread_as_daemon(
                        java_vm: *mut jni_sys::JavaVM,
                        jni_env: *mut *mut std::ffi::c_void,
                        argument: *mut std::ffi::c_void,
                    ) -> jni_sys::jint;
                }
            }

            /// Create a mock Java VM interface control structure for testing purposes.
            pub fn raw_java_vm() -> jni_sys::JNIInvokeInterface_ {
                unsafe extern "system" fn destroy_vm_impl(
                    java_vm: *mut jni_sys::JavaVM,
                ) -> jni_sys::jint {
                    mock_ffi::destroy_vm(java_vm)
                }

                unsafe extern "system" fn detach_thread_impl(
                    java_vm: *mut jni_sys::JavaVM,
                ) -> jni_sys::jint {
                    mock_ffi::detach_thread(java_vm)
                }

                unsafe extern "system" fn get_env_impl(
                    java_vm: *mut jni_sys::JavaVM,
                    jni_env: *mut *mut c_void,
                    version: jni_sys::jint,
                ) -> jni_sys::jint {
                    mock_ffi::get_env(java_vm, jni_env, version)
                }

                unsafe extern "system" fn attach_current_thread_impl(
                    java_vm: *mut jni_sys::JavaVM,
                    jni_env: *mut *mut c_void,
                    argument: *mut c_void,
                ) -> jni_sys::jint {
                    mock_ffi::attach_current_thread(java_vm, jni_env, argument)
                }

                unsafe extern "system" fn attach_current_thread_as_daemon_impl(
                    java_vm: *mut jni_sys::JavaVM,
                    jni_env: *mut *mut c_void,
                    argument: *mut c_void,
                ) -> jni_sys::jint {
                    mock_ffi::attach_current_thread_as_daemon(java_vm, jni_env, argument)
                }

                jni_sys::JNIInvokeInterface_ {
                    DestroyJavaVM: Some(destroy_vm_impl),
                    DetachCurrentThread: Some(detach_thread_impl),
                    GetEnv: Some(get_env_impl),
                    AttachCurrentThread: Some(attach_current_thread_impl),
                    AttachCurrentThreadAsDaemon: Some(attach_current_thread_as_daemon_impl),
                    ..$crate::testing::empty_raw_java_vm()
                }
            }

            pub use self::mock_ffi::*;
        }
    };
}

/// A macro to generate a mock of a Java VM.
/// The macro creates global variables and thus needs to be a macro, not a single definition.
#[macro_export]
macro_rules! generate_jni_env_mock {
    ($module:ident) => {
        // We're not using the non-test function.
        #[allow(dead_code)]
        mod $module {
            use mockall::*;

            #[automock]
            pub mod ffi {
                extern "Rust" {
                    pub fn delete_local_ref(
                        java_vm: *mut jni_sys::JNIEnv,
                        object: jni_sys::jobject,
                    );

                    pub fn get_version(env: *mut jni_sys::JNIEnv) -> jni_sys::jint;

                    pub fn exception_check(env: *mut jni_sys::JNIEnv) -> jni_sys::jboolean;

                    pub fn exception_describe(env: *mut jni_sys::JNIEnv);

                    pub fn exception_occured(env: *mut jni_sys::JNIEnv) -> jni_sys::jobject;

                    pub fn exception_clear(env: *mut jni_sys::JNIEnv);
                }
            }

            /// Create a mock JNI interface control structure for testing purposes.
            pub fn raw_jni_env() -> jni_sys::JNINativeInterface_ {
                unsafe extern "system" fn delete_local_ref_impl(
                    java_vm: *mut jni_sys::JNIEnv,
                    object: jni_sys::jobject,
                ) {
                    mock_ffi::delete_local_ref(java_vm, object)
                }

                unsafe extern "system" fn get_version_impl(
                    env: *mut jni_sys::JNIEnv,
                ) -> jni_sys::jint {
                    mock_ffi::get_version(env)
                }

                unsafe extern "system" fn exception_check_impl(
                    env: *mut ::jni_sys::JNIEnv,
                ) -> jni_sys::jboolean {
                    mock_ffi::exception_check(env)
                }

                unsafe extern "system" fn exception_describe_impl(env: *mut ::jni_sys::JNIEnv) {
                    mock_ffi::exception_describe(env)
                }

                unsafe extern "system" fn exception_occured_impl(
                    env: *mut jni_sys::JNIEnv,
                ) -> jni_sys::jobject {
                    mock_ffi::exception_occured(env)
                }

                unsafe extern "system" fn exception_clear_impl(env: *mut jni_sys::JNIEnv) {
                    mock_ffi::exception_clear(env)
                }

                jni_sys::JNINativeInterface_ {
                    DeleteLocalRef: Some(delete_local_ref_impl),
                    GetVersion: Some(get_version_impl),
                    ExceptionCheck: Some(exception_check_impl),
                    ExceptionDescribe: Some(exception_describe_impl),
                    ExceptionOccurred: Some(exception_occured_impl),
                    ExceptionClear: Some(exception_clear_impl),
                    ..$crate::testing::empty_raw_jni_env()
                }
            }

            pub use self::mock_ffi::*;
        }
    };
}
