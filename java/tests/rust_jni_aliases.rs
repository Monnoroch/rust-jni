#![allow(dead_code)]
extern crate java;
extern crate rust_jni;

#[cfg(test)]
mod rust_jni_aliases {

    // These functions would not compile if the types would not be aliases of `rust-jni`s types.

    fn test_object<'a>(value: ::rust_jni::java::lang::Object<'a>) -> ::java::lang::Object<'a> {
        value
    }

    fn test_throwable<'a>(
        value: ::rust_jni::java::lang::Throwable<'a>,
    ) -> ::java::lang::Throwable<'a> {
        value
    }

    fn test_class<'a>(value: ::rust_jni::java::lang::Class<'a>) -> ::java::lang::Class<'a> {
        value
    }

    fn test_string<'a>(value: ::rust_jni::java::lang::String<'a>) -> ::java::lang::String<'a> {
        value
    }

    #[test]
    fn test() {}
}
