use super::*;
use proc_macro2::*;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: Ident,
    pub value: TokenStream,
}

impl PartialEq for Annotation {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Eq for Annotation {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MethodArgument {
    pub name: Ident,
    pub data_type: JavaName,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaInterfaceMethod {
    pub name: Ident,
    pub return_type: JavaName,
    pub arguments: Vec<MethodArgument>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaClassMethod {
    pub name: Ident,
    pub return_type: JavaName,
    pub arguments: Vec<MethodArgument>,
    pub public: bool,
    pub is_static: bool,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone)]
pub struct JavaNativeMethod {
    pub name: Ident,
    pub return_type: JavaName,
    pub arguments: Vec<MethodArgument>,
    pub public: bool,
    pub is_static: bool,
    pub code: Group,
    pub annotations: Vec<Annotation>,
}

impl PartialEq for JavaNativeMethod {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Eq for JavaNativeMethod {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaConstructor {
    pub arguments: Vec<MethodArgument>,
    pub public: bool,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaClass {
    pub extends: Option<JavaName>,
    pub implements: Vec<JavaName>,
    pub methods: Vec<JavaClassMethod>,
    pub native_methods: Vec<JavaNativeMethod>,
    pub constructors: Vec<JavaConstructor>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaInterface {
    pub methods: Vec<JavaInterfaceMethod>,
    pub extends: Vec<JavaName>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JavaDefinitionKind {
    Class(JavaClass),
    Interface(JavaInterface),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaDefinition {
    pub name: JavaName,
    pub public: bool,
    pub definition: JavaDefinitionKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaClassMetadata {
    pub extends: Option<JavaName>,
    pub implements: Vec<JavaName>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaInterfaceMetadata {
    pub extends: Vec<JavaName>,
    pub methods: Vec<JavaInterfaceMethod>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JavaDefinitionMetadataKind {
    Class(JavaClassMetadata),
    Interface(JavaInterfaceMetadata),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaDefinitionMetadata {
    pub name: JavaName,
    pub definition: JavaDefinitionMetadataKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub definitions: Vec<JavaDefinitionMetadata>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JavaDefinitions {
    pub definitions: Vec<JavaDefinition>,
    pub metadata: Metadata,
}

fn parse_annotation(tokens: &[TokenTree]) -> Annotation {
    let name = match tokens[0] {
        TokenTree::Ident(ref identifier) => identifier.clone(),
        _ => unreachable!(),
    };
    let value = match tokens[1] {
        TokenTree::Group(ref group) => group.stream(),
        _ => unreachable!(),
    };
    Annotation {
        name,
        value: TokenStream::from_iter(value.into_iter()),
    }
}

fn parse_annotations(tokens: &[TokenTree]) -> Vec<Annotation> {
    if tokens.len() == 0 {
        vec![]
    } else {
        match tokens[0] {
            TokenTree::Punct(ref punct) => {
                if punct.spacing() == Spacing::Alone && punct.as_char() == '@' {
                    tokens
                        .split(|token| is_punctuation(token, '@'))
                        .filter(|slice| !slice.is_empty())
                        .map(parse_annotation)
                        .collect()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

fn comma_separated_names(tokens: impl Iterator<Item = TokenTree>) -> Vec<JavaName> {
    let tokens = tokens.collect::<Vec<_>>();
    tokens
        .split(|token| is_punctuation(token, ','))
        .filter(|slice| !slice.is_empty())
        .map(|slice| JavaName::from_tokens(slice.iter()))
        .collect()
}

fn is_punctuation(token: &TokenTree, value: char) -> bool {
    match token {
        TokenTree::Punct(punct) => punct.spacing() == Spacing::Alone && punct.as_char() == value,
        _ => false,
    }
}

fn parse_interface_header(header: &[TokenTree]) -> (JavaName, Vec<JavaName>) {
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
    (name, extends)
}

fn parse_class_header(header: &[TokenTree]) -> (JavaName, Option<JavaName>, Vec<JavaName>) {
    let name = JavaName::from_tokens(header.iter().take_while(|token| {
        !is_identifier(&token, "extends") && !is_identifier(&token, "implements")
    }));
    let implements = comma_separated_names(
        header
            .iter()
            .skip_while(|token| !is_identifier(&token, "implements"))
            .skip(1)
            .cloned(),
    );
    let has_extends = header
        .iter()
        .filter(|token| is_identifier(&token, "extends"))
        .next()
        .is_some();
    let extends = if has_extends {
        Some(JavaName::from_tokens(
            header
                .iter()
                .skip_while(|token| !is_identifier(&token, "extends"))
                .skip(1)
                .take_while(|token| !is_identifier(&token, "implements")),
        ))
    } else {
        None
    };
    (name, extends, implements)
}

fn parse_metadata(tokens: TokenStream) -> Metadata {
    let definitions = tokens.clone().into_iter().collect::<Vec<_>>();
    let definitions = definitions
        .split(is_metadata_definition)
        .filter(|tokens| !tokens.is_empty())
        .map(|header| {
            let (token, header) = header.split_first().unwrap();
            let is_class = is_identifier(&token, "class");
            let is_interface = is_identifier(&token, "interface");
            if !is_class && !is_interface {
                panic!("Expected \"class\" or \"interface\", got {:?}.", token);
            }

            if is_interface {
                let (name, extends) = parse_interface_header(header);
                JavaDefinitionMetadata {
                    name,
                    definition: JavaDefinitionMetadataKind::Interface(JavaInterfaceMetadata {
                        extends,
                        methods: vec![],
                    }),
                }
            } else {
                let (name, extends, implements) = parse_class_header(header);
                JavaDefinitionMetadata {
                    name,
                    definition: JavaDefinitionMetadataKind::Class(JavaClassMetadata {
                        extends,
                        implements,
                    }),
                }
            }
        })
        .zip(definitions.iter().cloned().filter(is_metadata_definition))
        .map(|(definition, token)| match token {
            TokenTree::Group(group) => (definition, group.stream()),
            TokenTree::Punct(_) => (definition, TokenStream::new()),
            _ => unreachable!(),
        })
        .map(|(definition, tokens)| {
            let java_definition = match definition.definition.clone() {
                JavaDefinitionMetadataKind::Interface(interface) => {
                    let methods = tokens.into_iter().collect::<Vec<_>>();
                    let methods = methods
                        .split(|token| is_punctuation(token, ';'))
                        .filter(|tokens| !tokens.is_empty())
                        .map(parse_interface_method)
                        .collect::<Vec<_>>();
                    JavaDefinitionMetadataKind::Interface(JavaInterfaceMetadata {
                        methods,
                        ..interface
                    })
                }
                definition => definition,
            };
            JavaDefinitionMetadata {
                definition: java_definition,
                ..definition
            }
        })
        .collect();
    Metadata { definitions }
}

fn is_constructor(tokens: &[TokenTree], class_name: &JavaName) -> bool {
    let class_name_len = class_name
        .clone()
        .with_dots()
        .into_iter()
        .collect::<Vec<_>>()
        .len();
    if tokens.len() <= class_name_len {
        return false;
    }
    let tokens = &tokens[tokens.len() - class_name_len - 1..tokens.len() - 1];
    TokenStream::from_iter(tokens.iter().cloned()).to_string()
        == class_name.clone().with_dots().to_string()
}

fn parse_method_arguments(token: TokenTree) -> Vec<MethodArgument> {
    match token {
        TokenTree::Group(group) => {
            if group.delimiter() != Delimiter::Parenthesis {
                panic!("Expected method arguments in parenthesis, got {:?}.", group);
            }
            let arguments = group.stream().into_iter().collect::<Vec<_>>();
            arguments
                .split(|token| is_punctuation(token, ','))
                .filter(|tokens| !tokens.is_empty())
                .map(|tokens| tokens.split_last().unwrap())
                .map(|(last, others)| {
                    let name = match last {
                        TokenTree::Ident(ident) => ident.clone(),
                        token => panic!("Expected argument name, got {:?}.", token),
                    };
                    MethodArgument {
                        name,
                        data_type: JavaName::from_tokens(others.iter()),
                    }
                })
                .collect::<Vec<_>>()
        }
        token => panic!("Expected method arguments, got {:?}.", token),
    }
}

fn parse_method(tokens: &[TokenTree]) -> JavaClassMethod {
    let public = tokens.iter().any(|token| is_identifier(token, "public"));
    let is_static = tokens.iter().any(|token| is_identifier(token, "static"));
    let tokens = tokens
        .iter()
        .filter(|token| !is_identifier(token, "public") && !is_identifier(token, "static"))
        .cloned()
        .collect::<Vec<_>>();
    let name = match tokens[tokens.len() - 2].clone() {
        TokenTree::Ident(ident) => ident,
        token => panic!("Expected method name, got {:?}.", token),
    };
    let annotations = parse_annotations(&tokens[0..tokens.len() - 2]);
    let return_type = JavaName::from_tokens(
        tokens[0..tokens.len() - 2]
            .iter()
            .skip(3 * annotations.len()),
    );
    let arguments = parse_method_arguments(tokens[tokens.len() - 1].clone());
    JavaClassMethod {
        public,
        name,
        return_type,
        arguments,
        is_static,
        annotations,
    }
}

fn parse_interface_method(tokens: &[TokenTree]) -> JavaInterfaceMethod {
    let tokens = tokens.iter().cloned().collect::<Vec<_>>();
    let name = match tokens[tokens.len() - 2].clone() {
        TokenTree::Ident(ident) => ident,
        token => panic!("Expected method name, got {:?}.", token),
    };
    let annotations = parse_annotations(&tokens[0..tokens.len() - 2]);
    let return_type = JavaName::from_tokens(
        tokens[0..tokens.len() - 2]
            .iter()
            .skip(3 * annotations.len()),
    );
    let arguments = parse_method_arguments(tokens[tokens.len() - 1].clone());
    JavaInterfaceMethod {
        name,
        return_type,
        arguments,
        annotations,
    }
}

fn parse_native_method(tokens: &[TokenTree]) -> JavaNativeMethod {
    let public = tokens.iter().any(|token| is_identifier(token, "public"));
    let is_static = tokens.iter().any(|token| is_identifier(token, "static"));
    let tokens = tokens
        .iter()
        .filter(|token| {
            !is_identifier(token, "public")
                && !is_identifier(token, "static")
                && !is_identifier(token, "native")
        })
        .cloned()
        .collect::<Vec<_>>();
    let code = match tokens[tokens.len() - 1].clone() {
        TokenTree::Group(group) => {
            if group.delimiter() == Delimiter::Brace {
                group
            } else {
                panic!("Expected method code in braces, got {:?}.", group)
            }
        }
        token => panic!("Expected method code, got {:?}.", token),
    };
    let name = match tokens[tokens.len() - 3].clone() {
        TokenTree::Ident(ident) => ident,
        token => panic!("Expected method name, got {:?}.", token),
    };
    let annotations = parse_annotations(&tokens[0..tokens.len() - 3]);
    let return_type = JavaName::from_tokens(
        tokens[0..tokens.len() - 3]
            .iter()
            .skip(3 * annotations.len()),
    );
    let arguments = parse_method_arguments(tokens[tokens.len() - 2].clone());
    JavaNativeMethod {
        public,
        name,
        return_type,
        arguments,
        is_static,
        code,
        annotations,
    }
}

fn parse_constructor(tokens: &[TokenTree]) -> JavaConstructor {
    let public = tokens.iter().any(|token| is_identifier(token, "public"));
    let tokens = tokens
        .iter()
        .filter(|token| !is_identifier(token, "public"))
        .cloned()
        .collect::<Vec<_>>();
    let annotations = parse_annotations(&tokens[0..tokens.len() - 1]);
    let arguments = parse_method_arguments(tokens[tokens.len() - 1].clone());
    JavaConstructor {
        public,
        arguments,
        annotations,
    }
}

pub fn parse_java_definition(input: TokenStream) -> JavaDefinitions {
    let mut definitions = input.clone().into_iter().collect::<Vec<_>>();
    let metadata = if definitions.len() > 1
        && is_identifier(&definitions[definitions.len() - 2], "metadata")
    {
        match definitions.pop().unwrap() {
            TokenTree::Group(group) => {
                if group.delimiter() == Delimiter::Brace {
                    let metadata = parse_metadata(group.stream());
                    definitions.pop().unwrap();
                    metadata
                } else {
                    panic!("Expected braces, got {:?}.", group)
                }
            }
            token => panic!("Expected braces, got {:?}.", token),
        }
    } else {
        Metadata {
            definitions: vec![],
        }
    };
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
                let (name, extends) = parse_interface_header(header);
                JavaDefinition {
                    name,
                    public,
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        methods: vec![],
                        extends,
                    }),
                }
            } else {
                let (name, extends, implements) = parse_class_header(header);
                JavaDefinition {
                    name,
                    public,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends,
                        implements,
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }
            }
        })
        .zip(definitions.iter().cloned().filter(is_definition))
        .map(|(definition, token)| match token {
            TokenTree::Group(group) => (definition, group.stream()),
            _ => unreachable!(),
        })
        .map(|(definition, tokens)| {
            let methods = tokens.into_iter().collect::<Vec<_>>();
            let java_definition = match definition.definition.clone() {
                JavaDefinitionKind::Class(class) => {
                    let constructors = methods
                        .split(|token| is_punctuation(token, ';'))
                        .filter(|tokens| !tokens.is_empty())
                        .filter(|tokens| is_constructor(tokens, &definition.name))
                        .map(parse_constructor)
                        .collect::<Vec<_>>();
                    let native_methods = methods
                        .split(|token| is_punctuation(token, ';'))
                        .filter(|tokens| !tokens.is_empty())
                        .filter(|tokens| !is_constructor(tokens, &definition.name))
                        .filter(|tokens| tokens.iter().any(|token| is_identifier(token, "native")))
                        .map(parse_native_method)
                        .collect::<Vec<_>>();
                    let methods = methods
                        .split(|token| is_punctuation(token, ';'))
                        .filter(|tokens| !tokens.is_empty())
                        .filter(|tokens| !is_constructor(tokens, &definition.name))
                        .filter(|tokens| !tokens.iter().any(|token| is_identifier(token, "native")))
                        .map(parse_method)
                        .collect::<Vec<_>>();
                    JavaDefinitionKind::Class(JavaClass {
                        methods,
                        native_methods,
                        constructors,
                        ..class
                    })
                }
                JavaDefinitionKind::Interface(interface) => {
                    let methods = methods
                        .split(|token| is_punctuation(token, ';'))
                        .filter(|tokens| !tokens.is_empty())
                        .map(parse_interface_method)
                        .collect::<Vec<_>>();
                    JavaDefinitionKind::Interface(JavaInterface {
                        methods,
                        ..interface
                    })
                }
            };
            JavaDefinition {
                definition: java_definition,
                ..definition
            }
        })
        .collect();
    JavaDefinitions {
        definitions,
        metadata,
    }
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

fn is_metadata_definition(token: &TokenTree) -> bool {
    match token {
        TokenTree::Group(group) => group.delimiter() == Delimiter::Brace,
        TokenTree::Punct(puntuation) => puntuation.as_char() == ';',
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
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn one_class() {
        let input = quote!{
            class TestClass1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestClass1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: None,
                        implements: vec![],
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn one_class_extends() {
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
                        extends: Some(JavaName(quote!{test1})),
                        implements: vec![],
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn one_class_public() {
        let input = quote!{
            public class TestClass1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestClass1}),
                    public: true,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: None,
                        implements: vec![],
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn one_class_packaged() {
        let input = quote!{
            class a.b.TestClass1 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b TestClass1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: None,
                        implements: vec![],
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn one_class_implements() {
        let input = quote!{
            class TestClass1 implements test2, a.b.test3 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{TestClass1}),
                    public: false,
                    definition: JavaDefinitionKind::Class(JavaClass {
                        extends: None,
                        implements: vec![JavaName(quote!{test2}), JavaName(quote!{a b test3})],
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
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
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        methods: vec![],
                        extends: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
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
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        methods: vec![],
                        extends: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
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
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        methods: vec![],
                        extends: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
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
                        methods: vec![],
                        extends: vec![
                            JavaName(quote!{TestInterface2}),
                            JavaName(quote!{a b TestInterface3}),
                        ],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn multiple() {
        let input = quote!{
            interface TestInterface1 {}
            interface TestInterface2 {}
            class TestClass1 {}
            class TestClass2 {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{TestInterface1}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{TestInterface2}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{TestClass1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: None,
                            implements: vec![],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{TestClass2}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: None,
                            implements: vec![],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                ],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn metadata_empty() {
        let input = quote!{
            metadata {}
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![],
                metadata: Metadata {
                    definitions: vec![],
                },
            }
        );
    }

    #[test]
    fn metadata() {
        let input = quote!{
            metadata {
                interface TestInterface1 {}
                interface TestInterface2 extends TestInterface1 {}
                class TestClass2;
                class TestClass1 extends TestClass2 implements TestInterface1, TestInterface2;
            }
        };
        assert_eq!(
            parse_java_definition(input),
            JavaDefinitions {
                definitions: vec![],
                metadata: Metadata {
                    definitions: vec![
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{TestInterface1}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    extends: vec![],
                                    methods: vec![],
                                },
                            ),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{TestInterface2}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    extends: vec![JavaName(quote!{TestInterface1})],
                                    methods: vec![],
                                },
                            ),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{TestClass2}),
                            definition: JavaDefinitionMetadataKind::Class(JavaClassMetadata {
                                extends: None,
                                implements: vec![],
                            }),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{TestClass1}),
                            definition: JavaDefinitionMetadataKind::Class(JavaClassMetadata {
                                extends: Some(JavaName(quote!{TestClass2})),
                                implements: vec![
                                    JavaName(quote!{TestInterface1}),
                                    JavaName(quote!{TestInterface2}),
                                ],
                            }),
                        },
                    ],
                },
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
    #[should_panic(expected = "Expected braces")]
    fn metadata_not_group() {
        let input = quote!{
            metadata abc
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected braces")]
    fn metadata_not_braces_group() {
        let input = quote!{
            metadata ()
        };
        parse_java_definition(input);
    }

    #[test]
    #[should_panic(expected = "Expected \"class\" or \"interface\"")]
    fn invalid_definition_metadata_kind() {
        let input = quote!{
            metadata {
                abc
            }
        };
        parse_java_definition(input);
    }
}
