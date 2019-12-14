use crate::simple_class::SimpleClass;
use java::lang::Object;
use jni_sys;
use rust_jni::*;

pub struct ClassWithObjectNativeMethods<'a> {
    object: Object<'a>,
}

impl<'a> ClassWithObjectNativeMethods<'a> {
    pub fn new(token: &NoException<'a>) -> JavaResult<'a, ClassWithObjectNativeMethods<'a>> {
        unsafe { call_constructor::<Self, _, fn()>(token, ()) }
    }

    pub fn test_function_object(
        &self,
        token: &NoException<'a>,
        argument: impl JavaObjectArgument<'a, SimpleClass<'a>>,
    ) -> JavaResult<'a, Option<SimpleClass<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(Option<&SimpleClass<'a>>) -> SimpleClass<'a>>(
                self,
                token,
                "testFunction\0",
                (argument.as_argument(),),
            )
        }
    }

    pub fn test_static_function_object(
        token: &NoException<'a>,
        argument: impl JavaObjectArgument<'a, SimpleClass<'a>>,
    ) -> JavaResult<'a, Option<SimpleClass<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_static_method::<Self, _, _, fn(Option<&SimpleClass<'a>>) -> SimpleClass<'a>>(
                token,
                "testStaticFunction\0",
                (argument.as_argument(),),
            )
        }
    }
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithObjectNativeMethods_testNativeFunction__Lrustjni_test_SimpleClass_2(
    raw_env: *mut jni_sys::JNIEnv,
    raw_object: jni_sys::jobject,
    argument: jni_sys::jobject,
) {
    native_method_implementation::<(SimpleClass,), SimpleClass, _>(
        raw_env,
        raw_object,
        (argument,),
        |_object, token, (argument,)| {
            (
                Ok(Box::new(argument.as_ref().or_npe(&token).unwrap())),
                token,
            )
        },
    );
}

#[no_mangle]
unsafe extern "C" fn Java_rustjni_test_ClassWithObjectNativeMethods_testStaticNativeFunction__Lrustjni_test_SimpleClass_2(
    raw_env: *mut jni_sys::JNIEnv,
    raw_class: jni_sys::jclass,
    argument: jni_sys::jobject,
) {
    static_native_method_implementation::<(SimpleClass,), SimpleClass, _>(
        raw_env,
        raw_class,
        (argument,),
        |_class, token, (argument,)| {
            (
                Ok(Box::new(argument.as_ref().or_npe(&token).unwrap())),
                token,
            )
        },
    );
}

impl<'a> ::std::ops::Deref for ClassWithObjectNativeMethods<'a> {
    type Target = Object<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for ClassWithObjectNativeMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<ClassWithObjectNativeMethods<'a>> for ClassWithObjectNativeMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &ClassWithObjectNativeMethods<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for ClassWithObjectNativeMethods<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'a> FromObject<'a> for ClassWithObjectNativeMethods<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self { object }
    }
}

impl JniSignature for ClassWithObjectNativeMethods<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/ClassWithObjectNativeMethods;"
    }
}
