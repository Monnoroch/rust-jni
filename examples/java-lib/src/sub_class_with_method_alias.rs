use crate::simple_class::SimpleClass;
use java::lang::Object;
use rust_jni::*;

pub struct SubClassWithMethodAlias<'a> {
    object: SimpleClass<'a>,
}

impl<'a> SubClassWithMethodAlias<'a> {
    pub fn new(token: &NoException<'a>, value: i32) -> JavaResult<'a, SubClassWithMethodAlias<'a>> {
        unsafe { Self::call_constructor::<_, fn(i32)>(token, (value,)) }
    }

    pub fn combine(
        &self,
        token: &NoException<'a>,
        other: impl JavaObjectArgument<SubClassWithMethodAlias<'a>>,
    ) -> JavaResult<'a, Option<SubClassWithMethodAlias<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            self.call_method::<_, fn(&SubClassWithMethodAlias) -> SubClassWithMethodAlias<'a>>(
                token,
                "combine\0",
                (other.as_argument(),),
            )
        }
    }
}

impl<'a> ::std::ops::Deref for SubClassWithMethodAlias<'a> {
    type Target = SimpleClass<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SubClassWithMethodAlias<'a>> for SubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SubClassWithMethodAlias<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SubClassWithMethodAlias<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'a> FromObject<'a> for SubClassWithMethodAlias<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self {
            object: SimpleClass::from_object(object),
        }
    }
}

impl JavaClassSignature for SubClassWithMethodAlias<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SubClassWithMethodAlias;"
    }
}
