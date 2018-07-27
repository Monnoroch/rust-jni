#![recursion_limit = "1024"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
extern crate rust_jni;

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

    fn with_underscores(self) -> String {
        self.0
            .into_iter()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
            .join("_")
    }

    fn with_double_colons(self) -> TokenStream {
        let mut tokens = vec![];
        for token in self.0.into_iter() {
            tokens.extend(quote!{::});
            tokens.push(token);
        }
        TokenStream::from_iter(tokens.iter().cloned())
    }

    fn with_dots(self) -> TokenStream {
        let mut tokens = vec![];
        let mut first = true;
        for token in self.0.into_iter() {
            if first {
                first = false;
            } else {
                tokens.extend(quote!{.});
            }
            tokens.push(token);
        }
        TokenStream::from_iter(tokens.iter().cloned())
    }

    fn as_primitive_type(&self) -> Option<TokenStream> {
        let tokens = self.clone().0.into_iter().collect::<Vec<_>>();
        if tokens.len() == 1 {
            let token = &tokens[0];
            if is_identifier(&token, "int") {
                Some(quote!{i32})
            } else if is_identifier(&token, "long") {
                Some(quote!{i64})
            } else if is_identifier(&token, "char") {
                Some(quote!{char})
            } else if is_identifier(&token, "byte") {
                Some(quote!{u8})
            } else if is_identifier(&token, "boolean") {
                Some(quote!{bool})
            } else if is_identifier(&token, "float") {
                Some(quote!{f32})
            } else if is_identifier(&token, "double") {
                Some(quote!{f64})
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_jni_signature(&self) -> String {
        let tokens = self.clone().0.into_iter().collect::<Vec<_>>();
        if tokens.len() == 1 {
            let token = &tokens[0];
            if is_identifier(&token, "int") {
                <i32 as rust_jni::JavaType>::__signature().to_owned()
            } else if is_identifier(&token, "long") {
                <i64 as rust_jni::JavaType>::__signature().to_owned()
            } else if is_identifier(&token, "char") {
                <char as rust_jni::JavaType>::__signature().to_owned()
            } else if is_identifier(&token, "byte") {
                <u8 as rust_jni::JavaType>::__signature().to_owned()
            } else if is_identifier(&token, "boolean") {
                <bool as rust_jni::JavaType>::__signature().to_owned()
            } else if is_identifier(&token, "float") {
                panic!(
                    "float values are not supported for not. \
                     See https://github.com/Monnoroch/rust-jni/issues/25 for more details"
                )
            } else if is_identifier(&token, "double") {
                <f64 as rust_jni::JavaType>::__signature().to_owned()
            } else {
                format!("L{}_2", self.clone().with_underscores())
            }
        } else {
            format!("L{}_2", self.clone().with_underscores())
        }
    }

    fn as_rust_type(self) -> TokenStream {
        let primitive = self.as_primitive_type();
        let with_double_colons = self.with_double_colons();
        primitive.unwrap_or(quote!{#with_double_colons <'a>})
    }

    fn as_rust_type_no_lifetime(self) -> TokenStream {
        let primitive = self.as_primitive_type();
        let with_double_colons = self.with_double_colons();
        primitive.unwrap_or(quote!{#with_double_colons})
    }

    fn as_rust_type_reference(self) -> TokenStream {
        let primitive = self.as_primitive_type();
        let with_double_colons = self.with_double_colons();
        primitive.unwrap_or(quote!{& #with_double_colons <'a>})
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct MethodArgument {
    name: Ident,
    data_type: JavaName,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaInterfaceMethod {
    name: Ident,
    return_type: JavaName,
    arguments: Vec<MethodArgument>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaClassMethod {
    name: Ident,
    return_type: JavaName,
    arguments: Vec<MethodArgument>,
    public: bool,
    is_static: bool,
}

#[derive(Debug, Clone)]
struct JavaNativeMethod {
    name: Ident,
    return_type: JavaName,
    arguments: Vec<MethodArgument>,
    public: bool,
    is_static: bool,
    code: Group,
}

impl PartialEq for JavaNativeMethod {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Eq for JavaNativeMethod {}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaConstructor {
    arguments: Vec<MethodArgument>,
    public: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaClass {
    extends: Option<JavaName>,
    implements: Vec<JavaName>,
    methods: Vec<JavaClassMethod>,
    native_methods: Vec<JavaNativeMethod>,
    constructors: Vec<JavaConstructor>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaInterface {
    methods: Vec<JavaInterfaceMethod>,
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
struct JavaClassMetadata {
    extends: Option<JavaName>,
    implements: Vec<JavaName>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaInterfaceMetadata {
    extends: Vec<JavaName>,
    methods: Vec<JavaInterfaceMethod>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum JavaDefinitionMetadataKind {
    Class(JavaClassMetadata),
    Interface(JavaInterfaceMetadata),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinitionMetadata {
    name: JavaName,
    definition: JavaDefinitionMetadataKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Metadata {
    definitions: Vec<JavaDefinitionMetadata>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct JavaDefinitions {
    definitions: Vec<JavaDefinition>,
    metadata: Metadata,
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
    let public = tokens.iter().any(|token| is_identifier(token, "public"));
    let tokens = if public {
        &tokens[1..tokens.len() - 1]
    } else {
        &tokens[0..tokens.len() - 1]
    };
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
    let return_type = JavaName::from_tokens(tokens[0..tokens.len() - 2].iter());
    let arguments = parse_method_arguments(tokens[tokens.len() - 1].clone());
    JavaClassMethod {
        public,
        name,
        return_type,
        arguments,
        is_static,
    }
}

fn parse_interface_method(tokens: &[TokenTree]) -> JavaInterfaceMethod {
    let tokens = tokens.iter().cloned().collect::<Vec<_>>();
    let name = match tokens[tokens.len() - 2].clone() {
        TokenTree::Ident(ident) => ident,
        token => panic!("Expected method name, got {:?}.", token),
    };
    let return_type = JavaName::from_tokens(tokens[0..tokens.len() - 2].iter());
    let arguments = parse_method_arguments(tokens[tokens.len() - 1].clone());
    JavaInterfaceMethod {
        name,
        return_type,
        arguments,
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
    let return_type = JavaName::from_tokens(tokens[0..tokens.len() - 3].iter());
    let arguments = parse_method_arguments(tokens[tokens.len() - 2].clone());
    JavaNativeMethod {
        public,
        name,
        return_type,
        arguments,
        is_static,
        code,
    }
}

fn parse_constructor(tokens: &[TokenTree]) -> JavaConstructor {
    let public = tokens.iter().any(|token| is_identifier(token, "public"));
    let tokens = tokens
        .iter()
        .filter(|token| !is_identifier(token, "public"))
        .cloned()
        .collect::<Vec<_>>();
    let arguments = parse_method_arguments(tokens[tokens.len() - 1].clone());
    JavaConstructor { public, arguments }
}

fn parse_java_definition(input: TokenStream) -> JavaDefinitions {
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

fn is_punctuation(token: &TokenTree, value: char) -> bool {
    match token {
        TokenTree::Punct(punct) => punct.spacing() == Spacing::Alone && punct.as_char() == value,
        _ => false,
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

#[derive(Debug, Clone)]
struct ClassMethodGeneratorDefinition {
    name: Ident,
    java_name: Literal,
    return_type: TokenStream,
    argument_names: Vec<Ident>,
    argument_types: Vec<TokenStream>,
    public: TokenStream,
}

#[derive(Debug, Clone)]
struct InterfaceMethodGeneratorDefinition {
    name: Ident,
    return_type: TokenStream,
    argument_names: Vec<Ident>,
    argument_types: Vec<TokenStream>,
}

#[derive(Debug, Clone)]
struct InterfaceMethodImplementationGeneratorDefinition {
    name: Ident,
    return_type: TokenStream,
    argument_names: Vec<Ident>,
    argument_types: Vec<TokenStream>,
    class_cast: TokenStream,
}

#[derive(Debug, Clone)]
struct NativeMethodGeneratorDefinition {
    name: Ident,
    java_name: Ident,
    return_type: TokenStream,
    argument_names: Vec<Ident>,
    argument_types: Vec<TokenStream>,
    argument_types_no_lifetime: Vec<TokenStream>,
    public: TokenStream,
    code: Group,
}

#[derive(Debug, Clone)]
struct ConstructorGeneratorDefinition {
    name: Ident,
    argument_names: Vec<Ident>,
    argument_types: Vec<TokenStream>,
    public: TokenStream,
}

#[derive(Debug, Clone)]
struct InterfaceImplementationGeneratorDefinition {
    interface: TokenStream,
    methods: Vec<InterfaceMethodImplementationGeneratorDefinition>,
}

#[derive(Debug, Clone)]
struct ClassGeneratorDefinition {
    class: Ident,
    public: TokenStream,
    super_class: TokenStream,
    transitive_extends: Vec<TokenStream>,
    implements: Vec<InterfaceImplementationGeneratorDefinition>,
    signature: Literal,
    full_signature: Literal,
    constructors: Vec<ConstructorGeneratorDefinition>,
    methods: Vec<ClassMethodGeneratorDefinition>,
    static_methods: Vec<ClassMethodGeneratorDefinition>,
    native_methods: Vec<NativeMethodGeneratorDefinition>,
    static_native_methods: Vec<NativeMethodGeneratorDefinition>,
}

#[derive(Debug, Clone)]
struct InterfaceGeneratorDefinition {
    interface: Ident,
    public: TokenStream,
    extends: Vec<TokenStream>,
    methods: Vec<InterfaceMethodGeneratorDefinition>,
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

fn public_token(public: bool) -> TokenStream {
    if public {
        quote!{pub}
    } else {
        TokenStream::new()
    }
}

fn to_generator_method(method: JavaClassMethod) -> ClassMethodGeneratorDefinition {
    let JavaClassMethod {
        name,
        public,
        return_type,
        arguments,
        ..
    } = method;
    let public = public_token(public);
    let java_name = Literal::string(&name.to_string());
    ClassMethodGeneratorDefinition {
        name,
        java_name,
        public,
        return_type: return_type.as_rust_type(),
        argument_names: arguments
            .iter()
            .map(|argument| argument.name.clone())
            .collect(),
        argument_types: arguments
            .iter()
            .map(|argument| argument.data_type.clone().as_rust_type_reference())
            .collect(),
    }
}

fn to_generator_interface_method(
    method: JavaInterfaceMethod,
) -> InterfaceMethodGeneratorDefinition {
    let JavaInterfaceMethod {
        name,
        return_type,
        arguments,
        ..
    } = method;
    InterfaceMethodGeneratorDefinition {
        name,
        return_type: return_type.as_rust_type(),
        argument_names: arguments
            .iter()
            .map(|argument| argument.name.clone())
            .collect(),
        argument_types: arguments
            .iter()
            .map(|argument| argument.data_type.clone().as_rust_type_reference())
            .collect(),
    }
}

fn to_generator_interface_method_implementation(
    method: JavaInterfaceMethod,
    class_methods: &Vec<JavaClassMethod>,
    interface: &JavaName,
    super_class: &TokenStream,
) -> InterfaceMethodImplementationGeneratorDefinition {
    let JavaInterfaceMethod {
        name,
        return_type,
        arguments,
        ..
    } = method;
    let class_has_method = class_methods.iter().any(|class_method| {
        class_method.name == name
            && class_method.return_type == return_type
            && class_method.arguments == arguments
    });
    let interface = interface.clone().with_double_colons();
    InterfaceMethodImplementationGeneratorDefinition {
        name,
        return_type: return_type.as_rust_type(),
        argument_names: arguments
            .iter()
            .map(|argument| argument.name.clone())
            .collect(),
        argument_types: arguments
            .iter()
            .map(|argument| argument.data_type.clone().as_rust_type_reference())
            .collect(),
        class_cast: if class_has_method {
            quote!{Self}
        } else {
            quote!{ <#super_class as #interface> }
        },
    }
}

fn to_generator_native_method(
    method: JavaNativeMethod,
    class_name: &JavaName,
) -> NativeMethodGeneratorDefinition {
    let JavaNativeMethod {
        name,
        public,
        return_type,
        arguments,
        code,
        ..
    } = method;
    let public = public_token(public);
    let signatures = arguments
        .iter()
        .map(|argument| &argument.data_type)
        .map(|name| name.get_jni_signature())
        .collect::<Vec<_>>();
    let java_name = Ident::new(
        &format!(
            "Java_{}_{}__{}",
            class_name.clone().with_underscores(),
            name.to_string(),
            signatures.join("")
        ),
        Span::call_site(),
    );
    NativeMethodGeneratorDefinition {
        name,
        java_name,
        public,
        code,
        return_type: return_type.as_rust_type(),
        argument_names: arguments
            .iter()
            .map(|argument| argument.name.clone())
            .collect(),
        argument_types: arguments
            .iter()
            .map(|argument| argument.data_type.clone().as_rust_type())
            .collect(),
        argument_types_no_lifetime: arguments
            .iter()
            .map(|argument| argument.data_type.clone().as_rust_type_no_lifetime())
            .collect(),
    }
}

fn to_generator_constructor(constructor: JavaConstructor) -> ConstructorGeneratorDefinition {
    let JavaConstructor {
        public, arguments, ..
    } = constructor;
    let public = public_token(public);
    let name = Ident::new("init", Span::call_site());
    ConstructorGeneratorDefinition {
        name,
        public,
        argument_names: arguments
            .iter()
            .map(|argument| argument.name.clone())
            .collect(),
        argument_types: arguments
            .iter()
            .map(|argument| argument.data_type.clone().as_rust_type_reference())
            .collect(),
    }
}

fn get_interfaces(name: &Option<JavaName>, definitions: &Vec<JavaDefinition>) -> Vec<JavaName> {
    match name {
        None => vec![],
        Some(ref name) => {
            let definition = definitions
                .iter()
                .filter(|definition| definition.name == *name)
                .next();
            match definition {
                Some(ref definition) => match definition.definition {
                    JavaDefinitionKind::Class(ref class) => {
                        let mut interfaces = class.implements.clone();
                        interfaces.extend(get_interfaces(&class.extends, definitions));
                        interfaces
                    }
                    _ => unreachable!(),
                },
                None => vec![],
            }
        }
    }
}

fn to_generator_data(definitions: JavaDefinitions) -> GeneratorData {
    let mut extends_map = HashMap::new();
    definitions
        .definitions
        .clone()
        .into_iter()
        .filter(|definition| match definition.definition {
            JavaDefinitionKind::Class(_) => true,
            _ => false,
        })
        .for_each(|definition| {
            let JavaDefinition {
                name, definition, ..
            } = definition;
            match definition {
                JavaDefinitionKind::Class(class) => {
                    let JavaClass { extends, .. } = class;
                    extends_map.insert(name, extends.unwrap_or(JavaName(quote!{java lang Object})));
                }
                _ => unreachable!(),
            }
        });
    definitions
        .metadata
        .definitions
        .clone()
        .into_iter()
        .filter(|definition| match definition.definition {
            JavaDefinitionMetadataKind::Class(_) => true,
            _ => false,
        })
        .for_each(|definition| {
            let JavaDefinitionMetadata {
                name, definition, ..
            } = definition;
            match definition {
                JavaDefinitionMetadataKind::Class(class) => {
                    let JavaClassMetadata { extends, .. } = class;
                    extends_map.insert(name, extends.unwrap_or(JavaName(quote!{java lang Object})));
                }
                _ => unreachable!(),
            }
        });
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
    definitions
        .metadata
        .definitions
        .clone()
        .into_iter()
        .filter(|definition| match definition.definition {
            JavaDefinitionMetadataKind::Interface(_) => true,
            _ => false,
        })
        .for_each(|definition| {
            let JavaDefinitionMetadata {
                name, definition, ..
            } = definition;
            match definition {
                JavaDefinitionMetadataKind::Interface(interface) => {
                    let JavaInterfaceMetadata { extends, .. } = interface;
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
            .clone()
            .into_iter()
            .map(|definition| {
                let JavaDefinition {
                    name,
                    public,
                    definition,
                    ..
                } = definition;
                let definition_name = name.clone().name();
                let public = public_token(public);
                match definition {
                    JavaDefinitionKind::Class(class) => {
                        let JavaClass {
                            extends,
                            constructors,
                            methods,
                            native_methods,
                            ..
                        } = class;
                        let mut transitive_extends = vec![];
                        let mut current = name.clone();
                        loop {
                            let super_class = extends_map.get(&current);
                            if super_class.is_none() {
                                break;
                            }
                            let super_class = super_class.unwrap();
                            transitive_extends.push(super_class.clone().with_double_colons());
                            current = super_class.clone();
                        }
                        let string_signature = name.clone().with_slashes();
                        let signature = Literal::string(&string_signature);
                        let full_signature = Literal::string(&format!("L{};", string_signature));
                        let super_class = extends
                            .map(|name| name.with_double_colons())
                            .unwrap_or(quote!{::java::lang::Object});
                        let implements =
                            get_interfaces(&Some(name.clone()), &definitions.definitions);
                        let mut implements = implements
                            .iter()
                            .flat_map(|name| interface_extends.get(&name).unwrap().iter())
                            .chain(implements.iter())
                            .cloned()
                            .collect::<HashSet<_>>()
                            .into_iter()
                            .collect::<Vec<_>>();
                        implements.sort_by(|left, right| left.to_string().cmp(&right.to_string()));
                        let mut implements = implements
                            .into_iter()
                            .map(|name| InterfaceImplementationGeneratorDefinition {
                                interface: name.clone().with_double_colons(),
                                methods: definitions
                                    .definitions
                                    .iter()
                                    .filter(|definition| definition.name == name)
                                    .next()
                                    .map(|definition| match definition.definition {
                                        JavaDefinitionKind::Interface(ref interface) => interface
                                            .methods
                                            .clone()
                                            .into_iter()
                                            .zip(iter::repeat(definition.name.clone())),
                                        _ => unreachable!(),
                                    })
                                    .or(definitions
                                        .metadata
                                        .definitions
                                        .clone()
                                        .into_iter()
                                        .filter(|definition| definition.name == name)
                                        .map(|definition| match definition.definition {
                                            JavaDefinitionMetadataKind::Interface(
                                                ref interface,
                                            ) => interface
                                                .methods
                                                .clone()
                                                .into_iter()
                                                .zip(iter::repeat(definition.name.clone())),
                                            _ => unreachable!(),
                                        })
                                        .next())
                                    .unwrap()
                                    .map(|(method, name)| {
                                        to_generator_interface_method_implementation(
                                            method,
                                            &methods,
                                            &name,
                                            &super_class,
                                        )
                                    })
                                    .collect(),
                            })
                            .collect::<Vec<_>>();
                        let static_methods = methods
                            .iter()
                            .filter(|method| method.is_static)
                            .cloned()
                            .map(to_generator_method)
                            .collect();
                        let methods = methods
                            .iter()
                            .filter(|method| !method.is_static)
                            .cloned()
                            .map(to_generator_method)
                            .collect();
                        let constructors = constructors
                            .into_iter()
                            .map(to_generator_constructor)
                            .collect();
                        let static_native_methods = native_methods
                            .iter()
                            .filter(|method| method.is_static)
                            .cloned()
                            .map(|method| to_generator_native_method(method, &name))
                            .collect();
                        let native_methods = native_methods
                            .iter()
                            .filter(|method| !method.is_static)
                            .cloned()
                            .map(|method| to_generator_native_method(method, &name))
                            .collect();
                        GeneratorDefinition::Class(ClassGeneratorDefinition {
                            class: definition_name,
                            public,
                            super_class,
                            transitive_extends,
                            implements,
                            signature,
                            full_signature,
                            constructors,
                            methods,
                            static_methods,
                            native_methods,
                            static_native_methods,
                        })
                    }
                    JavaDefinitionKind::Interface(interface) => {
                        let JavaInterface {
                            methods, extends, ..
                        } = interface;
                        let methods = methods
                            .iter()
                            .cloned()
                            .map(to_generator_interface_method)
                            .collect();
                        GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                            interface: definition_name,
                            public,
                            methods,
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
                metadata: Metadata {
                    definitions: vec![],
                },
            }),
            GeneratorData {
                definitions: vec![],
            }
        );
    }

    #[test]
    fn metadata_only() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![],
                metadata: Metadata {
                    definitions: vec![
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{c d test1}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    methods: vec![],
                                    extends: vec![],
                                },
                            ),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{a b test2}),
                            definition: JavaDefinitionMetadataKind::Class(JavaClassMetadata {
                                extends: None,
                                implements: vec![JavaName(quote!{c d test1})],
                            }),
                        },
                    ],
                },
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
                        extends: Some(JavaName(quote!{c d test2})),
                        implements: vec![],
                        methods: vec![],
                        native_methods: vec![],
                        constructors: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{::c::d::test2},
                    transitive_extends: vec![quote!{::c::d::test2}],
                    implements: vec![],
                    signature: Literal::string("a/b/test1"),
                    full_signature: Literal::string("La/b/test1;"),
                    methods: vec![],
                    static_methods: vec![],
                    native_methods: vec![],
                    static_native_methods: vec![],
                    constructors: vec![],
                })],
            }
        );
    }

    #[test]
    fn one_class_no_extends() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![JavaDefinition {
                    name: JavaName(quote!{a b test1}),
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
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{::java::lang::Object},
                    transitive_extends: vec![quote!{::java::lang::Object}],
                    implements: vec![],
                    signature: Literal::string("a/b/test1"),
                    full_signature: Literal::string("La/b/test1;"),
                    methods: vec![],
                    static_methods: vec![],
                    native_methods: vec![],
                    static_native_methods: vec![],
                    constructors: vec![],
                })],
            }
        );
    }

    #[test]
    fn one_class_extends_recursive() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{c d test2}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: Some(JavaName(quote!{e f test3})),
                            implements: vec![],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: Some(JavaName(quote!{c d test2})),
                            implements: vec![],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                ],
                metadata: Metadata {
                    definitions: vec![
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{e f test4}),
                            definition: JavaDefinitionMetadataKind::Class(JavaClassMetadata {
                                extends: None,
                                implements: vec![],
                            }),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{e f test3}),
                            definition: JavaDefinitionMetadataKind::Class(JavaClassMetadata {
                                extends: Some(JavaName(quote!{e f test4})),
                                implements: vec![],
                            }),
                        },
                    ],
                },
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test2", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::e::f::test3},
                        transitive_extends: vec![
                            quote!{::e::f::test3},
                            quote!{::e::f::test4},
                            quote!{::java::lang::Object},
                        ],
                        implements: vec![],
                        signature: Literal::string("c/d/test2"),
                        full_signature: Literal::string("Lc/d/test2;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::c::d::test2},
                        transitive_extends: vec![
                            quote!{::c::d::test2},
                            quote!{::e::f::test3},
                            quote!{::e::f::test4},
                            quote!{::java::lang::Object},
                        ],
                        implements: vec![],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
                    }),
                ],
            }
        );
    }

    #[test]
    fn one_class_implements() {
        assert_eq!(
            to_generator_data(JavaDefinitions {
                definitions: vec![
                    JavaDefinition {
                        name: JavaName(quote!{e f test4}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: None,
                            implements: vec![
                                JavaName(quote!{e f test3}),
                                JavaName(quote!{e f test4}),
                            ],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                ],
                metadata: Metadata {
                    definitions: vec![JavaDefinitionMetadata {
                        name: JavaName(quote!{e f test3}),
                        definition: JavaDefinitionMetadataKind::Interface(JavaInterfaceMetadata {
                            extends: vec![],
                            methods: vec![],
                        }),
                    }],
                },
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test4", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::e::f::test3},
                                methods: vec![],
                            },
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::e::f::test4},
                                methods: vec![],
                            },
                        ],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
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
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![JavaName(quote!{e f test4})],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: None,
                            implements: vec![JavaName(quote!{e f test3})],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                ],
                metadata: Metadata {
                    definitions: vec![
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{g h test5}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    methods: vec![],
                                    extends: vec![],
                                },
                            ),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{e f test4}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    methods: vec![],
                                    extends: vec![JavaName(quote!{g h test5})],
                                },
                            ),
                        },
                    ],
                },
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::e::f::test4}],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::e::f::test3},
                                methods: vec![],
                            },
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::e::f::test4},
                                methods: vec![],
                            },
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::g::h::test5},
                                methods: vec![],
                            },
                        ],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
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
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![JavaName(quote!{g h test4})],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Class(JavaClass {
                            extends: None,
                            implements: vec![
                                JavaName(quote!{e f test3}),
                                JavaName(quote!{g h test4}),
                            ],
                            methods: vec![],
                            native_methods: vec![],
                            constructors: vec![],
                        }),
                    },
                ],
                metadata: Metadata {
                    definitions: vec![],
                },
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test4", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::g::h::test4}],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::e::f::test3},
                                methods: vec![],
                            },
                            InterfaceImplementationGeneratorDefinition {
                                interface: quote!{::g::h::test4},
                                methods: vec![],
                            },
                        ],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
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
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: quote!{pub},
                    super_class: quote!{::java::lang::Object},
                    transitive_extends: vec![quote!{::java::lang::Object}],
                    implements: vec![],
                    signature: Literal::string("a/b/test1"),
                    full_signature: Literal::string("La/b/test1;"),
                    methods: vec![],
                    static_methods: vec![],
                    native_methods: vec![],
                    static_native_methods: vec![],
                    constructors: vec![],
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
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        methods: vec![],
                        extends: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Interface(
                    InterfaceGeneratorDefinition {
                        interface: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                        methods: vec![],
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
                        name: JavaName(quote!{e f test3}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![JavaName(quote!{c d test2}), JavaName(quote!{e f test3})],
                        }),
                    },
                ],
                metadata: Metadata {
                    definitions: vec![
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{c d test4}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    methods: vec![],
                                    extends: vec![],
                                },
                            ),
                        },
                        JavaDefinitionMetadata {
                            name: JavaName(quote!{c d test2}),
                            definition: JavaDefinitionMetadataKind::Interface(
                                JavaInterfaceMetadata {
                                    methods: vec![],
                                    extends: vec![JavaName(quote!{c d test4})],
                                },
                            ),
                        },
                    ],
                },
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test3", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![quote!{::c::d::test2}, quote!{::e::f::test3}],
                        methods: vec![],
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
                    definition: JavaDefinitionKind::Interface(JavaInterface {
                        methods: vec![],
                        extends: vec![],
                    }),
                }],
                metadata: Metadata {
                    definitions: vec![],
                },
            }),
            GeneratorData {
                definitions: vec![GeneratorDefinition::Interface(
                    InterfaceGeneratorDefinition {
                        interface: Ident::new("test1", Span::call_site()),
                        public: quote!{pub},
                        extends: vec![],
                        methods: vec![],
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
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{e f test_if2}),
                        public: false,
                        definition: JavaDefinitionKind::Interface(JavaInterface {
                            methods: vec![],
                            extends: vec![],
                        }),
                    },
                    JavaDefinition {
                        name: JavaName(quote!{a b test1}),
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
                        name: JavaName(quote!{test2}),
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
            }),
            GeneratorData {
                definitions: vec![
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test_if1", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                        interface: Ident::new("test_if2", Span::call_site()),
                        public: TokenStream::new(),
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test1", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![],
                        signature: Literal::string("a/b/test1"),
                        full_signature: Literal::string("La/b/test1;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
                    }),
                    GeneratorDefinition::Class(ClassGeneratorDefinition {
                        class: Ident::new("test2", Span::call_site()),
                        public: TokenStream::new(),
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![],
                        signature: Literal::string("test2"),
                        full_signature: Literal::string("Ltest2;"),
                        methods: vec![],
                        static_methods: vec![],
                        native_methods: vec![],
                        static_native_methods: vec![],
                        constructors: vec![],
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

fn generate_interface_method(method: InterfaceMethodGeneratorDefinition) -> TokenStream {
    let InterfaceMethodGeneratorDefinition {
        name,
        return_type,
        argument_names,
        argument_types,
    } = method;
    quote!{
        fn #name(
            &self,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type>;
    }
}

fn generate_class_method(method: ClassMethodGeneratorDefinition) -> TokenStream {
    let ClassMethodGeneratorDefinition {
        name,
        java_name,
        return_type,
        public,
        argument_names,
        argument_types,
    } = method;
    let argument_names_1 = argument_names.clone();
    let argument_types_1 = argument_types.clone();
    quote!{
        #public fn #name(
            &self,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            // Safe because the method name and arguments are correct.
            unsafe {
                ::rust_jni::__generator::call_method::<_, _, _,
                    fn(#(#argument_types_1,)*) -> #return_type
                >
                (
                    self,
                    #java_name,
                    (#(#argument_names_1,)*),
                    token,
                )
            }
        }
    }
}

fn generate_static_class_method(method: ClassMethodGeneratorDefinition) -> TokenStream {
    let ClassMethodGeneratorDefinition {
        name,
        java_name,
        return_type,
        public,
        argument_names,
        argument_types,
    } = method;
    let argument_names_1 = argument_names.clone();
    let argument_types_1 = argument_types.clone();
    quote!{
        #public fn #name(
            env: &'a ::rust_jni::JniEnv<'a>,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            // Safe because the method name and arguments are correct.
            unsafe {
                ::rust_jni::__generator::call_static_method::<Self, _, _,
                    fn(#(#argument_types_1,)*) -> #return_type
                >
                (
                    env,
                    #java_name,
                    (#(#argument_names_1,)*),
                    token,
                )
            }
        }
    }
}

fn generate_class_native_method(method: NativeMethodGeneratorDefinition) -> TokenStream {
    let NativeMethodGeneratorDefinition {
        name,
        return_type,
        public,
        argument_names,
        argument_types,
        code,
        ..
    } = method;
    quote!{
        #public fn #name(
            &self,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            #code
        }
    }
}

fn generate_static_class_native_method(method: NativeMethodGeneratorDefinition) -> TokenStream {
    let NativeMethodGeneratorDefinition {
        name,
        return_type,
        public,
        argument_names,
        argument_types,
        code,
        ..
    } = method;
    quote!{
        #public fn #name(
            env: &'a ::rust_jni::JniEnv<'a>,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            #code
        }
    }
}

fn generate_class_native_method_function(
    method: NativeMethodGeneratorDefinition,
    class_name: &Ident,
) -> TokenStream {
    let NativeMethodGeneratorDefinition {
        name,
        java_name,
        return_type,
        argument_names,
        argument_types_no_lifetime,
        ..
    } = method;
    let argument_names_1 = argument_names.clone();
    let argument_names_2 = argument_names.clone();
    let argument_names_3 = argument_names.clone();
    let argument_types_no_lifetime_1 = argument_types_no_lifetime.clone();
    quote!{
        #[no_mangle]
        #[doc(hidden)]
        pub unsafe extern "C" fn #java_name<'a>(
            raw_env: *mut ::jni_sys::JNIEnv,
            object: ::jni_sys::jobject,
            #(#argument_names: <#argument_types_no_lifetime as ::rust_jni::JavaType>::__JniType,)*
        ) -> <#return_type as ::rust_jni::JavaType>::__JniType {
            // TODO: make sure `#return_type: ::rust_jni::__generator::FromJni`.
            // Compile-time check that declared arguments implement the `JniArgumentType`
            // trait.
            #(::rust_jni::__generator::test_jni_argument_type(#argument_names_1);)*
            ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                // Compile-time check that declared arguments implement the `FromJni` trait.
                #(
                    {
                        let value =
                            <#argument_types_no_lifetime_1 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, #argument_names_2);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }
                )*

                let object = <#class_name as ::rust_jni::__generator::FromJni>::__from_jni(env, object);
                object
                    .#name(
                        #(::rust_jni::__generator::FromJni::__from_jni(env, #argument_names_3),)*
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
    }
}

fn generate_static_class_native_method_function(
    method: NativeMethodGeneratorDefinition,
    class_name: &Ident,
) -> TokenStream {
    let NativeMethodGeneratorDefinition {
        name,
        java_name,
        return_type,
        argument_names,
        argument_types_no_lifetime,
        ..
    } = method;
    let argument_names_1 = argument_names.clone();
    let argument_names_2 = argument_names.clone();
    let argument_names_3 = argument_names.clone();
    let argument_types_no_lifetime_1 = argument_types_no_lifetime.clone();
    let class_mismatch_error = format!(
        "Native method {} does not belong to class {}",
        name.to_string(),
        class_name.to_string()
    );
    quote!{
        #[no_mangle]
        #[doc(hidden)]
        pub unsafe extern "C" fn #java_name<'a>(
            raw_env: *mut ::jni_sys::JNIEnv,
            raw_class: ::jni_sys::jclass,
            #(#argument_names: <#argument_types_no_lifetime as ::rust_jni::JavaType>::__JniType,)*
        ) -> <#return_type as ::rust_jni::JavaType>::__JniType {
            // TODO: make sure `#return_type: ::rust_jni::__generator::FromJni`.
            // Compile-time check that declared arguments implement the `JniArgumentType`
            // trait.
            #(::rust_jni::__generator::test_jni_argument_type(#argument_names_1);)*
            ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                // Compile-time check that declared arguments implement the `FromJni` trait.
                #(
                    {
                        let value =
                            <#argument_types_no_lifetime_1 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, #argument_names_2);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }
                )*

                let class = #class_name::get_class(env, &token)?;
                let raw_class = <::rust_jni::java::lang::Class as ::rust_jni::__generator::FromJni>::__from_jni(env, raw_class);
                if !class.is_same_as(&raw_class, &token) {
                    // This should never happen, as native method's link name has the class,
                    // so it must be bound to a correct clas by the JVM.
                    // Still, this is a good test to ensure that the system
                    // is in a consistent state.
                    panic!(#class_mismatch_error);
                }

                #class_name::#name(
                    env,
                    #(::rust_jni::__generator::FromJni::__from_jni(env, #argument_names_3),)*
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
    }
}

fn generate_constructor(method: ConstructorGeneratorDefinition) -> TokenStream {
    let ConstructorGeneratorDefinition {
        name,
        public,
        argument_names,
        argument_types,
    } = method;
    let argument_names_1 = argument_names.clone();
    let argument_types_1 = argument_types.clone();
    quote!{
        #public fn #name(
            env: &'a ::rust_jni::JniEnv<'a>,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, Self> {
            // Safe because the method name and arguments are correct.
            unsafe {
                ::rust_jni::__generator::call_constructor::<Self, _, fn(#(#argument_types_1,)*)>
                (
                    env,
                    (#(#argument_names_1,)*),
                    token,
                )
            }
        }
    }
}

fn generate_interface_method_implementation(
    method: InterfaceMethodImplementationGeneratorDefinition,
) -> TokenStream {
    let InterfaceMethodImplementationGeneratorDefinition {
        name,
        argument_names,
        argument_types,
        return_type,
        class_cast,
    } = method;
    let argument_names_1 = argument_names.clone();
    quote!{
        fn #name(
            &self,
            #(#argument_names: #argument_types),*,
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            #class_cast::#name(
                self, #(#argument_names_1),*, token
            )
        }
    }
}

fn generate_interface_implementation(
    interface: InterfaceImplementationGeneratorDefinition,
    class: &Ident,
) -> TokenStream {
    let InterfaceImplementationGeneratorDefinition {
        interface, methods, ..
    } = interface;
    let methods = methods
        .into_iter()
        .map(generate_interface_method_implementation)
        .collect::<Vec<_>>();
    quote! {
        impl<'a> #interface<'a> for #class<'a> {
            #(
                #methods
            )*
        }
    }
}

fn generate_class_definition(definition: ClassGeneratorDefinition) -> TokenStream {
    let ClassGeneratorDefinition {
        class,
        public,
        super_class,
        transitive_extends,
        implements,
        signature,
        full_signature,
        constructors,
        methods,
        static_methods,
        native_methods,
        static_native_methods,
        ..
    } = definition;
    let multiplied_class = iter::repeat(class.clone());
    let multiplied_class_1 = multiplied_class.clone();
    let transitive_extends_1 = transitive_extends.clone();
    let methods = methods
        .into_iter()
        .map(generate_class_method)
        .collect::<Vec<_>>();
    let static_methods = static_methods
        .into_iter()
        .map(generate_static_class_method)
        .collect::<Vec<_>>();
    let native_method_functions = native_methods
        .clone()
        .into_iter()
        .map(|method| generate_class_native_method_function(method, &class))
        .collect::<Vec<_>>();
    let static_native_method_functions = static_native_methods
        .clone()
        .into_iter()
        .map(|method| generate_static_class_native_method_function(method, &class))
        .collect::<Vec<_>>();
    let native_methods = native_methods
        .into_iter()
        .map(generate_class_native_method)
        .collect::<Vec<_>>();
    let static_native_methods = static_native_methods
        .into_iter()
        .map(generate_static_class_native_method)
        .collect::<Vec<_>>();
    let constructors = constructors
        .into_iter()
        .map(generate_constructor)
        .collect::<Vec<_>>();
    let implementations = implements
        .into_iter()
        .map(|interface| generate_interface_implementation(interface, &class))
        .collect::<Vec<_>>();
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

        #(
            impl<'a> ::rust_jni::Cast<'a, #transitive_extends<'a>> for #multiplied_class_1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b #transitive_extends_1<'a> {
                    self
                }
            }
        )*

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

            #(
                #constructors
            )*

            #(
                #methods
            )*

            #(
                #static_methods
            )*

            #(
                #native_methods
            )*

            #(
                #static_native_methods
            )*
        }

        // TODO: put them into an anonymous module.

        #(
            #native_method_functions
        )*

        #(
            #static_native_method_functions
        )*

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
            #implementations
        )*
    }
}

fn generate_interface_definition(definition: InterfaceGeneratorDefinition) -> TokenStream {
    let InterfaceGeneratorDefinition {
        interface,
        public,
        extends,
        methods,
        ..
    } = definition;
    let extends = if extends.is_empty() {
        TokenStream::new()
    } else {
        quote!{: #(#extends<'a>)+*}
    };
    let methods = methods
        .into_iter()
        .map(generate_interface_method)
        .collect::<Vec<_>>();
    quote! {
        #public trait #interface<'a> #extends {
            #(
                #methods
            )*
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
                transitive_extends: vec![quote!{c::d::test2}],
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![],
                native_methods: vec![],
                static_native_methods: vec![],
                constructors: vec![],
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
                transitive_extends: vec![quote!{c::d::test2}],
                implements: vec![
                    InterfaceImplementationGeneratorDefinition {
                        interface: quote!{e::f::test3},
                        methods: vec![],
                    },
                    InterfaceImplementationGeneratorDefinition {
                        interface: quote!{e::f::test4},
                        methods: vec![],
                    },
                ],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![],
                native_methods: vec![],
                static_native_methods: vec![],
                constructors: vec![],
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

            impl<'a> e::f::test3<'a> for test1<'a> {
            }

            impl<'a> e::f::test4<'a> for test1<'a> {
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
                    methods: vec![],
                },
            )],
        };
        let expected = quote!{
            test_public trait test1<'a> {
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
                    methods: vec![],
                },
            )],
        };
        let expected = quote!{
            trait test1<'a> : c::d::test2<'a> + e::f::test3<'a> {
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
                    methods: vec![],
                }),
                GeneratorDefinition::Interface(InterfaceGeneratorDefinition {
                    interface: Ident::new("test_if2", Span::call_site()),
                    public: TokenStream::new(),
                    extends: vec![],
                    methods: vec![],
                }),
                GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test1", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{c::d::test3},
                    transitive_extends: vec![quote!{c::d::test3}],
                    implements: vec![],
                    signature: Literal::string("test/sign1"),
                    full_signature: Literal::string("test/signature1"),
                    methods: vec![],
                    static_methods: vec![],
                    constructors: vec![],
                    native_methods: vec![],
                    static_native_methods: vec![],
                }),
                GeneratorDefinition::Class(ClassGeneratorDefinition {
                    class: Ident::new("test2", Span::call_site()),
                    public: TokenStream::new(),
                    super_class: quote!{c::d::test4},
                    transitive_extends: vec![quote!{c::d::test4}],
                    implements: vec![],
                    signature: Literal::string("test/sign2"),
                    full_signature: Literal::string("test/signature2"),
                    methods: vec![],
                    static_methods: vec![],
                    native_methods: vec![],
                    static_native_methods: vec![],
                    constructors: vec![],
                }),
            ],
        };
        let expected = quote!{
            trait test_if1<'a> {
            }

            trait test_if2<'a> {
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
                long primitiveFunc3(int arg1, char arg2);
                c.d.TestClass2 objectFunc3(a.b.TestClass3 arg);
            }

            public class a.b.TestClass3 extends c.d.TestClass2 implements e.f.TestInterface1, a.b.TestInterface4 {
                public a.b.TestClass3(int arg1, a.b.TestClass3 arg2);

                long primitiveFunc3(int arg1, char arg2);
                public c.d.TestClass2 objectFunc3(a.b.TestClass3 arg);

                static long primitiveStaticFunc3(int arg1, char arg2);
                public static c.d.TestClass2 objectStaticFunc3(a.b.TestClass3 arg);

                public native long primitiveNativeFunc3(int arg1, char arg2) {
                    println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, self);
                    Ok(0)
                };
                native a.b.TestClass3 objectNativeFunc3(a.b.TestClass3 arg) {
                    println!("{:?} {:?} {:?}", arg, token, self);
                    Ok(arg)
                };

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
                fn primitiveFunc3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64>;

                fn objectFunc3(
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

                fn primitiveFunc3(
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

                pub fn objectFunc3(
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

                fn primitiveStaticFunc3(
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

                pub fn objectStaticFunc3(
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

                pub fn primitiveNativeFunc3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    {
                        println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, self);
                        Ok(0)
                    }
                }

                fn objectNativeFunc3(
                    &self,
                    arg: ::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> > {
                    {
                        println!("{:?} {:?} {:?}", arg, token, self);
                        Ok(arg)
                    }
                }

                fn primitiveStaticNativeFunc3(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    {
                        println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, env);
                        Ok(0)
                    }
                }

                pub fn objectStaticNativeFunc3(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg: ::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::a::b::TestClass3<'a> > {
                    {
                        println!("{:?} {:?} {:?}", arg, token, env);
                        Ok(arg)
                    }
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
                        .primitiveNativeFunc3(
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

                    TestClass3::primitiveStaticNativeFunc3(
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
                fn primitiveFunc3(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    Self::primitiveFunc3(self, arg1, arg2, token)
                }

                fn objectFunc3(
                    &self,
                    arg: &::a::b::TestClass3<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, ::c::d::TestClass2<'a> > {
                    Self::objectFunc3(self, arg, token)
                }
            }

            impl<'a> ::c::d::TestInterface2<'a> for TestClass3<'a> {
            }

            impl<'a> ::e::f::TestInterface1<'a> for TestClass3<'a> {
                fn primitiveInterfaceFunc1(
                    &self,
                    arg1: i32,
                    arg2: char,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, i64> {
                    < ::c::d::TestClass2 as ::e::f::TestInterface1 >
                        ::primitiveInterfaceFunc1(self, arg1, arg2, token)
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
