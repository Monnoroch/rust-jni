use crate::simple_class::SimpleClass;
use java::lang::Object;
use rust_jni::*;

pub struct SubClassWithMethodOverride<'a> {
    object: SimpleClass<'a>,
}

impl<'a> SubClassWithMethodOverride<'a> {
    pub fn new(
        token: &NoException<'a>,
        value: i32,
    ) -> JavaResult<'a, SubClassWithMethodOverride<'a>> {
        unsafe { call_constructor::<Self, _, fn(i32)>(token, (value,)) }
    }
}

impl<'a> ::std::ops::Deref for SubClassWithMethodOverride<'a> {
    type Target = SimpleClass<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SubClassWithMethodOverride<'a>> for SubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SubClassWithMethodOverride<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SubClassWithMethodOverride<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'a> FromObject<'a> for SubClassWithMethodOverride<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self {
            object: SimpleClass::from_object(object),
        }
    }
}

impl JniSignature for SubClassWithMethodOverride<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SubClassWithMethodOverride;"
    }
}
