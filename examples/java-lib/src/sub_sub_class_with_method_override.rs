use crate::simple_class::SimpleClass;
use crate::sub_class_with_method_override::SubClassWithMethodOverride;
use java::lang::Object;
use rust_jni::*;

pub struct SubSubClassWithMethodOverride<'a> {
    object: SubClassWithMethodOverride<'a>,
}

impl<'a> SubSubClassWithMethodOverride<'a> {
    pub fn new(
        env: &'a JniEnv<'a>,
        token: &NoException<'a>,
        value: i32,
    ) -> JavaResult<'a, SubSubClassWithMethodOverride<'a>> {
        unsafe { call_constructor::<Self, _, fn(i32)>(env, token, (value,)) }
    }
}

impl<'a> ::std::ops::Deref for SubSubClassWithMethodOverride<'a> {
    type Target = SubClassWithMethodOverride<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SubSubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SubSubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SubClassWithMethodOverride<'a>> for SubSubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SubClassWithMethodOverride<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SubSubClassWithMethodOverride<'a>> for SubSubClassWithMethodOverride<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SubSubClassWithMethodOverride<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SubSubClassWithMethodOverride<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'a> FromObject<'a> for SubSubClassWithMethodOverride<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self {
            object: SubClassWithMethodOverride::from_object(object),
        }
    }
}

impl JniSignature for SubSubClassWithMethodOverride<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SubSubClassWithMethodOverride;"
    }
}
