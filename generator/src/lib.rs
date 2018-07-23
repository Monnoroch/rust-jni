#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro2::*;
use quote::ToTokens;
use std::iter::FromIterator;
use std::ops::Deref;

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

#[derive(Debug, Clone)]
struct JavaName(TokenStream);

impl Deref for JavaName {
    type Target = TokenStream;

    fn deref(&self) -> &TokenStream {
        &self.0
    }
}

impl ToTokens for JavaName {
    fn to_tokens(&self, stream: &mut TokenStream) {
        self.0.to_tokens(stream)
    }
}

impl PartialEq for JavaName {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Eq for JavaName {}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
struct FlatMapThreaded<I, F, S> {
    iterator: I,
    function: F,
    state: S,
}

impl<I, F, S, T> Iterator for FlatMapThreaded<I, F, S>
where
    I: Iterator<Item = T>,
    F: FnMut(&T, &S) -> S,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.iterator.next() {
            None => None,
            Some(value) => {
                self.state = (self.function)(&value, &self.state);
                Some(value)
            }
        }
    }
}

fn flat_map_threaded<I, T, F, S>(iterator: I, initial: S, function: F) -> FlatMapThreaded<I, F, S>
where
    I: Iterator<Item = T>,
    F: FnMut(&T, &S) -> S,
{
    FlatMapThreaded {
        iterator,
        function,
        state: initial,
    }
}

impl JavaName {
    fn from_tokens<'a>(tokens: impl Iterator<Item = &'a TokenTree>) -> JavaName {
        let tokens = flat_map_threaded(tokens, false, |token, was_identifier| {
            match (token, was_identifier) {
                (TokenTree::Ident(_), false) => true,
                (TokenTree::Punct(punct), true) => {
                    if punct.as_char() != '.' {
                        panic!("Expected a dot, got {:?}.", punct);
                    }
                    false
                }
                (token, true) => {
                    panic!("Expected a dot, got {:?}.", token);
                }
                (token, false) => {
                    panic!("Expected an identifier, got {:?}.", token);
                }
            }
        }).filter(|token| match token {
            TokenTree::Ident(_) => true,
            _ => false,
        });
        let tokens = TokenStream::from_iter(tokens.cloned());
        if tokens.is_empty() {
            panic!("Expected a Java name, got no tokens.");
        }
        JavaName(tokens)
    }

    fn name(self) -> Ident {
        match self.0.into_iter().last().unwrap() {
            TokenTree::Ident(identifier) => identifier,
            token => panic!("Expected an identifier, got {:?}", token),
        }
    }

    fn with_double_colons(self) -> TokenStream {
        let mut tokens = vec![];
        let mut first = true;
        for token in self.0.into_iter() {
            if first {
                first = false;
            } else {
                tokens.extend(quote!{::});
            }
            tokens.push(token);
        }
        TokenStream::from_iter(tokens.iter().cloned())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinition {
    class: JavaName,
    public: bool,
    extends: JavaName,
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
            let (token, header) = header.split_first().unwrap();
            let public = is_identifier(&token, "public");
            let (token, header) = if public {
                header.split_first().unwrap()
            } else {
                (token, header)
            };
            if !is_identifier(&token, "class") {
                panic!("Expected \"class\", got {:?}.", token);
            }

            let class = JavaName::from_tokens(
                header
                    .iter()
                    .take_while(|token| !is_identifier(&token, "extends")),
            );
            let extends = JavaName::from_tokens(
                header
                    .iter()
                    .skip_while(|token| !is_identifier(&token, "extends"))
                    .skip(1),
            );
            JavaDefinition {
                class,
                public,
                extends,
            }
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
            class TestClass1 extends test1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    class: JavaName(quote!{TestClass1}),
                    public: false,
                    extends: JavaName(quote!{test1}),
                }],
            }
        );
    }

    #[test]
    fn one_public() {
        let input = quote!{
            public class TestClass1 extends test1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    class: JavaName(quote!{TestClass1}),
                    public: true,
                    extends: JavaName(quote!{test1}),
                }],
            }
        );
    }

    #[test]
    fn one_packaged() {
        let input = quote!{
            class a.b.TestClass1 extends c.d.test1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    class: JavaName(quote!{a b TestClass1}),
                    public: false,
                    extends: JavaName(quote!{c d test1}),
                }],
            }
        );
    }

    #[test]
    fn multiple() {
        let input = quote!{
            class TestClass1 extends test1 {}
            class TestClass2 extends test2 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        class: JavaName(quote!{TestClass1}),
                        public: false,
                        extends: JavaName(quote!{test1}),
                    },
                    JavaDefinition {
                        class: JavaName(quote!{TestClass2}),
                        public: false,
                        extends: JavaName(quote!{test2}),
                    },
                ],
            }
        );
    }

    #[test]
    #[should_panic(expected = "Expected \"class\"")]
    fn invalid_definition_kind() {
        let input = quote!{
            invalid 1
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected a Java name")]
    fn too_few_tokens() {
        let input = quote!{
            class
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected an identifier")]
    fn definition_name_not_identifier_after_dot() {
        let input = quote!{
            class a.1 {}
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected a dot")]
    fn definition_name_no_dot_after_identifier() {
        let input = quote!{
            class a b {}
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected a dot")]
    fn definition_name_not_dot_punctuation() {
        let input = quote!{
            class a,b {}
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected a Java name")]
    fn no_extends() {
        let input = quote!{
            class a {}
        };
        parse_java_definition(input);
    }
}

#[derive(Debug, Clone)]
struct GeneratorDefinition {
    class: Ident,
    class_public: TokenStream,
    super_class: TokenStream,
}

impl PartialEq for GeneratorDefinition {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Eq for GeneratorDefinition {}

#[derive(Debug, PartialEq, Eq, Clone)]
struct GeneratorData {
    definitions: Vec<GeneratorDefinition>,
}

fn to_generator_data(definitions: JavaDefinitions) -> GeneratorData {
    GeneratorData {
        definitions: definitions
            .definitions
            .into_iter()
            .map(|definition| {
                let JavaDefinition {
                    class,
                    public,
                    extends,
                    ..
                } = definition;
                let class = class.name();
                let class_public = if public {
                    quote!{pub}
                } else {
                    TokenStream::new()
                };
                let super_class = extends.with_double_colons();
                GeneratorDefinition {
                    class,
                    class_public,
                    super_class,
                }
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
                    class: JavaName(quote!{a b test1}),
                    public: false,
                    extends: JavaName(quote!{c d test2}),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    class_public: TokenStream::new(),
                    super_class: quote!{::c::d::test2},
                }],
            }
        );
    }

    #[test]
    fn one_public() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    class: JavaName(quote!{a b test1}),
                    public: true,
                    extends: JavaName(quote!{c d test2}),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    class_public: quote!{pub},
                    super_class: quote!{::c::d::test2},
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
                        class: JavaName(quote!{a b test1}),
                        public: false,
                        extends: JavaName(quote!{c d test3}),
                    },
                    JavaDefinition {
                        class: JavaName(quote!{test2}),
                        public: false,
                        extends: JavaName(quote!{c d test4}),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        class_public: TokenStream::new(),
                        super_class: quote!{::c::d::test3},
                    },
                    GeneratorDefinition {
                        class: Ident::new("test2", Span::call_site()),
                        class_public: TokenStream::new(),
                        super_class: quote!{::c::d::test4},
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
    let GeneratorDefinition {
        class,
        class_public,
        super_class,
        ..
    } = definition;
    quote! {
        #[derive(Debug)]
        #class_public struct #class<'env> {
            object: #super_class<'env>,
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
                class_public: quote!{test_public},
                super_class: quote!{::c::d::test2},
            }],
        };
        let expected = quote!{
            #[derive(Debug)]
            test_public struct test1<'env> {
                object: c::d::test2<'env>,
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
                    class_public: TokenStream::new(),
                    super_class: quote!{::c::d::test3},
                },
                GeneratorDefinition {
                    class: Ident::new("test2", Span::call_site()),
                    class_public: TokenStream::new(),
                    super_class: quote!{::c::d::test4},
                },
            ],
        };
        let expected = quote!{
            #[derive(Debug)]
            struct test1<'env> {
                object: c::d::test3<'env>,
            }

            #[derive(Debug)]
            struct test2<'env> {
                object: c::d::test4<'env>,
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
            class a.b.TestClass1 extends c.d.TestClass2 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            struct TestClass1<'env> {
                object: c::d::TestClass2<'env>,
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn one_public() {
        let input = quote!{
            public class TestClass1 extends TestClass2 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            pub struct TestClass1<'env> {
                object: TestClass2<'env>,
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn multiple() {
        let input = quote!{
            class TestClass1 extends TestClass3 {}
            class TestClass2 extends TestClass4 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            struct TestClass1<'env> {
                object: TestClass3<'env>,
            }

            #[derive(Debug)]
            struct TestClass2<'env> {
                object: TestClass4<'env>,
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn integration() {
        let input = quote!{
            class a.b.TestClass1 extends java.lang.Object {}
            public class TestClass2 extends a.b.TestClass1 {}
        };
        let expected = quote!{
            #[derive(Debug)]
            struct TestClass1<'env> {
                object: java::lang::Object<'env>,
            }

            #[derive(Debug)]
            struct TestClass2<'env> {
                object: a::b::TestClass1<'env>,
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }
}

#[cfg(test)]
fn assert_tokens_equals(left: TokenStream, right: TokenStream) {
    assert_eq!(format!("{:?}", left), format!("{:?}", right),);
}
