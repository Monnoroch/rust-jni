use java_string::*;
/// A module with tools used in unit tests.
use jni::{Cast, Object};
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

macro_rules! generate_method_check_impl {
    ($method:ident, fn($($argument_name:ident: $argument_type:ty),*) -> $resutl_type:ty, $code:expr) => {
        impl $method {
            #[doc(hidden)]
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

            unsafe fn __check_call_impl(
                &self,
                $($argument_name: $argument_type,)*
            ) -> $resutl_type {
                $code(self)
            }
        }
    };
    ($method:ident, fn($($argument_name:ident: $argument_type:ty,)*) -> $resutl_type:ty, $code:expr) => {
        generate_method_check_impl!($method, fn($($argument_name: $argument_type),*) -> $resutl_type, $code);
    };
}

#[doc(hidden)]
#[derive(Debug)]
pub struct DeleteLocalRef {
    pub object: jni_sys::jobject,
}

impl DeleteLocalRef {
    #[doc(hidden)]
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

    unsafe fn __check_call_impl(&self, object: jni_sys::jobject) {
        assert_eq!(object, self.object);
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NewLocalRef {
    pub object: jni_sys::jobject,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    NewLocalRef,
    fn(object: jni_sys::jobject) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(object, call.object);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionOccurred {
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    ExceptionOccurred,
    fn() -> jni_sys::jobject,
    |call: &Self| call.result
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionClear {}

generate_method_check_impl!(ExceptionClear, fn() -> (), |_| ());

#[doc(hidden)]
#[derive(Debug)]
pub struct ExceptionCheck {
    pub result: jni_sys::jboolean,
}

generate_method_check_impl!(ExceptionCheck, fn() -> jni_sys::jboolean, |call: &Self| {
    call.result
});

#[doc(hidden)]
#[derive(Debug)]
pub struct IsSameObject {
    pub object1: jni_sys::jobject,
    pub object2: jni_sys::jobject,
    pub result: jni_sys::jboolean,
}

generate_method_check_impl!(
    IsSameObject,
    fn(object1: jni_sys::jobject, object2: jni_sys::jobject) -> jni_sys::jboolean,
    |call: &Self| {
        assert_eq!(object1, call.object1);
        assert_eq!(object2, call.object2);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetObjectClass {
    pub object: jni_sys::jobject,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    GetObjectClass,
    fn(object: jni_sys::jobject) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(object, call.object);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct IsInstanceOf {
    pub object: jni_sys::jobject,
    pub class: jni_sys::jobject,
    pub result: jni_sys::jboolean,
}

generate_method_check_impl!(
    IsInstanceOf,
    fn(object: jni_sys::jobject, class: jni_sys::jobject) -> jni_sys::jboolean,
    |call: &Self| {
        assert_eq!(object, call.object);
        assert_eq!(class, call.class);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct Throw {
    pub object: jni_sys::jobject,
    pub result: jni_sys::jint,
}

generate_method_check_impl!(
    Throw,
    fn(object: jni_sys::jobject) -> jni_sys::jint,
    |call: &Self| {
        assert_eq!(object, call.object);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct ThrowNew {
    pub class: jni_sys::jobject,
    pub message: String,
    pub result: jni_sys::jint,
}

generate_method_check_impl!(
    ThrowNew,
    fn(class: jni_sys::jobject, message: *const c_char) -> jni_sys::jint,
    |call: &Self| {
        assert_eq!(class, call.class);
        assert_eq!(
            from_java_string(CStr::from_ptr(message).to_bytes_with_nul()).unwrap(),
            call.message
        );
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct FindClass {
    pub name: String,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    FindClass,
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
pub struct DefineClass {
    pub name: *const c_char,
    pub loader: jni_sys::jobject,
    pub buffer: Vec<i8>,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    DefineClass,
    fn(
        name: *const c_char,
        loader: jni_sys::jobject,
        buffer: *const jni_sys::jbyte,
        buffer_size: jni_sys::jsize,
    ) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(name, call.name);
        assert_eq!(loader, call.loader);
        assert_eq!(buffer_size, call.buffer.len() as jni_sys::jsize);
        let buffer = Vec::<i8>::from_raw_parts(
            buffer as *mut i8,
            buffer_size as usize,
            buffer_size as usize,
        );
        assert_eq!(buffer, call.buffer);
        mem::forget(buffer);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct IsAssignableFrom {
    pub class1: jni_sys::jobject,
    pub class2: jni_sys::jobject,
    pub result: jni_sys::jboolean,
}

generate_method_check_impl!(
    IsAssignableFrom,
    fn(class1: jni_sys::jobject, class2: jni_sys::jobject) -> jni_sys::jboolean,
    |call: &Self| {
        assert_eq!(class1, call.class1);
        assert_eq!(class2, call.class2);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetSuperclass {
    pub class: jni_sys::jobject,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    GetSuperclass,
    fn(class: jni_sys::jobject) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(class, call.class);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct NewString {
    pub name: *const jni_sys::jchar,
    pub size: jni_sys::jsize,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    NewString,
    fn(name: *const jni_sys::jchar, size: jni_sys::jsize) -> jni_sys::jobject,
    |call: &Self| {
        assert_eq!(name, call.name);
        assert_eq!(size, call.size);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct NewStringUTF {
    pub string: String,
    pub result: jni_sys::jobject,
}

generate_method_check_impl!(
    NewStringUTF,
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
pub struct GetStringLength {
    pub string: jni_sys::jobject,
    pub result: jni_sys::jsize,
}

generate_method_check_impl!(
    GetStringLength,
    fn(string: jni_sys::jobject) -> jni_sys::jsize,
    |call: &Self| {
        assert_eq!(string, call.string);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetStringUTFLength {
    pub string: jni_sys::jobject,
    pub result: jni_sys::jsize,
}

generate_method_check_impl!(
    GetStringUTFLength,
    fn(string: jni_sys::jobject) -> jni_sys::jsize,
    |call: &Self| {
        assert_eq!(string, call.string);
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub struct GetMethodID {
    pub class: jni_sys::jclass,
    pub name: String,
    pub signature: String,
    pub result: jni_sys::jmethodID,
}

generate_method_check_impl!(
    GetMethodID,
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
pub struct GetStaticMethodID {
    pub class: jni_sys::jclass,
    pub name: String,
    pub signature: String,
    pub result: jni_sys::jmethodID,
}

generate_method_check_impl!(
    GetStaticMethodID,
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
pub struct GetStringUTFRegion {
    pub string: jni_sys::jstring,
    pub start: jni_sys::jsize,
    pub len: jni_sys::jsize,
    pub buffer: String,
}

generate_method_check_impl!(
    GetStringUTFRegion,
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
pub struct ExceptionDescribe {}

generate_method_check_impl!(ExceptionDescribe, fn() -> (), |_| ());

#[doc(hidden)]
#[derive(Debug)]
pub struct GetVersion {
    pub result: jni_sys::jint,
}

generate_method_check_impl!(GetVersion, fn() -> jni_sys::jint, |call: &Self| {
    call.result
});

#[doc(hidden)]
#[derive(Debug)]
pub struct GetJavaVM {
    pub vm: *mut jni_sys::JavaVM,
    pub result: jni_sys::jint,
}

generate_method_check_impl!(
    GetJavaVM,
    fn(vm: *mut *mut jni_sys::JavaVM) -> jni_sys::jint,
    |call: &Self| {
        assert_ne!(vm, ptr::null_mut());
        *vm = call.vm;
        call.result
    }
);

#[doc(hidden)]
#[derive(Debug)]
pub enum JniCall {
    DeleteLocalRef(DeleteLocalRef),
    NewLocalRef(NewLocalRef),
    ExceptionOccurred(ExceptionOccurred),
    ExceptionClear(ExceptionClear),
    ExceptionCheck(ExceptionCheck),
    IsSameObject(IsSameObject),
    GetObjectClass(GetObjectClass),
    IsInstanceOf(IsInstanceOf),
    Throw(Throw),
    ThrowNew(ThrowNew),
    FindClass(FindClass),
    DefineClass(DefineClass),
    IsAssignableFrom(IsAssignableFrom),
    GetSuperclass(GetSuperclass),
    NewString(NewString),
    NewStringUTF(NewStringUTF),
    GetStringLength(GetStringLength),
    GetStringUTFLength(GetStringUTFLength),
    GetMethodID(GetMethodID),
    GetStaticMethodID(GetStaticMethodID),
    GetStringUTFRegion(GetStringUTFRegion),
    ExceptionDescribe(ExceptionDescribe),
    GetJavaVM(GetJavaVM),
    GetVersion(GetVersion),
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

/// Make [`JniCalls`](struct.JniCalls.html) sendable between threads.
/// Safe, because this type is only used for testing.
unsafe impl Send for JniCalls {}
/// Make [`JniCalls`](struct.JniCalls.html) shareable by multiple threads.
/// Safe, because this type is only used for testing.
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

    /// Check that the object is what was expected.
    pub fn assert_eq<'env, T>(&self, object: &T, raw_object: jni_sys::jobject)
    where
        T: Cast<'env, Object<'env>>,
    {
        unsafe {
            assert_eq!(object.cast().raw_object(), raw_object);
            assert_eq!(object.cast().env().raw_env(), self.env);
        }
    }
}

#[doc(hidden)]
pub unsafe fn __to_static_ref<T>(reference: &'static T) -> &'static mut T {
    #[allow(mutable_transmutes)]
    mem::transmute::<_, &'static mut T>(reference)
}

#[macro_export]
macro_rules! test_raw_jni_env {
    ($calls:expr) => {
        test_raw_jni_env!($calls, empty_raw_jni_env())
    };
    ($calls:expr, $default_raw_env:expr) => {{
        lazy_static! {
            static ref CALLS: JniCalls = JniCalls::new($calls);
        }
        unsafe extern "system" fn delete_local_ref(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) {
            DeleteLocalRef::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn new_local_ref(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) -> ::jni_sys::jobject {
            NewLocalRef::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn exception_occured(
            env: *mut ::jni_sys::JNIEnv,
        ) -> ::jni_sys::jobject {
            ExceptionOccurred::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn exception_clear(env: *mut ::jni_sys::JNIEnv) {
            ExceptionClear::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn exception_check(
            env: *mut ::jni_sys::JNIEnv,
        ) -> ::jni_sys::jboolean {
            ExceptionCheck::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn is_same_object(
            env: *mut ::jni_sys::JNIEnv,
            object1: ::jni_sys::jobject,
            object2: ::jni_sys::jobject,
        ) -> ::jni_sys::jboolean {
            IsSameObject::__check_call(__to_static_ref(&CALLS), env, object1, object2)
        }
        unsafe extern "system" fn get_object_class(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) -> ::jni_sys::jobject {
            GetObjectClass::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn is_instance_of(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
            class: ::jni_sys::jobject,
        ) -> ::jni_sys::jboolean {
            IsInstanceOf::__check_call(__to_static_ref(&CALLS), env, object, class)
        }
        unsafe extern "system" fn throw(
            env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
        ) -> ::jni_sys::jint {
            Throw::__check_call(__to_static_ref(&CALLS), env, object)
        }
        unsafe extern "system" fn throw_new(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
            message: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jint {
            ThrowNew::__check_call(__to_static_ref(&CALLS), env, class, message)
        }
        unsafe extern "system" fn find_class(
            env: *mut ::jni_sys::JNIEnv,
            name: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jobject {
            FindClass::__check_call(__to_static_ref(&CALLS), env, name)
        }
        unsafe extern "system" fn define_class(
            env: *mut ::jni_sys::JNIEnv,
            name: *const ::std::os::raw::c_char,
            loader: ::jni_sys::jobject,
            buffer: *const ::jni_sys::jbyte,
            buffer_size: ::jni_sys::jsize,
        ) -> ::jni_sys::jobject {
            DefineClass::__check_call(
                __to_static_ref(&CALLS),
                env,
                name,
                loader,
                buffer,
                buffer_size,
            )
        }
        unsafe extern "system" fn is_assignable_from(
            env: *mut ::jni_sys::JNIEnv,
            class1: ::jni_sys::jobject,
            class2: ::jni_sys::jobject,
        ) -> ::jni_sys::jboolean {
            IsAssignableFrom::__check_call(__to_static_ref(&CALLS), env, class1, class2)
        }
        unsafe extern "system" fn get_superclass(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
        ) -> ::jni_sys::jobject {
            GetSuperclass::__check_call(__to_static_ref(&CALLS), env, class)
        }
        unsafe extern "system" fn new_string(
            env: *mut ::jni_sys::JNIEnv,
            name: *const ::jni_sys::jchar,
            size: ::jni_sys::jsize,
        ) -> ::jni_sys::jobject {
            NewString::__check_call(__to_static_ref(&CALLS), env, name, size)
        }
        unsafe extern "system" fn new_string_utf(
            env: *mut ::jni_sys::JNIEnv,
            string: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jobject {
            NewStringUTF::__check_call(__to_static_ref(&CALLS), env, string)
        }
        unsafe extern "system" fn get_string_length(
            env: *mut ::jni_sys::JNIEnv,
            string: ::jni_sys::jobject,
        ) -> ::jni_sys::jsize {
            GetStringLength::__check_call(__to_static_ref(&CALLS), env, string)
        }
        unsafe extern "system" fn get_string_utf_length(
            env: *mut ::jni_sys::JNIEnv,
            string: ::jni_sys::jobject,
        ) -> ::jni_sys::jsize {
            GetStringUTFLength::__check_call(__to_static_ref(&CALLS), env, string)
        }
        unsafe extern "system" fn get_method_id(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
            name: *const ::std::os::raw::c_char,
            signature: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jmethodID {
            GetMethodID::__check_call(__to_static_ref(&CALLS), env, class, name, signature)
        }
        unsafe extern "system" fn get_static_method_id(
            env: *mut ::jni_sys::JNIEnv,
            class: ::jni_sys::jobject,
            name: *const ::std::os::raw::c_char,
            signature: *const ::std::os::raw::c_char,
        ) -> ::jni_sys::jmethodID {
            GetStaticMethodID::__check_call(__to_static_ref(&CALLS), env, class, name, signature)
        }
        unsafe extern "system" fn get_string_utf_region(
            env: *mut ::jni_sys::JNIEnv,
            string: ::jni_sys::jobject,
            start: ::jni_sys::jsize,
            len: ::jni_sys::jsize,
            buffer: *mut ::std::os::raw::c_char,
        ) {
            GetStringUTFRegion::__check_call(
                __to_static_ref(&CALLS),
                env,
                string,
                start,
                len,
                buffer,
            )
        }
        unsafe extern "system" fn exception_describe(env: *mut ::jni_sys::JNIEnv) {
            ExceptionDescribe::__check_call(__to_static_ref(&CALLS), env)
        }
        unsafe extern "system" fn get_java_vm(
            env: *mut jni_sys::JNIEnv,
            vm: *mut *mut jni_sys::JavaVM,
        ) -> jni_sys::jint {
            GetJavaVM::__check_call(__to_static_ref(&CALLS), env, vm)
        }
        unsafe extern "system" fn get_version(env: *mut jni_sys::JNIEnv) -> jni_sys::jint {
            GetVersion::__check_call(__to_static_ref(&CALLS), env)
        }
        let raw_env = ::jni_sys::JNINativeInterface_ {
            GetJavaVM: Some(get_java_vm),
            GetVersion: Some(get_version),
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
            DefineClass: Some(define_class),
            Throw: Some(throw),
            ThrowNew: Some(throw_new),
            IsInstanceOf: Some(is_instance_of),
            DeleteLocalRef: Some(delete_local_ref),
            GetObjectClass: Some(get_object_class),
            NewLocalRef: Some(new_local_ref),
            ExceptionOccurred: Some(exception_occured),
            ExceptionClear: Some(exception_clear),
            ExceptionCheck: Some(exception_check),
            IsSameObject: Some(is_same_object),
            ..$default_raw_env
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
