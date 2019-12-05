use java::lang::Object;
use rust_jni::*;

pub struct ClassWithPrimitiveMethods<'a> {
    object: Object<'a>,
}

impl<'a> ClassWithPrimitiveMethods<'a> {
    pub fn new(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
    ) -> JavaResult<'a, ClassWithPrimitiveMethods<'a>> {
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

impl<'a> ::std::ops::Deref for ClassWithPrimitiveMethods<'a> {
    type Target = Object<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for ClassWithPrimitiveMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<ClassWithPrimitiveMethods<'a>> for ClassWithPrimitiveMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &ClassWithPrimitiveMethods<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for ClassWithPrimitiveMethods<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'a> FromObject<'a> for ClassWithPrimitiveMethods<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self { object }
    }
}

impl JniSignature for ClassWithPrimitiveMethods<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/ClassWithPrimitiveMethods;"
    }
}
