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
struct JavaDefinition {
    class: Ident,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinitions {
    definitions: Vec<JavaDefinition>,
}

fn parse_java_definition(input: TokenStream) -> JavaDefinitions {
    let definitions = input.clone().into_iter().collect::<Vec<_>>();
    let definitions = definitions
        .split(is_definition)
        .filter(|tokens| !tokens.is_empty())
        .map(|header| {
            if header.len() < 2 {
                panic!("Expected a class definition, got {:?}.", header);
            }

            let (token, header) = header.split_first().unwrap();
            if !is_identifier(&token, "class") {
                panic!("Expected \"class\", got {:?}.", token);
            }

            let (class, _) = header.split_first().unwrap();
            let class = match class.clone() {
                TokenTree::Ident(identifier) => identifier,
                token => panic!("Expected an identifier, got {:?}.", token),
            };
            JavaDefinition { class }
        })
        .collect();
    JavaDefinitions { definitions }
}

fn is_identifier(token: &TokenTree, name: &str) -> bool {
    match token {
        TokenTree::Ident(identifier) => identifier == name,
        _ => false,
    }
}

fn is_definition(token: &TokenTree) -> bool {
    match token {
        TokenTree::Group(group) => group.delimiter() == Delimiter::Brace,
        _ => false,
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn empty() {
        let input = quote!{};
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![],
            }
        );
    }

    #[test]
    fn one() {
        let input = quote!{
            class TestClass1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    class: Ident::new("TestClass1", Span::call_site()),
                }],
            }
        );
    }

    #[test]
    fn multiple() {
        let input = quote!{
            class TestClass1 {}
            class TestClass2 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        class: Ident::new("TestClass1", Span::call_site()),
                    },
                    JavaDefinition {
                        class: Ident::new("TestClass2", Span::call_site()),
                    },
                ],
            }
        );
    }

    #[test]
    #[should_panic(expected = "Expected \"class\"")]
    fn invalid_definition_kind() {
        let input = quote!{
            class test {}
            invalid 1
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected a class definition")]
    fn too_few_tokens() {
        let input = quote!{
            class test {}
            invalid
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected an identifier")]
    fn definition_name_not_identifier() {
        let input = quote!{
            class 1 {}
        };
        parse_java_definition(input);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct GeneratorDefinition {
    class: Ident,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct GeneratorData {
    definitions: Vec<GeneratorDefinition>,
}

fn to_generator_data(definitions: JavaDefinitions) -> GeneratorData {
    GeneratorData {
        definitions: definitions
            .definitions
            .into_iter()
            .map(|definition| GeneratorDefinition {
                class: definition.class,
            })
            .collect(),
    }
}

#[cfg(test)]
mod to_generator_data_tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![],
            }),
            GeneratorData {
                definitions: vec![],
            }
        );
    }

    #[test]
    fn one() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    class: Ident::new("test1", Span::call_site()),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                }],
            }
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        class: Ident::new("test1", Span::call_site()),
                    },
                    JavaDefinition {
                        class: Ident::new("test2", Span::call_site()),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                    },
                    GeneratorDefinition {
                        class: Ident::new("test2", Span::call_site()),
                    },
                ],
            }
        );
    }
}

fn generate(data: GeneratorData) -> TokenStream {
    let mut tokens = TokenStream::new();
    for definition in data.definitions {
        tokens.extend(generate_definition(definition));
    }
    tokens
}

fn generate_definition(definition: GeneratorDefinition) -> TokenStream {
    let GeneratorDefinition { class, .. } = definition;
    quote! {
        #[derive(Debug)]
        pub struct #class {
        }
    }
}

#[cfg(test)]
mod generate_tests {
    use super::*;

    #[test]
    fn empty() {
        let input = GeneratorData {
            definitions: vec![],
        };
        let expected = quote!{};
        assert_tokens_equals(generate(input), expected);
    }

    #[test]
    fn one() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition {
                class: Ident::new("test1", Span::call_site()),
            }],
        };
        let expected = quote!{
            #[derive(Debug)]
            pub struct test1 {
            }
        };
        assert_tokens_equals(generate(input), expected);
    }

    #[test]
    fn multiple() {
        let input = GeneratorData {
            definitions: vec![
                GeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                },
                GeneratorDefinition {
                    class: Ident::new("test2", Span::call_site()),
                },
            ],
        };
        let expected = quote!{
            #[derive(Debug)]
            pub struct test1 {
            }

            #[derive(Debug)]
            pub struct test2 {
            }
        };
        assert_tokens_equals(generate(input), expected);
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

    #[test]
    fn one() {
        let input = quote!{
            class TestClass1 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            pub struct TestClass1 {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn multiple() {
        let input = quote!{
            class TestClass1 {}
            class TestClass2 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            pub struct TestClass1 {
            }

            #[derive(Debug)]
            pub struct TestClass2 {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }
}

#[cfg(test)]
fn assert_tokens_equals(left: TokenStream, right: TokenStream) {
    assert_eq!(format!("{:?}", left), format!("{:?}", right),);
}
