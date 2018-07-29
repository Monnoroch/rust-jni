#![recursion_limit = "1024"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
extern crate rust_jni;

mod generate;
mod java_name;
mod parse;
mod prepare;

use generate::*;
use java_name::*;
use parse::*;
use prepare::*;
use proc_macro2::*;

/// Generate `rust-jni` wrappers for Java classes and interfaces.
///
/// TODO(#76): examples.
#[proc_macro]
pub fn java_generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    java_generate_impl(input.into()).into()
}

fn java_generate_impl(input: TokenStream) -> TokenStream {
    generate(&to_generator_data(parse_java_definition(input)))
}

#[cfg(test)]
mod java_generate_tests {
    use super::*;

    #[test]
    fn empty() {
        let input = quote!{};
        let expected = quote!{};
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_class() {
        let input = quote!{
            class TestClass1 extends TestClass2 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            struct TestClass1<'env> {
                object: ::TestClass2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "LTestClass1;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::TestClass2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass1<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::TestClass2<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass1<'a> {
                type Target = ::TestClass2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "TestClass1", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }
            }

            impl<'a> ::std::fmt::Display for TestClass1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass1<'a> {}
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_class_implements() {
        let input = quote!{
            interface a.b.TestInterface1 {}
            interface a.b.TestInterface2 {}
            class TestClass1 extends TestClass2 implements a.b.TestInterface1, a.b.TestInterface2 {}
        };
        let expected = quote!{
            trait TestInterface1<'a> {
            }

            trait TestInterface2<'a> {
            }

            #[derive(Debug)]
            struct TestClass1<'env> {
                object: ::TestClass2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "LTestClass1;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::TestClass2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass1<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::TestClass2<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass1<'a> {
                type Target = ::TestClass2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "TestClass1", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }
            }

            impl<'a> ::std::fmt::Display for TestClass1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass1<'a> {}

            impl<'a> ::a::b::TestInterface1<'a> for TestClass1<'a> {
            }

            impl<'a> ::a::b::TestInterface2<'a> for TestClass1<'a> {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_class_packaged() {
        let input = quote!{
            class a.b.TestClass1 extends c.d.TestClass2 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            struct TestClass1<'env> {
                object: ::c::d::TestClass2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "La/b/TestClass1;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::c::d::TestClass2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass1<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::c::d::TestClass2<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::c::d::TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass1<'a> {
                type Target = ::c::d::TestClass2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "a/b/TestClass1", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }
            }

            impl<'a> ::std::fmt::Display for TestClass1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass1<'a> {}
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_class_public() {
        let input = quote!{
            public class TestClass1 extends TestClass2 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            pub struct TestClass1<'env> {
                object: ::TestClass2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "LTestClass1;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::TestClass2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass1<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::TestClass2<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass1<'a> {
                type Target = ::TestClass2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "TestClass1", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }
            }

            impl<'a> ::std::fmt::Display for TestClass1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass1<'a> {}
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_interface() {
        let input = quote!{
            interface TestInterface1 {}
        };
        let expected = quote!{
            trait TestInterface1<'a> {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_interface_packaged() {
        let input = quote!{
            interface a.b.TestInterface1 {}
        };
        let expected = quote!{
            trait TestInterface1<'a> {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_interface_public() {
        let input = quote!{
            public interface TestInterface1 {}
        };
        let expected = quote!{
            pub trait TestInterface1<'a> {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_interface_extends() {
        let input = quote!{
            interface TestInterface2 {}
            interface TestInterface3 {}
            interface TestInterface1 extends TestInterface2, TestInterface3 {}
        };
        let expected = quote!{
            trait TestInterface2<'a> {
            }

            trait TestInterface3<'a> {
            }

            trait TestInterface1<'a>: ::TestInterface2<'a> + ::TestInterface3<'a> {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn multiple() {
        let input = quote!{
            interface TestInterface1 {}
            interface TestInterface2 {}
            class TestClass1 {}
            class TestClass2 {}

            metadata {
                interface TestInterface3 {}
                class TestClass3;
            }
        };
        let expected = quote!{
            trait TestInterface1<'a> {
            }

            trait TestInterface2<'a> {
            }

            #[derive(Debug)]
            struct TestClass1<'env> {
                object: ::java::lang::Object<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "LTestClass1;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::java::lang::Object as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass1<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::java::lang::Object<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::java::lang::Object<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass1<'a> {
                type Target = ::java::lang::Object<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "TestClass1", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }
            }

            impl<'a> ::std::fmt::Display for TestClass1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass1<'a> {}

            #[derive(Debug)]
            struct TestClass2<'env> {
                object: ::java::lang::Object<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass2<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "LTestClass2;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass2<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass2<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::java::lang::Object as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass2<'a>> for TestClass2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::java::lang::Object<'a>> for TestClass2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::java::lang::Object<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass2<'a> {
                type Target = ::java::lang::Object<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass2<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "TestClass2", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }
            }

            impl<'a> ::std::fmt::Display for TestClass2<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass2<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass2<'a> {}
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn integration() {
        let input = quote!{
            public interface a.b.TestInterface3 {
                long primitiveInterfaceFunc3(int arg1, char arg2);
                a.b.TestClass3 objectInterfaceFunc3(a.b.TestClass3 arg);
            }

            public interface a.b.TestInterface4 extends c.d.TestInterface2, a.b.TestInterface3 {
                @RustName(primitive_func_3)
                long primitiveFunc3(int arg1, char arg2);
                @RustName(object_func_3)
                c.d.TestClass2 objectFunc3(a.b.TestClass3 arg);
            }

            public class a.b.TestClass3 extends c.d.TestClass2 implements e.f.TestInterface1, a.b.TestInterface4 {
                @RustName(init)
                public a.b.TestClass3(int arg1, a.b.TestClass3 arg2);

                @RustName(primitive_func_3)
                long primitiveFunc3(int arg1, char arg2);
                @RustName(object_func_3)
                public c.d.TestClass2 objectFunc3(a.b.TestClass3 arg);

                @RustName(primitive_static_func_3)
                static long primitiveStaticFunc3(int arg1, char arg2);
                @RustName(object_static_func_3)
                public static c.d.TestClass2 objectStaticFunc3(a.b.TestClass3 arg);

                @RustName(primitive_native_func_3)
                public native long primitiveNativeFunc3(int arg1, char arg2) {
                    println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, self);
                    Ok(0)
                };
                native a.b.TestClass3 objectNativeFunc3(a.b.TestClass3 arg) {
                    println!("{:?} {:?} {:?}", arg, token, self);
                    Ok(arg)
                };

                @RustName(primitive_static_native_func_3)
                static native long primitiveStaticNativeFunc3(int arg1, char arg2) {
                    println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, env);
                    Ok(0)
                };
                public static native a.b.TestClass3 objectStaticNativeFunc3(a.b.TestClass3 arg) {
                    println!("{:?} {:?} {:?}", arg, token, env);
                    Ok(arg)
                };

                long primitiveInterfaceFunc3(int arg1, char arg2);
                a.b.TestClass3 objectInterfaceFunc3(a.b.TestClass3 arg);
            }

            metadata {
                interface e.f.TestInterface1 {
                    @RustName(primitive_interface_func_1)
                    long primitiveInterfaceFunc1(int arg1, char arg2);
                }
                interface c.d.TestInterface2 extends e.f.TestInterface1 {}

                class c.d.TestClass1;
                class c.d.TestClass2 extends c.d.TestClass1 implements e.f.TestInterface1;
            }
        };
        let expected = quote!{
            pub trait TestInterface3<'a> {
                fn primitiveInterfaceFunc3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64>;

                fn objectInterfaceFunc3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> >;
            }

            pub trait TestInterface4<'a>: ::c::d::TestInterface2<'a> + ::a::b::TestInterface3<'a> {
                fn primitive_func_3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64>;

                fn object_func_3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::c::d::TestClass2<'a> >;
            }

            #[derive(Debug)]
            pub struct TestClass3<'env> {
                object: ::c::d::TestClass2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass3<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "La/b/TestClass3;"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for TestClass3<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for TestClass3<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <::c::d::TestClass2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass3<'a>> for TestClass3<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass3<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::c::d::TestClass2<'a>> for TestClass3<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::c::d::TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::c::d::TestClass1<'a>> for TestClass3<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::c::d::TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::java::lang::Object<'a>> for TestClass3<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::java::lang::Object<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass3<'a> {
                type Target = ::c::d::TestClass2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass3<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "a/b/TestClass3", token)
                }

                pub fn clone(&self, token: &::rust_jni::NoException<'a>) -> ::rust_jni::JavaResult<'a, Self>
                where
                    Self: Sized,
                {
                    self.object
                        .clone(token)
                        .map(|object| Self { object })
                }

                pub fn to_string(&self, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::String<'a>> {
                    self.object.to_string(token)
                }

                pub fn init(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: i32,
                    arg2: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, Self> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_constructor::<Self, _, fn(i32, &::a::b::TestClass3<'a>,)>
                        (
                            env,
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                fn primitive_func_3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn(i32, char,) -> i64
                        >
                        (
                            self,
                            "primitiveFunc3",
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                pub fn object_func_3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::c::d::TestClass2<'a> > {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn(&::a::b::TestClass3<'a>,) -> ::c::d::TestClass2<'a>
                        >
                        (
                            self,
                            "objectFunc3",
                            (arg,),
                            token,
                        )
                    }
                }

                fn primitiveInterfaceFunc3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn(i32, char,) -> i64
                        >
                        (
                            self,
                            "primitiveInterfaceFunc3",
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                fn objectInterfaceFunc3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> > {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn(&::a::b::TestClass3<'a>,) -> ::a::b::TestClass3<'a>
                        >
                        (
                            self,
                            "objectInterfaceFunc3",
                            (arg,),
                            token,
                        )
                    }
                }

                fn primitive_static_func_3(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_static_method::<Self, _, _,
                            fn(i32, char,) -> i64
                        >
                        (
                            env,
                            "primitiveStaticFunc3",
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                pub fn object_static_func_3(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::c::d::TestClass2<'a> > {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_static_method::<Self, _, _,
                            fn(&::a::b::TestClass3<'a>,) -> ::c::d::TestClass2<'a>
                        >
                        (
                            env,
                            "objectStaticFunc3",
                            (arg,),
                            token,
                        )
                    }
                }

                pub fn primitive_native_func_3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, self);
                    Ok(0)
                }

                fn objectNativeFunc3(
                    &self,
                    arg: ::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> > {
                    println!("{:?} {:?} {:?}", arg, token, self);
                    Ok(arg)
                }

                fn primitive_static_native_func_3(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, env);
                    Ok(0)
                }

                pub fn objectStaticNativeFunc3(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg: ::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> > {
                    println!("{:?} {:?} {:?}", arg, token, env);
                    Ok(arg)
                }
            }

            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn Java_a_b_TestClass3_primitiveNativeFunc3__IC<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                object: ::jni_sys::jobject,
                arg1: <i32 as ::rust_jni::JavaType>::__JniType,
                arg2: <char as ::rust_jni::JavaType>::__JniType,
            ) -> <i64 as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::test_jni_argument_type(arg1);
                ::rust_jni::__generator::test_jni_argument_type(arg2);
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    {
                        let value =
                            <i32 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg1);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }
                    {
                        let value =
                            <char as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg2);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }

                    let object = <TestClass3 as ::rust_jni::__generator::FromJni>::__from_jni(env, object);
                    object
                        .primitive_native_func_3(
                            ::rust_jni::__generator::FromJni::__from_jni(env, arg1),
                            ::rust_jni::__generator::FromJni::__from_jni(env, arg2),
                            &token,
                        )
                        .map(|value| {
                            let result = ::rust_jni::__generator::ToJni::__to_jni(&value);
                            // We don't want to delete the reference to result for object results.
                            ::std::mem::forget(value);
                            result
                        })
                })
            }

            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn Java_a_b_TestClass3_objectNativeFunc3__La_b_TestClass3_2<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                object: ::jni_sys::jobject,
                arg: <::a::b::TestClass3 as ::rust_jni::JavaType>::__JniType,
            ) -> <::a::b::TestClass3<'a> as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::test_jni_argument_type(arg);
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    {
                        let value =
                            <::a::b::TestClass3 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }

                    let object = <TestClass3 as ::rust_jni::__generator::FromJni>::__from_jni(env, object);
                    object
                        .objectNativeFunc3(
                            ::rust_jni::__generator::FromJni::__from_jni(env, arg),
                            &token,
                        )
                        .map(|value| {
                            let result = ::rust_jni::__generator::ToJni::__to_jni(&value);
                            // We don't want to delete the reference to result for object results.
                            ::std::mem::forget(value);
                            result
                        })
                })
            }

            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn Java_a_b_TestClass3_primitiveStaticNativeFunc3__IC<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                raw_class: ::jni_sys::jclass,
                arg1: <i32 as ::rust_jni::JavaType>::__JniType,
                arg2: <char as ::rust_jni::JavaType>::__JniType,
            ) -> <i64 as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::test_jni_argument_type(arg1);
                ::rust_jni::__generator::test_jni_argument_type(arg2);
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    {
                        let value =
                            <i32 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg1);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }
                    {
                        let value =
                            <char as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg2);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }

                    let class = TestClass3::get_class(env, &token)?;
                    let raw_class = <::rust_jni::java::lang::Class as ::rust_jni::__generator::FromJni>::__from_jni(env, raw_class);
                    if !class.is_same_as(&raw_class, &token) {
                        panic!("Native method primitiveStaticNativeFunc3 does not belong to class TestClass3");
                    }

                    TestClass3::primitive_static_native_func_3(
                        env,
                        ::rust_jni::__generator::FromJni::__from_jni(env, arg1),
                        ::rust_jni::__generator::FromJni::__from_jni(env, arg2),
                        &token,
                    )
                    .map(|value| {
                        let result = ::rust_jni::__generator::ToJni::__to_jni(&value);
                        ::std::mem::forget(value);
                        result
                    })
                })
            }

            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn Java_a_b_TestClass3_objectStaticNativeFunc3__La_b_TestClass3_2<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                raw_class: ::jni_sys::jclass,
                arg: <::a::b::TestClass3 as ::rust_jni::JavaType>::__JniType,
            ) -> <::a::b::TestClass3<'a> as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::test_jni_argument_type(arg);
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    {
                        let value =
                            <::a::b::TestClass3 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }

                    let class = TestClass3::get_class(env, &token)?;
                    let raw_class = <::rust_jni::java::lang::Class as ::rust_jni::__generator::FromJni>::__from_jni(env, raw_class);
                    if !class.is_same_as(&raw_class, &token) {
                        panic!("Native method objectStaticNativeFunc3 does not belong to class TestClass3");
                    }

                    TestClass3::objectStaticNativeFunc3(
                        env,
                        ::rust_jni::__generator::FromJni::__from_jni(env, arg),
                        &token,
                    )
                    .map(|value| {
                        let result = ::rust_jni::__generator::ToJni::__to_jni(&value);
                        ::std::mem::forget(value);
                        result
                    })
                })
            }

            impl<'a> ::std::fmt::Display for TestClass3<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for TestClass3<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for TestClass3<'a> {}


            impl<'a> ::a::b::TestInterface3<'a> for TestClass3<'a> {
                fn primitiveInterfaceFunc3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    Self::primitiveInterfaceFunc3(self, arg1, arg2, token)
                }

                fn objectInterfaceFunc3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> > {
                    Self::objectInterfaceFunc3(self, arg, token)
                }
            }

            impl<'a> ::a::b::TestInterface4<'a> for TestClass3<'a> {
                fn primitive_func_3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    Self::primitive_func_3(self, arg1, arg2, token)
                }

                fn object_func_3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::c::d::TestClass2<'a> > {
                    Self::object_func_3(self, arg, token)
                }
            }

            impl<'a> ::c::d::TestInterface2<'a> for TestClass3<'a> {
            }

            impl<'a> ::e::f::TestInterface1<'a> for TestClass3<'a> {
                fn primitive_interface_func_1(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    < ::c::d::TestClass2 as ::e::f::TestInterface1 >
                        ::primitive_interface_func_1(self, arg1, arg2, token)
                }
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }
}

#[cfg(test)]
fn assert_tokens_equals(left: TokenStream, right: TokenStream) {
    assert_eq!(format!("{:?}", left), format!("{:?}", right),);
}
