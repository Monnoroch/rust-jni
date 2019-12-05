use java::lang::Object;
use jni_sys;
use rust_jni::*;

pub struct ClassWithPrimitiveNativeMethods<'a> {
    object: Object<'a>,
}

impl<'a> ClassWithPrimitiveNativeMethods<'a> {
    pub fn new(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
    ) -> JavaResult<'a, ClassWithPrimitiveNativeMethods<'a>> {
        unsafe { call_constructor::<Self, _, fn()>(env, token, ()) }
    }

    pub fn test_function_void(&self, token: &NoException<'a>) -> JavaResult<'a, ()> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_method::<Self, _, _, fn()>(self, token, "testFunction\0", ()) }
    }

    pub fn test_function_bool(
        &self,
        token: &NoException<'a>,
        argument: bool,
    ) -> JavaResult<'a, bool> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(bool) -> bool>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_function_char(
        &self,
        token: &NoException<'a>,
        argument: char,
    ) -> JavaResult<'a, char> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(char) -> char>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_function_u8(&self, token: &NoException<'a>, argument: u8) -> JavaResult<'a, u8> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(u8) -> u8>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_function_i16(&self, token: &NoException<'a>, argument: i16) -> JavaResult<'a, i16> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(i16) -> i16>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_function_i32(&self, token: &NoException<'a>, argument: i32) -> JavaResult<'a, i32> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(i32) -> i32>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_function_i64(&self, token: &NoException<'a>, argument: i64) -> JavaResult<'a, i64> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(i64) -> i64>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_function_f32(
        &self,
        token: &NoException<'a>,
        // TODO(#25): floating point numbers don't work properly.
        argument: f64,
    ) -> JavaResult<'a, f32> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            // TODO(#25): floating point numbers don't work properly.
            call_method::<Self, _, _, fn(f64) -> f32>(
                self,
                token,
                "testFloatFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_function_f64(&self, token: &NoException<'a>, argument: f64) -> JavaResult<'a, f64> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(f64) -> f64>(self, token, "testFunction\0", (argument,))
        }
    }

    pub fn test_static_function_void(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
    ) -> JavaResult<'a, ()> {
        // Safe because we ensure correct arguments and return type.
        unsafe { call_static_method::<Self, _, _, fn()>(env, token, "testStaticFunction\0", ()) }
    }

    pub fn test_static_function_bool(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: bool,
    ) -> JavaResult<'a, bool> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(bool) -> bool>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_char(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: char,
    ) -> JavaResult<'a, char> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(char) -> char>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_u8(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: u8,
    ) -> JavaResult<'a, u8> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(u8) -> u8>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_i16(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: i16,
    ) -> JavaResult<'a, i16> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(i16) -> i16>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_i32(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: i32,
    ) -> JavaResult<'a, i32> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(i32) -> i32>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_i64(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: i64,
    ) -> JavaResult<'a, i64> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(i64) -> i64>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_f32(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        // TODO(#25): floating point numbers don't work properly.
        argument: f64,
    ) -> JavaResult<'a, f32> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            // TODO(#25): floating point numbers don't work properly.
            call_static_method::<Self, _, _, fn(f64) -> f32>(
                env,
                token,
                "testStaticFloatFunction\0",
                (argument,),
            )
        }
    }

    pub fn test_static_function_f64(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        argument: f64,
    ) -> JavaResult<'a, f64> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(f64) -> f64>(
                env,
                token,
                "testStaticFunction\0",
                (argument,),
            )
        }
    }
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
) {
    native_method_implementation::<(), (), _>(raw_env, raw_object, (), |_object, token, _| {
        (Ok(Box::new(())), token)
    });
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__Z(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jboolean,
) {
    native_method_implementation::<(bool,), bool, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(!argument)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__C(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jchar,
) {
    native_method_implementation::<(char,), char, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new((*argument as u8 + 1) as char)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__B(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jbyte,
) {
    native_method_implementation::<(u8,), u8, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(argument + 2)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__S(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jshort,
) {
    native_method_implementation::<(i16,), i16, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(argument + 3)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__I(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jint,
) {
    native_method_implementation::<(i32,), i32, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(argument + 4)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__J(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jlong,
) {
    native_method_implementation::<(i64,), i64, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(argument + 5)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__F(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jfloat,
) {
    native_method_implementation::<(f32,), f32, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(argument + 6.)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testNativeFunction__D(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jdouble,
) {
    native_method_implementation::<(f64,), f64, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| (Ok(Box::new(argument + 7.)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
) {
    static_native_method_implementation::<(), (), _>(raw_env, raw_class, (), |_class, token, _| {
        (Ok(Box::new(())), token)
    });
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__Z(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jboolean,
) {
    static_native_method_implementation::<(bool,), bool, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(!argument)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__C(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jchar,
) {
    static_native_method_implementation::<(char,), char, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new((*argument as u8 + 1) as char)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__B(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jbyte,
) {
    static_native_method_implementation::<(u8,), u8, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(argument + 2)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__S(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jshort,
) {
    static_native_method_implementation::<(i16,), i16, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(argument + 3)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__I(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jint,
) {
    static_native_method_implementation::<(i32,), i32, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(argument + 4)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__J(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jlong,
) {
    static_native_method_implementation::<(i64,), i64, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(argument + 5)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__F(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jfloat,
) {
    static_native_method_implementation::<(f32,), f32, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(argument + 6.)), token),
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithPrimitiveNativeMethods_testStaticNativeFunction__D(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jdouble,
) {
    static_native_method_implementation::<(f64,), f64, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| (Ok(Box::new(argument + 7.)), token),
    );
}

impl<'a> ::std::ops::Deref for ClassWithPrimitiveNativeMethods<'a> {
    type Target = Object<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for ClassWithPrimitiveNativeMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<ClassWithPrimitiveNativeMethods<'a>> for ClassWithPrimitiveNativeMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &ClassWithPrimitiveNativeMethods<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for ClassWithPrimitiveNativeMethods<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'a> FromObject<'a> for ClassWithPrimitiveNativeMethods<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self { object }
    }
}

impl JniSignature for ClassWithPrimitiveNativeMethods<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/ClassWithPrimitiveNativeMethods;"
    }
}
