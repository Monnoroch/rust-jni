#[cfg(test)]
mod rust_jni_aliases {

    // These functions would not compile if the types would not be aliases of `rust-jni`s types.

    #[allow(dead_code)]
    fn test_object<'a>(value: ::rust_jni::java::lang::Object<'a>) -> ::java::lang::Object<'a> {
        value
    }

    #[allow(dead_code)]
    fn test_throwable<'a>(
        value: ::rust_jni::java::lang::Throwable<'a>,
    ) -> ::java::lang::Throwable<'a> {
        value
    }

    #[allow(dead_code)]
    fn test_class<'a>(value: ::rust_jni::java::lang::Class<'a>) -> ::java::lang::Class<'a> {
        value
    }

    #[allow(dead_code)]
    fn test_string<'a>(value: ::rust_jni::java::lang::String<'a>) -> ::java::lang::String<'a> {
        value
    }

    #[allow(dead_code)]
    fn test_exception<'a>(
        value: ::rust_jni::java::lang::Exception<'a>,
    ) -> ::java::lang::Exception<'a> {
        value
    }

    #[allow(dead_code)]
    fn test_null_pointer_exception<'a>(
        value: ::rust_jni::java::lang::NullPointerException<'a>,
    ) -> ::java::lang::NullPointerException<'a> {
        value
    }

    #[test]
    fn test() {}
}
