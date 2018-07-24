#![recursion_limit = "1024"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro2::*;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::iter::{self, FromIterator};
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

impl Hash for JavaName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
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

    fn with_slashes(self) -> String {
        self.0
            .into_iter()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
            .join("/")
    }

    fn with_double_colons(self) -> TokenStream {
        let mut tokens = vec![];
        for token in self.0.into_iter() {
            tokens.extend(quote!{::});
            tokens.push(token);
        }
        TokenStream::from_iter(tokens.iter().cloned())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaClass {
    extends: JavaName,
    implements: Vec<JavaName>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaInterface {
    extends: Vec<JavaName>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum JavaDefinitionKind {
    Class(JavaClass),
    Interface(JavaInterface),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinition {
    name: JavaName,
    public: bool,
    definition: JavaDefinitionKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinitions {
    definitions: Vec<JavaDefinition>,
}

fn comma_separated_names(tokens: impl Iterator<Item = TokenTree>) -> Vec<JavaName> {
    let tokens = tokens.collect::<Vec<_>>();
    tokens
        .split(|token| match token {
            TokenTree::Punct(punct) => punct.spacing() == Spacing::Alone && punct.as_char() == ',',
            _ => false,
        })
        .filter(|slice| slice.len() > 0)
        .map(|slice| JavaName::from_tokens(slice.iter()))
        .collect()
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
            let is_class = is_identifier(&token, "class");
            let is_interface = is_identifier(&token, "interface");
            if !is_class && !is_interface {
                panic!("Expected \"class\" or \"interface\", got {:?}.", token);
            }

            if is_interface {
                let name = JavaName::from_tokens(
                    header
                        .iter()
                        .take_while(|token| !is_identifier(&token, "extends")),
                );
                let extends = comma_separated_names(
                    header
                        .iter()
                        .skip_while(|token| !is_identifier(&token, "extends"))
                        .skip(1)
                        .cloned(),
                );
                JavaDefinition {
                    name,
                    public,
                    definition: JavaDefinitionKind::Interface(JavaInterface { extends }),
                }
            } else {
                let name = JavaName::from_tokens(
                    header
                        .iter()
                        .take_while(|token| !is_identifier(&token, "extends")),
                );
                let extends = JavaName::from_tokens(
                    header
                        .iter()
                        .skip_while(|token| !is_identifier(&token, "extends"))
                        .skip(1)
                        .take_while(|token| !is_identifier(&token, "implements")),
                );
                let implements = comma_separated_names(
                    header
                        .iter()
                        .skip_while(|token| !is_identifier(&token, "implements"))
                        .skip(1)
                        .cloned(),
                );
                JavaDefinition {
                    name,
                    public,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends,
                        implements,
                    }),
                }
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
    fn one_class() {
        let input = quote!{
            class TestClass1 extends test1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestClass1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: JavaName(quote!{test1}),
                        implements: vec![],
                    }),
                }],
            }
        );
    }

    #[test]
    fn one_class_public() {
        let input = quote!{
            public class TestClass1 extends test1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestClass1}),
                    public: true,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: JavaName(quote!{test1}),
                        implements: vec![],
                    }),
                }],
            }
        );
    }

    #[test]
    fn one_class_packaged() {
        let input = quote!{
            class a.b.TestClass1 extends c.d.test1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b TestClass1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: JavaName(quote!{c d test1}),
                        implements: vec![],
                    }),
                }],
            }
        );
    }

    #[test]
    fn one_class_implements() {
        let input = quote!{
            class TestClass1 extends test1 implements test2, a.b.test3 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestClass1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: JavaName(quote!{test1}),
                        implements: vec![JavaName(quote!{test2}), JavaName(quote!{a b test3})],
                    }),
                }],
            }
        );
    }

    #[test]
    fn one_interface() {
        let input = quote!{
            interface TestInterface1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestInterface1}),
                    public: false,
                    definition: JavaDefinitionKind::Interface(JavaInterface { extends: vec![] }),
                }],
            }
        );
    }

    #[test]
    fn one_interface_public() {
        let input = quote!{
            public interface TestInterface1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestInterface1}),
                    public: true,
                    definition: JavaDefinitionKind::Interface(JavaInterface { extends: vec![] }),
                }],
            }
        );
    }

    #[test]
    fn one_interface_packaged() {
        let input = quote!{
            interface a.b.TestInterface1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b TestInterface1}),
                    public: false,
                    definition: JavaDefinitionKind::Interface(JavaInterface { extends: vec![] }),
                }],
            }
        );
    }

    #[test]
    fn one_interface_extends() {
        let input = quote!{
            interface TestInterface1 extends TestInterface2, a.b.TestInterface3 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestInterface1}),
                    public: false,
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        extends: vec![
                            JavaName(quote!{TestInterface2}),
                            JavaName(quote!{a b TestInterface3}),
                        ],
                    }),
                }],
            }
        );
    }

    #[test]
    fn multiple() {
        let input = quote!{
            interface TestInterface1 {}
            interface TestInterface2 extends TestInterface3 {}
            class TestClass1 extends test1 {}
            class TestClass2 extends test2 implements test3 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{TestInterface1}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{TestInterface2}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![JavaName(quote!{TestInterface3})],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{TestClass1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{test1}),
                            implements: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{TestClass2}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{test2}),
                            implements: vec![JavaName(quote!{test3})],
                        }),
                    },
                ],
            }
        );
    }

    #[test]
    #[should_panic(expected = "Expected \"class\" or \"interface\"")]
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
struct ClassGeneratorDefinition {
    class: Ident,
    public: TokenStream,
    super_class: TokenStream,
    implements: Vec<TokenStream>,
    signature: Literal,
    full_signature: Literal,
}

#[derive(Debug, Clone)]
struct InterfaceGeneratorDefinition {
    interface: Ident,
    public: TokenStream,
    extends: Vec<TokenStream>,
}

#[derive(Debug, Clone)]
enum GeneratorDefinition {
    Class(ClassGeneratorDefinition),
    Interface(InterfaceGeneratorDefinition),
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

fn populate_interface_extends_rec(
    interface_extends: &mut HashMap<JavaName, HashSet<JavaName>>,
    key: &JavaName,
) {
    let mut interfaces = interface_extends.get(key).unwrap().clone();
    // TODO: this will break in case of cycles.
    for interface in interfaces.iter() {
        populate_interface_extends_rec(interface_extends, interface)
    }
    for interface in interfaces.clone().iter() {
        interfaces.extend(interface_extends.get(interface).unwrap().iter().cloned());
    }
    *interface_extends.get_mut(key).unwrap() = interfaces;
}

fn populate_interface_extends(interface_extends: &mut HashMap<JavaName, HashSet<JavaName>>) {
    for key in interface_extends.keys().cloned().collect::<Vec<_>>() {
        populate_interface_extends_rec(interface_extends, &key);
    }
}

fn to_generator_data(definitions: JavaDefinitions) -> GeneratorData {
    let mut interface_extends = HashMap::new();
    definitions
        .definitions
        .clone()
        .into_iter()
        .filter(|definition| match definition.definition {
            JavaDefinitionKind::Interface(_) => true,
            _ => false,
        })
        .for_each(|definition| {
            let JavaDefinition {
                name, definition, ..
            } = definition;
            match definition {
                JavaDefinitionKind::Interface(interface) => {
                    let JavaInterface { extends, .. } = interface;
                    let all_extends = interface_extends.entry(name).or_insert(HashSet::new());
                    extends.into_iter().for_each(|extends_name| {
                        all_extends.insert(extends_name);
                    });
                }
                _ => unreachable!(),
            }
        });
    populate_interface_extends(&mut interface_extends);
    GeneratorData {
        definitions: definitions
            .definitions
            .into_iter()
            .map(|definition| {
                let JavaDefinition {
                    name,
                    public,
                    definition,
                    ..
                } = definition;
                let definition_name = name.clone().name();
                let public = if public {
                    quote!{pub}
                } else {
                    TokenStream::new()
                };
                match definition {
                    JavaDefinitionKind::Class(class) => {
                        let string_signature = name.with_slashes();
                        let signature = Literal::string(&string_signature);
                        let full_signature = Literal::string(&format!("L{};", string_signature));
                        let JavaClass {
                            extends,
                            implements,
                            ..
                        } = class;
                        let super_class = extends.with_double_colons();
                        let implements = implements
                            .iter()
                            .flat_map(|name| interface_extends.get(&name).unwrap().iter())
                            .chain(implements.iter())
                            .cloned()
                            .collect::<HashSet<_>>();
                        let mut implements = implements
                            .into_iter()
                            .map(|name| name.with_double_colons())
                            .collect::<Vec<_>>();
                        implements.sort_by(|left, right| left.to_string().cmp(&right.to_string()));
                        GeneratorDefinition::Class(ClassGeneratorDefinition {
                            class: definition_name,
                            public,
                            super_class,
                            implements,
                            signature,
                            full_signature,
                        })
                    }
                    JavaDefinitionKind::Interface(interface) => {
                        let JavaInterface { extends, .. } = interface;
                        GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                            interface: definition_name,
                            public,
                            extends: extends
                                .into_iter()
                                .map(|name| name.with_double_colons())
                                .collect(),
                        })
                    }
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
    fn one_class() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b test1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: JavaName(quote!{c d test2}),
                        implements: vec![],
                    }),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{::c::d::test2},
                    implements: vec![],
                    signature: Literal::string("a/b/test1"),
                    full_signature: Literal::string("La/b/test1;"),
                })],
            }
        );
    }

    #[test]
    fn one_class_implements() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test4}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{c d test2}),
                            implements: vec![
                                JavaName(quote!{e f test3}),
                                JavaName(quote!{e f test4}),
                            ],
                        }),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test4", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::c::d::test2},
                        implements: vec![quote!{::e::f::test3}, quote!{::e::f::test4}],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                    }),
                ],
            }
        );
    }

    #[test]
    fn one_class_implements_recursive() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{g h test5}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test4}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![JavaName(quote!{g h test5})],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![JavaName(quote!{e f test4})],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{c d test2}),
                            implements: vec![JavaName(quote!{e f test3})],
                        }),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test5", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test4", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::g::h::test5}],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::e::f::test4}],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::c::d::test2},
                        implements: vec![
                            quote!{::e::f::test3},
                            quote!{::e::f::test4},
                            quote!{::g::h::test5},
                        ],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                    }),
                ],
            }
        );
    }

    #[test]
    fn one_class_implements_recursive_duplicated() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{g h test4}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![JavaName(quote!{g h test4})],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{c d test2}),
                            implements: vec![
                                JavaName(quote!{e f test3}),
                                JavaName(quote!{g h test4}),
                            ],
                        }),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test4", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::g::h::test4}],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::c::d::test2},
                        implements: vec![quote!{::e::f::test3}, quote!{::g::h::test4}],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                    }),
                ],
            }
        );
    }

    #[test]
    fn one_class_public() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b test1}),
                    public: true,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: JavaName(quote!{c d test2}),
                        implements: vec![],
                    }),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: quote!{pub},
                    super_class: quote!{::c::d::test2},
                    implements: vec![],
                    signature: Literal::string("a/b/test1"),
                    full_signature: Literal::string("La/b/test1;"),
                })],
            }
        );
    }

    #[test]
    fn one_interface() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b test1}),
                    public: false,
                    definition: JavaDefinitionKind::Interface(JavaInterface { extends: vec![] }),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Interface(
                    InterfaceGeneratorDefinition {
                        interface: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    },
                )],
            }
        );
    }

    #[test]
    fn one_interface_extends() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{c d test2}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![JavaName(quote!{c d test2}), JavaName(quote!{e f test3})],
                        }),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test2", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::c::d::test2}, quote!{::e::f::test3}],
                    }),
                ],
            }
        );
    }

    #[test]
    fn one_interface_public() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b test1}),
                    public: true,
                    definition: JavaDefinitionKind::Interface(JavaInterface { extends: vec![] }),
                }],
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Interface(
                    InterfaceGeneratorDefinition {
                        interface: Ident::new("test1", Span::call_site()),
                        public: quote!{pub},
                        extends: vec![],
                    },
                )],
            }
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{e f test_if1}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test_if2}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{c d test3}),
                            implements: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{test2}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: JavaName(quote!{c d test4}),
                            implements: vec![],
                        }),
                    },
                ],
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test_if1", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test_if2", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::c::d::test3},
                        implements: vec![],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test2", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::c::d::test4},
                        implements: vec![],
                        signature: Literal::string("test2"),
                        full_signature: Literal::string("Ltest2;"),
                    }),
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
    match definition {
        GeneratorDefinition::Class(class) => generate_class_definition(class),
        GeneratorDefinition::Interface(interface) => generate_interface_definition(interface),
    }
}

fn generate_class_definition(definition: ClassGeneratorDefinition) -> TokenStream {
    let ClassGeneratorDefinition {
        class,
        public,
        super_class,
        implements,
        signature,
        full_signature,
        ..
    } = definition;
    let multiplied_class = iter::repeat(class.clone());
    quote! {
        #[derive(Debug)]
        #public struct #class<'env> {
            object: #super_class<'env>,
        }

        impl<'a> ::rust_jni::JavaType for #class<'a> {
            #[doc(hidden)]
            type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

            #[doc(hidden)]
            fn __signature() -> &'static str {
                #full_signature
            }
        }

        impl<'a> ::rust_jni::__generator::ToJni for #class<'a> {
            unsafe fn __to_jni(&self) -> Self::__JniType {
                self.raw_object()
            }
        }

        impl<'a> ::rust_jni::__generator::FromJni<'a> for #class<'a> {
            unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                Self {
                    object: <#super_class as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                }
            }
        }

        impl<'a> ::rust_jni::Cast<'a, #class<'a>> for #class<'a> {
            #[doc(hidden)]
            fn cast<'b>(&'b self) -> &'b #class<'a> {
                self
            }
        }

        impl<'a> ::rust_jni::Cast<'a, #super_class<'a>> for #class<'a> {
            #[doc(hidden)]
            fn cast<'b>(&'b self) -> &'b #super_class<'a> {
                self
            }
        }

        impl<'a> ::std::ops::Deref for #class<'a> {
            type Target = #super_class<'a>;

            fn deref(&self) -> &Self::Target {
                &self.object
            }
        }

        impl<'a> #class<'a> {
            pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                ::rust_jni::java::lang::Class::find(env, #signature, token)
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

        impl<'a> ::std::fmt::Display for #class<'a> {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                self.object.fmt(formatter)
            }
        }

        impl<'a, T> PartialEq<T> for #class<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
            fn eq(&self, other: &T) -> bool {
                self.object.eq(other)
            }
        }

        impl<'a> Eq for #class<'a> {}

        #(
            impl<'env> #implements for #multiplied_class<'env> {
            }
        )*
    }
}

fn generate_interface_definition(definition: InterfaceGeneratorDefinition) -> TokenStream {
    let InterfaceGeneratorDefinition {
        interface,
        public,
        extends,
        ..
    } = definition;
    let extends = if extends.is_empty() {
        TokenStream::new()
    } else {
        quote!{: #(#extends)+*}
    };
    quote! {
        #public trait #interface #extends {
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
    fn one_class() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                class: Ident::new("test1", Span::call_site()),
                public: quote!{test_public},
                super_class: quote!{c::d::test2},
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
            })],
        };
        let expected = quote!{
            #[derive(Debug)]
            test_public struct test1<'env> {
                object: c::d::test2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for test1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "test/signature1"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for test1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for test1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <c::d::test2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, test1<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b test1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, c::d::test2<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b c::d::test2<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for test1<'a> {
                type Target = c::d::test2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> test1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "test/sign1", token)
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

            impl<'a> ::std::fmt::Display for test1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for test1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for test1<'a> {}
        };
        assert_tokens_equals(generate(input), expected);
    }

    #[test]
    fn one_class_implements() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                class: Ident::new("test1", Span::call_site()),
                public: quote!{test_public},
                super_class: quote!{c::d::test2},
                implements: vec![quote!{e::f::test3}, quote!{e::f::test4}],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
            })],
        };
        let expected = quote!{
            #[derive(Debug)]
            test_public struct test1<'env> {
                object: c::d::test2<'env>,
            }

            impl<'a> ::rust_jni::JavaType for test1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "test/signature1"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for test1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for test1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <c::d::test2 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, test1<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b test1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, c::d::test2<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b c::d::test2<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for test1<'a> {
                type Target = c::d::test2<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> test1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "test/sign1", token)
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

            impl<'a> ::std::fmt::Display for test1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for test1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for test1<'a> {}

            impl<'env> e::f::test3 for test1<'env> {
            }

            impl<'env> e::f::test4 for test1<'env> {
            }
        };
        assert_tokens_equals(generate(input), expected);
    }

    #[test]
    fn one_interface() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Interface(
                InterfaceGeneratorDefinition {
                    interface: Ident::new("test1", Span::call_site()),
                    public: quote!{test_public},
                    extends: vec![],
                },
            )],
        };
        let expected = quote!{
            test_public trait test1 {
            }
        };
        assert_tokens_equals(generate(input), expected);
    }

    #[test]
    fn one_interface_extends() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Interface(
                InterfaceGeneratorDefinition {
                    interface: Ident::new("test1", Span::call_site()),
                    public: TokenStream::new(),
                    extends: vec![quote!{c::d::test2}, quote!{e::f::test3}],
                },
            )],
        };
        let expected = quote!{
            trait test1 : c::d::test2 + e::f::test3 {
            }
        };
        assert_tokens_equals(generate(input), expected);
    }

    #[test]
    fn multiple() {
        let input = GeneratorData {
            definitions: vec![
                GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                    interface: Ident::new("test_if1", Span::call_site()),
                    public: TokenStream::new(),
                    extends: vec![],
                }),
                GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                    interface: Ident::new("test_if2", Span::call_site()),
                    public: TokenStream::new(),
                    extends: vec![],
                }),
                GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{c::d::test3},
                    implements: vec![],
                    signature: Literal::string("test/sign1"),
                    full_signature: Literal::string("test/signature1"),
                }),
                GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test2", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{c::d::test4},
                    implements: vec![],
                    signature: Literal::string("test/sign2"),
                    full_signature: Literal::string("test/signature2"),
                }),
            ],
        };
        let expected = quote!{
            trait test_if1 {
            }

            trait test_if2 {
            }

            #[derive(Debug)]
            struct test1<'env> {
                object: c::d::test3<'env>,
            }

            impl<'a> ::rust_jni::JavaType for test1<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "test/signature1"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for test1<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for test1<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <c::d::test3 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, test1<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b test1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, c::d::test3<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b c::d::test3<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for test1<'a> {
                type Target = c::d::test3<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> test1<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "test/sign1", token)
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

            impl<'a> ::std::fmt::Display for test1<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for test1<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for test1<'a> {}

            #[derive(Debug)]
            struct test2<'env> {
                object: c::d::test4<'env>,
            }

            impl<'a> ::rust_jni::JavaType for test2<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "test/signature2"
                }
            }

            impl<'a> ::rust_jni::__generator::ToJni for test2<'a> {
                unsafe fn __to_jni(&self) -> Self::__JniType {
                    self.raw_object()
                }
            }

            impl<'a> ::rust_jni::__generator::FromJni<'a> for test2<'a> {
                unsafe fn __from_jni(env: &'a ::rust_jni::JniEnv<'a>, value: Self::__JniType) -> Self {
                    Self {
                        object: <c::d::test4 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, test2<'a>> for test2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b test2<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, c::d::test4<'a>> for test2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b c::d::test4<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for test2<'a> {
                type Target = c::d::test4<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> test2<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "test/sign2", token)
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

            impl<'a> ::std::fmt::Display for test2<'a> {
                fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    self.object.fmt(formatter)
                }
            }

            impl<'a, T> PartialEq<T> for test2<'a> where T: ::rust_jni::Cast<'a, ::rust_jni::java::lang::Object<'a>> {
                fn eq(&self, other: &T) -> bool {
                    self.object.eq(other)
                }
            }

            impl<'a> Eq for test2<'a> {}
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
            trait TestInterface1 {
            }

            trait TestInterface2 {
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

            impl<'env> ::a::b::TestInterface1 for TestClass1<'env> {
            }

            impl<'env> ::a::b::TestInterface2 for TestClass1<'env> {
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
            trait TestInterface1 {
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
            trait TestInterface1 {
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
            pub trait TestInterface1 {
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
            trait TestInterface2 {
            }

            trait TestInterface3 {
            }

            trait TestInterface1: ::TestInterface2 + ::TestInterface3 {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }

    #[test]
    fn multiple() {
        let input = quote!{
            interface TestInterface1 {}
            interface TestInterface2 {}
            class TestClass1 extends TestClass3 {}
            class TestClass2 extends TestClass4 {}
        };
        let expected = quote!{
            trait TestInterface1 {
            }

            trait TestInterface2 {
            }

            #[derive(Debug)]
            struct TestClass1<'env> {
                object: ::TestClass3<'env>,
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
                        object: <::TestClass3 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass1<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::TestClass3<'a>> for TestClass1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::TestClass3<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass1<'a> {
                type Target = ::TestClass3<'a>;

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
                object: ::TestClass4<'env>,
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
                        object: <::TestClass4 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass2<'a>> for TestClass2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::TestClass4<'a>> for TestClass2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::TestClass4<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass2<'a> {
                type Target = ::TestClass4<'a>;

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
            public interface a.b.TestInterface1 {}
            public interface a.b.TestInterface2 extends a.b.TestInterface1 {}
            public interface a.b.TestInterface3 {}
            public interface a.b.TestInterface4 extends a.b.TestInterface2, a.b.TestInterface3 {}

            public class a.b.TestClass1 extends java.lang.Object {}
            public class a.b.TestClass2 extends a.b.TestClass1 implements a.b.TestInterface4, a.b.TestInterface3 {}
        };
        let expected = quote!{
            pub trait TestInterface1 {
            }

            pub trait TestInterface2: ::a::b::TestInterface1 {
            }

            pub trait TestInterface3 {
            }

            pub trait TestInterface4: ::a::b::TestInterface2 + ::a::b::TestInterface3 {
            }

            #[derive(Debug)]
            pub struct TestClass1<'env> {
                object: ::java::lang::Object<'env>,
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

            #[derive(Debug)]
            pub struct TestClass2<'env> {
                object: ::a::b::TestClass1<'env>,
            }

            impl<'a> ::rust_jni::JavaType for TestClass2<'a> {
                #[doc(hidden)]
                type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

                #[doc(hidden)]
                fn __signature() -> &'static str {
                    "La/b/TestClass2;"
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
                        object: <::a::b::TestClass1 as ::rust_jni::__generator::FromJni<'a>>::__from_jni(env, value),
                    }
                }
            }

            impl<'a> ::rust_jni::Cast<'a, TestClass2<'a>> for TestClass2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b TestClass2<'a> {
                    self
                }
            }

            impl<'a> ::rust_jni::Cast<'a, ::a::b::TestClass1<'a>> for TestClass2<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b ::a::b::TestClass1<'a> {
                    self
                }
            }

            impl<'a> ::std::ops::Deref for TestClass2<'a> {
                type Target = ::a::b::TestClass1<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.object
                }
            }

            impl<'a> TestClass2<'a> {
                pub fn get_class(env: &'a ::rust_jni::JniEnv<'a>, token: &::rust_jni::NoException<'a>)
                    -> ::rust_jni::JavaResult<'a, ::rust_jni::java::lang::Class<'a>> {
                    ::rust_jni::java::lang::Class::find(env, "a/b/TestClass2", token)
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

            impl<'env> ::a::b::TestInterface1 for TestClass2<'env> {
            }

            impl<'env> ::a::b::TestInterface2 for TestClass2<'env> {
            }

            impl<'env> ::a::b::TestInterface3 for TestClass2<'env> {
            }

            impl<'env> ::a::b::TestInterface4 for TestClass2<'env> {
            }
        };
        assert_tokens_equals(java_generate_impl(input), expected);
    }
}

#[cfg(test)]
fn assert_tokens_equals(left: TokenStream, right: TokenStream) {
    assert_eq!(format!("{:?}", left), format!("{:?}", right),);
}
