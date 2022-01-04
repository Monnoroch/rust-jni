use crate::simple_class::SimpleClass;
use java::lang::Object;
use rust_jni::*;

pub struct ClassWithObjectMethods<'a> {
    object: Object<'a>,
}

impl<'a> ClassWithObjectMethods<'a> {
    pub fn new(token: &NoException<'a>) -> JavaResult<'a, ClassWithObjectMethods<'a>> {
        unsafe { Self::call_constructor::<_, fn()>(token, ()) }
    }

    pub fn test_function_object(
        &self,
        token: &NoException<'a>,
        argument: impl JavaObjectArgument<SimpleClass<'a>>,
    ) -> JavaResult<'a, Option<SimpleClass<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            self.call_method::<_, fn(&SimpleClass) -> SimpleClass<'a>>(
                token,
                "testFunction\0",
                (argument.as_argument(),),
            )
        }
    }

    pub fn test_static_function_object(
        token: &NoException<'a>,
        argument: impl JavaObjectArgument<SimpleClass<'a>>,
    ) -> JavaResult<'a, Option<SimpleClass<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            Self::call_static_method::<_, fn(&SimpleClass) -> SimpleClass<'a>>(
                token,
                "testStaticFunction\0",
                (argument.as_argument(),),
            )
        }
    }
}

impl<'a> ::std::ops::Deref for ClassWithObjectMethods<'a> {
    type Target = Object<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for ClassWithObjectMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<ClassWithObjectMethods<'a>> for ClassWithObjectMethods<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &ClassWithObjectMethods<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for ClassWithObjectMethods<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'a> FromObject<'a> for ClassWithObjectMethods<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self { object }
    }
}

impl JavaClassSignature for ClassWithObjectMethods<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/ClassWithObjectMethods;"
    }
}
