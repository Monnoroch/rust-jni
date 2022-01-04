use crate::simple_class::SimpleClass;
use crate::sub_class_with_method_alias::SubClassWithMethodAlias;
use java::lang::Object;
use rust_jni::*;

pub struct SubSubClassWithMethodAlias<'a> {
    object: SubClassWithMethodAlias<'a>,
}

impl<'a> SubSubClassWithMethodAlias<'a> {
    pub fn new(
        token: &NoException<'a>,
        value: i32,
    ) -> JavaResult<'a, SubSubClassWithMethodAlias<'a>> {
        unsafe { Self::call_constructor::<_, fn(i32)>(token, (value,)) }
    }

    pub fn combine(
        &self,
        token: &NoException<'a>,
        other: impl JavaObjectArgument<SubSubClassWithMethodAlias<'a>>,
    ) -> JavaResult<'a, Option<SubSubClassWithMethodAlias<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            self.call_method::<_, fn(&SubSubClassWithMethodAlias) -> SubSubClassWithMethodAlias<'a>>(
                token,
                "combine\0",
                (other.as_argument(),),
            )
        }
    }
}

impl<'a> ::std::ops::Deref for SubSubClassWithMethodAlias<'a> {
    type Target = SubClassWithMethodAlias<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SubSubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SubSubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SubClassWithMethodAlias<'a>> for SubSubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SubClassWithMethodAlias<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SubSubClassWithMethodAlias<'a>> for SubSubClassWithMethodAlias<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SubSubClassWithMethodAlias<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SubSubClassWithMethodAlias<'a> {
    fn into(self) -> Object<'a> {
        self.object.into()
    }
}

impl<'a> FromObject<'a> for SubSubClassWithMethodAlias<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self {
            object: SubClassWithMethodAlias::from_object(object),
        }
    }
}

impl JavaClassSignature for SubSubClassWithMethodAlias<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SubSubClassWithMethodAlias;"
    }
}
