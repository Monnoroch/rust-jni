use proc_macro2::*;
use quote::ToTokens;
use rust_jni;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct JavaName(pub TokenStream);

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
    pub fn from_tokens<'a>(tokens: impl Iterator<Item = &'a TokenTree>) -> JavaName {
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
        })
        .filter(|token| match token {
            TokenTree::Ident(_) => true,
            _ => false,
        });
        let tokens = TokenStream::from_iter(tokens.cloned());
        if tokens.is_empty() {
            panic!("Expected a Java name, got no tokens.");
        }
        JavaName(tokens)
    }

    pub fn name(self) -> Ident {
        match self.0.into_iter().last().unwrap() {
            TokenTree::Ident(identifier) => identifier,
            token => panic!("Expected an identifier, got {:?}", token),
        }
    }

    pub fn with_slashes(self) -> String {
        self.0
            .into_iter()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn with_underscores(self) -> String {
        self.0
            .into_iter()
            .map(|token| token.to_string())
            .collect::<Vec<_>>()
            .join("_")
    }

    pub fn with_double_colons(self) -> TokenStream {
        let mut tokens = vec![];
        for token in self.0.into_iter() {
            tokens.extend(quote! {::});
            tokens.push(token);
        }
        TokenStream::from_iter(tokens.iter().cloned())
    }

    pub fn with_dots(self) -> TokenStream {
        let mut tokens = vec![];
        let mut first = true;
        for token in self.0.into_iter() {
            if first {
                first = false;
            } else {
                tokens.extend(quote! {.});
            }
            tokens.push(token);
        }
        TokenStream::from_iter(tokens.iter().cloned())
    }

    pub fn as_primitive_type(&self) -> Option<TokenStream> {
        let tokens = self.clone().0.into_iter().collect::<Vec<_>>();
        if tokens.len() == 1 {
            let token = &tokens[0];
            if is_identifier(&token, "int") {
                Some(quote! {i32})
            } else if is_identifier(&token, "long") {
                Some(quote! {i64})
            } else if is_identifier(&token, "char") {
                Some(quote! {char})
            } else if is_identifier(&token, "byte") {
                Some(quote! {u8})
            } else if is_identifier(&token, "boolean") {
                Some(quote! {bool})
            } else if is_identifier(&token, "float") {
                Some(quote! {f32})
            } else if is_identifier(&token, "double") {
                Some(quote! {f64})
            } else if is_identifier(&token, "void") {
                Some(quote! {()})
            } else if is_identifier(&token, "short") {
                Some(quote! {i64})
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_jni_signature(&self) -> String {
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
            } else if is_identifier(&token, "void") {
                <() as rust_jni::JavaType>::__signature().to_owned()
            } else if is_identifier(&token, "short") {
                <i16 as rust_jni::JavaType>::__signature().to_owned()
            } else {
                format!("L{}_2", self.clone().with_underscores())
            }
        } else {
            format!("L{}_2", self.clone().with_underscores())
        }
    }

    pub fn as_rust_type(self) -> TokenStream {
        let primitive = self.as_primitive_type();
        let with_double_colons = self.with_double_colons();
        primitive.unwrap_or(quote! {#with_double_colons <'a>})
    }

    pub fn as_rust_type_no_lifetime(self) -> TokenStream {
        let primitive = self.as_primitive_type();
        let with_double_colons = self.with_double_colons();
        primitive.unwrap_or(quote! {#with_double_colons})
    }

    pub fn as_rust_type_reference(self) -> TokenStream {
        let primitive = self.as_primitive_type();
        let with_double_colons = self.with_double_colons();
        primitive.unwrap_or(quote! {& #with_double_colons <'a>})
    }
}

fn is_identifier(token: &TokenTree, name: &str) -> bool {
    match token {
        TokenTree::Ident(identifier) => identifier == name,
        _ => false,
    }
}
