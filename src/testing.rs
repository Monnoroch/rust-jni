/// A module with tools used in unit tests.
use java_string::*;
use jni_sys;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_char;
use std::ptr;

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

macro_rules! __generate_method_check_impl {
    ($method:ident, $type:ident, fn($($argument_name:ident: $argument_type:ty),*) -> $resutl_type:ty, $code:expr) => {
        impl $type {
            pub unsafe fn __check_call(
                calls: &mut JniCalls,
                env: *mut jni_sys::JNIEnv,
                $($argument_name: $argument_type,)*
            ) -> $resutl_type {
                match calls.__check_method_call(env, stringify!($method)) {
                    JniCall::$method(ref call) => call.__check_call_impl(
                        $($argument_name),*
                    ),
                    ref call => panic!(
                        "Unexpected call {:?}, actual call: {}.",
                        call,
                        stringify!($method)
                    ),
                }
            }

            pub unsafe fn __check_call_impl(
                &self,
                $($argument_name: $argument_type,)*
            ) -> $resutl_type {
                $code(self)
            }
        }
    };
}

#[doc(hidden)]
#[derive(Debug)]
pub struct DeleteLocalRefCall {
    pub object: jni_sys::jobject,
}

impl DeleteLocalRefCall {
    pub unsafe fn __check_call(
        calls: &mut JniCalls,
        env: *mut jni_sys::JNIEnv,
        object: jni_sys::jobject,
    ) {
        assert_eq!(env, calls.env);
        let current_call = calls.current_call;
        if current_call >= calls.calls.len() {
            // These are destructors for local variables. Could do explicit `mem::forget` on
            // all of them instead, but that will only introduce unnecessary noise.
            return;
        }

        match calls.__check_method_call(env, stringify!(DeleteLocalRef)) {
            JniCall::DeleteLocalRef(ref call) => call.__check_call_impl(object),
            ref call => panic!(
                "Unexpected call {:?}, actual call: {}.",
                call,
                stringify!(DeleteLocalRef)
            ),
        }
    }

    pub unsafe fn __check_call_impl(&self, object: jni_sys::jobject) {
        assert_eq!(object, self.object);
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NewLocalRefCall {
    pub object: jni_sys::jobject,
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    NewLocalRef,
    NewLocalRefCall,
    fn(object: jni_sys::jobject) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(object, call.object);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionOccurredCall {
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    ExceptionOccurred,
    ExceptionOccurredCall,
    fn() -> jni_sys::jobject,
    |call: &Self| call.result
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionClearCall {}

__generate_method_check_impl!(ExceptionClear, ExceptionClearCall, fn() -> (), |_| ());

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionCheckCall {
    pub result: jni_sys::jboolean,
}

__generate_method_check_impl!(
    ExceptionCheck,
    ExceptionCheckCall,
    fn() -> jni_sys::jboolean,
    |call: &Self| call.result
);

#[doc(hidden)]
#[derive(Debug)]
pub struct IsSameObjectCall {
    pub object1: jni_sys::jobject,
    pub object2: jni_sys::jobject,
    pub result: jni_sys::jboolean,
}

__generate_method_check_impl!(
    IsSameObject,
    IsSameObjectCall,
    fn(object1: jni_sys::jobject, object2: jni_sys::jobject) -> jni_sys::jboolean,
    |call: &Self| {
        assert_eq!(object1, call.object1);
        assert_eq!(object2, call.object2);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetObjectClassCall {
    pub object: jni_sys::jobject,
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    GetObjectClass,
    GetObjectClassCall,
    fn(object: jni_sys::jobject) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(object, call.object);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct IsInstanceOfCall {
    pub object: jni_sys::jobject,
    pub class: jni_sys::jobject,
    pub result: jni_sys::jboolean,
}

__generate_method_check_impl!(
    IsInstanceOf,
    IsInstanceOfCall,
    fn(object: jni_sys::jobject, class: jni_sys::jobject) -> jni_sys::jboolean,
    |call: &Self| {
        assert_eq!(object, call.object);
        assert_eq!(class, call.class);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ThrowCall {
    pub object: jni_sys::jobject,
    pub result: jni_sys::jint,
}

__generate_method_check_impl!(
    Throw,
    ThrowCall,
    fn(object: jni_sys::jobject) -> jni_sys::jint,
    |call: &Self| {
        assert_eq!(object, call.object);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct FindClassCall {
    pub name: String,
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    FindClass,
    FindClassCall,
    fn(name: *const c_char) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(
            from_java_string(CStr::from_ptr(name).to_bytes_with_nul()).unwrap(),
            call.name
        );
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct IsAssignableFromCall {
    pub class1: jni_sys::jobject,
    pub class2: jni_sys::jobject,
    pub result: jni_sys::jboolean,
}

__generate_method_check_impl!(
    IsAssignableFrom,
    IsAssignableFromCall,
    fn(class1: jni_sys::jobject, class2: jni_sys::jobject) -> jni_sys::jboolean,
    |call: &Self| {
        assert_eq!(class1, call.class1);
        assert_eq!(class2, call.class2);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetSuperclassCall {
    pub class: jni_sys::jobject,
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    GetSuperclass,
    GetSuperclassCall,
    fn(class: jni_sys::jobject) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(class, call.class);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct NewStringCall {
    pub name: *const jni_sys::jchar,
    pub size: jni_sys::jsize,
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    NewString,
    NewStringCall,
    fn(name: *const jni_sys::jchar, size: jni_sys::jsize) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(name, call.name);
        assert_eq!(size, call.size);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct NewStringUTFCall {
    pub string: String,
    pub result: jni_sys::jobject,
}

__generate_method_check_impl!(
    NewStringUTF,
    NewStringUTFCall,
    fn(string: *const c_char) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(
            from_java_string(CStr::from_ptr(string).to_bytes_with_nul()).unwrap(),
            call.string
        );
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetStringLengthCall {
    pub string: jni_sys::jobject,
    pub result: jni_sys::jsize,
}

__generate_method_check_impl!(
    GetStringLength,
    GetStringLengthCall,
    fn(string: jni_sys::jobject) -> jni_sys::jsize,
    |call: &Self| {
        assert_eq!(string, call.string);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetStringUTFLengthCall {
    pub string: jni_sys::jobject,
    pub result: jni_sys::jsize,
}

__generate_method_check_impl!(
    GetStringUTFLength,
    GetStringUTFLengthCall,
    fn(string: jni_sys::jobject) -> jni_sys::jsize,
    |call: &Self| {
        assert_eq!(string, call.string);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetMethodIDCall {
    pub class: jni_sys::jclass,
    pub name: String,
    pub signature: String,
    pub result: jni_sys::jmethodID,
}

__generate_method_check_impl!(
    GetMethodID,
    GetMethodIDCall,
    fn(class: jni_sys::jobject, name: *const c_char, signature: *const c_char)
        -> jni_sys::jmethodID,
    |call: &Self| {
        assert_eq!(class, call.class);
        assert_eq!(
            from_java_string(CStr::from_ptr(name).to_bytes_with_nul()).unwrap(),
            call.name
        );
        assert_eq!(
            from_java_string(CStr::from_ptr(signature).to_bytes_with_nul()).unwrap(),
            call.signature
        );
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetStaticMethodIDCall {
    pub class: jni_sys::jclass,
    pub name: String,
    pub signature: String,
    pub result: jni_sys::jmethodID,
}

__generate_method_check_impl!(
    GetStaticMethodID,
    GetStaticMethodIDCall,
    fn(class: jni_sys::jobject, name: *const c_char, signature: *const c_char)
        -> jni_sys::jmethodID,
    |call: &Self| {
        assert_eq!(class, call.class);
        assert_eq!(
            from_java_string(CStr::from_ptr(name).to_bytes_with_nul()).unwrap(),
            call.name
        );
        assert_eq!(
            from_java_string(CStr::from_ptr(signature).to_bytes_with_nul()).unwrap(),
            call.signature
        );
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetStringUTFRegionCall {
    pub string: jni_sys::jstring,
    pub start: jni_sys::jsize,
    pub len: jni_sys::jsize,
    pub buffer: String,
}

__generate_method_check_impl!(
    GetStringUTFRegion,
    GetStringUTFRegionCall,
    fn(string: jni_sys::jstring, start: jni_sys::jsize, len: jni_sys::jsize, buffer: *mut c_char)
        -> (),
    |call: &Self| {
        assert_eq!(string, call.string);
        assert_eq!(start, call.start);
        assert_eq!(len, call.len);
        assert_ne!(buffer, ptr::null_mut());
        let test_buffer = to_java_string(&call.buffer);
        for i in 0..test_buffer.len() {
            *buffer.offset(i as isize) = test_buffer[i] as i8;
        }
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionDescribeCall {}

__generate_method_check_impl!(ExceptionDescribe, ExceptionDescribeCall, fn() -> (), |_| ());

#[doc(hidden)]
#[derive(Debug)]
pub enum JniCall {
    DeleteLocalRef(DeleteLocalRefCall),
    NewLocalRef(NewLocalRefCall),
    ExceptionOccurred(ExceptionOccurredCall),
    ExceptionClear(ExceptionClearCall),
    ExceptionCheck(ExceptionCheckCall),
    IsSameObject(IsSameObjectCall),
    GetObjectClass(GetObjectClassCall),
    IsInstanceOf(IsInstanceOfCall),
    Throw(ThrowCall),
    FindClass(FindClassCall),
    IsAssignableFrom(IsAssignableFromCall),
    GetSuperclass(GetSuperclassCall),
    NewString(NewStringCall),
    NewStringUTF(NewStringUTFCall),
    GetStringLength(GetStringLengthCall),
    GetStringUTFLength(GetStringUTFLengthCall),
    GetMethodID(GetMethodIDCall),
    GetStaticMethodID(GetStaticMethodIDCall),
    GetStringUTFRegion(GetStringUTFRegionCall),
    ExceptionDescribe(ExceptionDescribeCall),
}

#[doc(hidden)]
#[derive(Debug)]
pub struct JniCalls {
    pub calls: Vec<JniCall>,
    pub current_call: usize,
    pub env: *mut jni_sys::JNIEnv,
}

impl Drop for JniCalls {
    fn drop(&mut self) {
        // Test that all expected calls have happened.
        assert_eq!(self.current_call, self.calls.len());
    }
}

unsafe impl Send for JniCalls {}
unsafe impl Sync for JniCalls {}

impl JniCalls {
    pub fn new(calls: Vec<JniCall>) -> Self {
        Self {
            current_call: 0,
            calls,
            env: ptr::null_mut(),
        }
    }

    pub fn __check_method_call<'a>(
        &'a mut self,
        env: *mut jni_sys::JNIEnv,
        method_name: &str,
    ) -> &'a JniCall {
        assert_eq!(env, self.env);
        let current_call = self.current_call;
        if current_call >= self.calls.len() {
            panic!("Unexpected {} method call.", method_name);
        }
        self.current_call += 1;
        &self.calls[current_call]
    }
}

#[doc(hidden)]
pub unsafe fn __to_static_ref<T>(reference: &'static T) -> &'static mut T {
    #[allow(mutable_transmutes)]
    mem::transmute::<_, &'static mut T>(reference)
}

#[macro_export]
macro_rules! test_raw_jni_env {
    ($calls:expr) => {{
        lazy_static! {
            static ref CALLS: JniCalls = JniCalls::new($calls);
        }
        unsafe extern "system" fn delete_local_ref(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) {
            DeleteLocalRefCall::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn new_local_ref(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) -> ::jni_sys::jobject {
            NewLocalRefCall::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn exception_occured(
            env: *mut ::jni_sys::JNIEnv,
        ) -> ::jni_sys::jobject {
            ExceptionOccurredCall::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn exception_clear(env: *mut ::jni_sys::JNIEnv) {
            ExceptionClearCall::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn exception_check(
            env: *mut ::jni_sys::JNIEnv,
        ) -> ::jni_sys::jboolean {
            ExceptionCheckCall::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn is_same_object(
            env: *mut ::jni_sys::JNIEnv,
            object1: ::jni_sys::jobject,
            object2: ::jni_sys::jobject,
        ) -> ::jni_sys::jboolean {
            IsSameObjectCall::__check_call(__to_static_ref(&CALLS), env, object1, object2)
        }
        unsafe extern "system" fn get_object_class(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) -> ::jni_sys::jobject {
            GetObjectClassCall::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn is_instance_of(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
            class: ::jni_sys::jobject,
        ) -> ::jni_sys::jboolean {
            IsInstanceOfCall::__check_call(__to_static_ref(&CALLS), env, object, class)
        }
        unsafe extern "system" fn throw(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) -> ::jni_sys::jint {
            ThrowCall::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn find_class(
            env: *mut ::jni_sys::JNIEnv,
            name: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jobject {
            FindClassCall::__check_call(__to_static_ref(&CALLS), env, name)
        }
        unsafe extern "system" fn is_assignable_from(
            env: *mut ::jni_sys::JNIEnv,
            class1: ::jni_sys::jobject,
            class2: ::jni_sys::jobject,
        ) -> ::jni_sys::jboolean {
            IsAssignableFromCall::__check_call(__to_static_ref(&CALLS), env, class1, class2)
        }
        unsafe extern "system" fn get_superclass(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
        ) -> ::jni_sys::jobject {
            GetSuperclassCall::__check_call(__to_static_ref(&CALLS), env, class)
        }
        unsafe extern "system" fn new_string(
            env: *mut ::jni_sys::JNIEnv,
            name: *const ::jni_sys::jchar,
            size: ::jni_sys::jsize,
        ) -> ::jni_sys::jobject {
            NewStringCall::__check_call(__to_static_ref(&CALLS), env, name, size)
        }
        unsafe extern "system" fn new_string_utf(
            env: *mut ::jni_sys::JNIEnv,
            string: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jobject {
            NewStringUTFCall::__check_call(__to_static_ref(&CALLS), env, string)
        }
        unsafe extern "system" fn get_string_length(
            env: *mut ::jni_sys::JNIEnv,
            string: ::jni_sys::jobject,
        ) -> ::jni_sys::jsize {
            GetStringLengthCall::__check_call(__to_static_ref(&CALLS), env, string)
        }
        unsafe extern "system" fn get_string_utf_length(
            env: *mut ::jni_sys::JNIEnv,
            string: ::jni_sys::jobject,
        ) -> ::jni_sys::jsize {
            GetStringUTFLengthCall::__check_call(__to_static_ref(&CALLS), env, string)
        }
        unsafe extern "system" fn get_method_id(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
            name: *const ::std::os::raw::c_char,
            signature: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jmethodID {
            GetMethodIDCall::__check_call(__to_static_ref(&CALLS), env, class, name, signature)
        }
        unsafe extern "system" fn get_static_method_id(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
            name: *const ::std::os::raw::c_char,
            signature: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jmethodID {
            GetStaticMethodIDCall::__check_call(
                __to_static_ref(&CALLS),
                env,
                class,
                name,
                signature,
            )
        }
        unsafe extern "system" fn get_string_utf_region(
            env: *mut ::jni_sys::JNIEnv,
            string: ::jni_sys::jobject,
            start: ::jni_sys::jsize,
            len: ::jni_sys::jsize,
            buffer: *mut ::std::os::raw::c_char,
        ) {
            GetStringUTFRegionCall::__check_call(
                __to_static_ref(&CALLS),
                env,
                string,
                start,
                len,
                buffer,
            )
        }
        unsafe extern "system" fn exception_describe(env: *mut ::jni_sys::JNIEnv) {
            ExceptionDescribeCall::__check_call(__to_static_ref(&CALLS), env)
        }
        let raw_env = ::jni_sys::JNINativeInterface_ {
            ExceptionDescribe: Some(exception_describe),
            GetStringUTFRegion: Some(get_string_utf_region),
            GetMethodID: Some(get_method_id),
            GetStaticMethodID: Some(get_static_method_id),
            GetStringUTFLength: Some(get_string_utf_length),
            GetStringLength: Some(get_string_length),
            NewStringUTF: Some(new_string_utf),
            NewString: Some(new_string),
            GetSuperclass: Some(get_superclass),
            IsAssignableFrom: Some(is_assignable_from),
            FindClass: Some(find_class),
            Throw: Some(throw),
            IsInstanceOf: Some(is_instance_of),
            DeleteLocalRef: Some(delete_local_ref),
            GetObjectClass: Some(get_object_class),
            NewLocalRef: Some(new_local_ref),
            ExceptionOccurred: Some(exception_occured),
            ExceptionClear: Some(exception_clear),
            ExceptionCheck: Some(exception_check),
            IsSameObject: Some(is_same_object),
            ..empty_raw_jni_env()
        };
        let calls: &'static mut JniCalls = unsafe { __to_static_ref(&CALLS) };
        // The `raw_env` value is supposed to be dropped at the end of this scope, so I have
        // no idea how this is working, but it does.
        calls.env = &mut (&raw_env as ::jni_sys::JNIEnv) as *mut ::jni_sys::JNIEnv;
        calls
    }};
}

pub fn empty_raw_jni_env() -> jni_sys::JNINativeInterface_ {
    unsafe extern "system" fn delete_local_ref(_: *mut jni_sys::JNIEnv, _: jni_sys::jobject) {}

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
        // To not fail during the destructor call.
        DeleteLocalRef: Some(delete_local_ref),
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
