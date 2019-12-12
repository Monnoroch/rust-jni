use java::lang::Object;
use rust_jni::*;

pub struct SimpleClass<'a> {
    object: Object<'a>,
}

impl<'a> SimpleClass<'a> {
    pub fn new(token: &NoException<'a>, value: i32) -> JavaResult<'a, SimpleClass<'a>> {
        unsafe { call_constructor::<Self, _, fn(i32)>(token, (value,)) }
    }

    pub fn value_with_added(&self, token: &NoException<'a>, to_add: i32) -> JavaResult<'a, i32> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(i32) -> i32>(self, token, "valueWithAdded\0", (to_add,))
        }
    }

    pub fn combine(
        &self,
        token: &NoException<'a>,
        other: impl JavaObjectArgument<'a, SimpleClass<'a>>,
    ) -> JavaResult<'a, Option<SimpleClass<'a>>> {
        // Safe because we ensure correct arguments and return type.
        unsafe {
            call_method::<Self, _, _, fn(Option<&SimpleClass<'a>>) -> SimpleClass<'a>>(
                self,
                token,
                "combine\0",
                (other.as_argument(),),
            )
        }
    }
}

impl<'a> ::std::ops::Deref for SimpleClass<'a> {
    type Target = Object<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<'a> AsRef<Object<'a>> for SimpleClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &Object<'a> {
        self.object.as_ref()
    }
}

impl<'a> AsRef<SimpleClass<'a>> for SimpleClass<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &SimpleClass<'a> {
        self
    }
}

impl<'a> Into<Object<'a>> for SimpleClass<'a> {
    fn into(self) -> Object<'a> {
        self.object
    }
}

impl<'a> FromObject<'a> for SimpleClass<'a> {
    #[inline(always)]
    unsafe fn from_object(object: Object<'a>) -> Self {
        Self { object }
    }
}

impl JniSignature for SimpleClass<'_> {
    #[inline(always)]
    fn signature() -> &'static str {
        "Lrustjni/test/SimpleClass;"
    }
}
