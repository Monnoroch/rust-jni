use crate::simple_class::SimpleClass;
use java::lang::Object;
use rust_jni::*;

pub struct SimpleSubClass<'a> {
    object: SimpleClass<'a>,
}

impl<'a> SimpleSubClass<'a> {
    pub fn new(token: &NoException<'a>, value: i32) -> JavaResult<'a, SimpleSubClass<'a>> {
        unsafe { call_constructor::<Self, _, fn(i32)>(token, (value,)) }
    }
}

impl<'a> ::std::ops::Deref for SimpleSubClass<'a> {
    type Target = SimpleClass<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SimpleSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SimpleSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleSubClass<'a>> for SimpleSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleSubClass<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SimpleSubClass<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'a> FromObject<'a> for SimpleSubClass<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self {
            object: SimpleClass::from_object(object),
        }
    }
}

impl JniSignature for SimpleSubClass<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SimpleSubClass;"
    }
}
