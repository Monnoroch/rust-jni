#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro2::*;

/// Generate `rust-jni` wrappers for Java classes and interfaces.
///
/// TODO(#76): examples.
#[proc_macro]
pub fn java_generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    java_generate_impl(input).into()
}

fn java_generate_impl(input: TokenStream) -> TokenStream {
    generate(to_generator_data(parse_java_definition(input)))
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinitions {}

fn parse_java_definition(_input: TokenStream) -> JavaDefinitions {
    JavaDefinitions {}
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn empty() {
        let input = quote!{};
        assert_eq!(parse_java_definition(input), JavaDefinitions {});
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct GeneratorData {}

fn to_generator_data(_definitions: JavaDefinitions) -> GeneratorData {
    GeneratorData {}
}

#[cfg(test)]
mod to_generator_data_tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(to_generator_data(JavaDefinitions {}), GeneratorData {});
    }
}

fn generate(_data: GeneratorData) -> TokenStream {
    TokenStream::new()
}

#[cfg(test)]
mod generate_tests {
    use super::*;

    #[test]
    fn empty() {
        let expected = quote!{};
        assert_tokens_equals(generate(GeneratorData {}), expected);
    }
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
}

#[cfg(test)]
fn assert_tokens_equals(left: TokenStream, right: TokenStream) {
    assert_eq!(format!("{:?}", left), format!("{:?}", right),);
}
