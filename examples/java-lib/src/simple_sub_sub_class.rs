use crate::simple_class::SimpleClass;
use crate::simple_sub_class::SimpleSubClass;
use java::lang::Object;
use rust_jni::*;

pub struct SimpleSubSubClass<'a> {
    object: SimpleSubClass<'a>,
}

impl<'a> SimpleSubSubClass<'a> {
    pub fn new(token: &NoException<'a>, value: i32) -> JavaResult<'a, SimpleSubSubClass<'a>> {
        unsafe { call_constructor::<Self, _, fn(i32)>(token, (value,)) }
    }
}

impl<'a> ::std::ops::Deref for SimpleSubSubClass<'a> {
    type Target = SimpleSubClass<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SimpleSubSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SimpleSubSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleSubClass<'a>> for SimpleSubSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleSubClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleSubSubClass<'a>> for SimpleSubSubClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleSubSubClass<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SimpleSubSubClass<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'a> FromObject<'a> for SimpleSubSubClass<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self {
            object: SimpleSubClass::from_object(object),
        }
    }
}

impl JniSignature for SimpleSubSubClass<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SimpleSubSubClass;"
    }
}
