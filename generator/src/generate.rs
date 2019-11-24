#[cfg(test)]
use super::assert_tokens_equals;
use proc_macro2::*;
use std::iter;
use std::iter::FromIterator;

#[derive(Debug)]
pub struct ClassMethod {
    pub name: Ident,
    pub java_name: Literal,
    pub return_type: TokenStream,
    pub argument_names: Vec<Ident>,
    pub argument_types: Vec<TokenStream>,
    pub public: bool,
}

#[derive(Debug)]
pub struct InterfaceMethod {
    pub name: Ident,
    pub return_type: TokenStream,
    pub argument_names: Vec<Ident>,
    pub argument_types: Vec<TokenStream>,
}

#[derive(Debug)]
pub struct InterfaceMethodImplementation {
    pub name: Ident,
    pub return_type: TokenStream,
    pub argument_names: Vec<Ident>,
    pub argument_types: Vec<TokenStream>,
    pub class_has_method: bool,
}

#[derive(Debug)]
pub struct NativeMethod {
    pub name: Ident,
    pub rust_name: Ident,
    pub java_name: Ident,
    pub return_type: TokenStream,
    pub argument_names: Vec<Ident>,
    pub argument_types: Vec<TokenStream>,
    pub argument_types_no_lifetime: Vec<TokenStream>,
    pub public: bool,
    pub code: TokenStream,
}

#[derive(Debug)]
pub struct Constructor {
    pub name: Ident,
    pub argument_names: Vec<Ident>,
    pub argument_types: Vec<TokenStream>,
    pub public: bool,
}

#[derive(Debug)]
pub struct InterfaceImplementation {
    pub interface: TokenStream,
    pub methods: Vec<InterfaceMethodImplementation>,
}

#[derive(Debug)]
pub struct Class {
    pub class: Ident,
    pub public: bool,
    pub super_class: TokenStream,
    pub transitive_extends: Vec<TokenStream>,
    pub implements: Vec<InterfaceImplementation>,
    pub signature: Literal,
    pub full_signature: Literal,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<ClassMethod>,
    pub static_methods: Vec<ClassMethod>,
    pub native_methods: Vec<NativeMethod>,
    pub static_native_methods: Vec<NativeMethod>,
}

#[derive(Debug)]
pub struct Interface {
    pub interface: Ident,
    pub public: bool,
    pub extends: Vec<TokenStream>,
    pub methods: Vec<InterfaceMethod>,
}

#[derive(Debug)]
pub enum GeneratorDefinition {
    Interface(Interface),
    Class(Class),
}

#[derive(Debug)]
pub struct GeneratorData {
    pub definitions: Vec<GeneratorDefinition>,
}

pub fn generate(data: &GeneratorData) -> TokenStream {
    TokenStream::from_iter(
        data.definitions
            .iter()
            .map(generate_definition)
            .flat_map(|tokens| tokens.into_iter()),
    )
}

fn generate_definition(definition: &GeneratorDefinition) -> TokenStream {
    match definition {
        GeneratorDefinition::Interface(interface) => generate_interface(interface),
        GeneratorDefinition::Class(class) => generate_class(class),
    }
}

fn generate_interface(definition: &Interface) -> TokenStream {
    let Interface {
        interface,
        public,
        extends,
        methods,
    } = definition;
    let extends = if extends.is_empty() {
        quote! {}
    } else {
        quote! {: #(#extends<'a>)+*}
    };
    let methods = methods.iter().map(generate_interface_method);
    let public = generate_public(*public);
    quote! {
        #public trait #interface<'a> #extends {
            #(
                #methods
            )*
        }
    }
}

fn generate_interface_method(method: &InterfaceMethod) -> TokenStream {
    let InterfaceMethod {
        name,
        return_type,
        argument_names,
        argument_types,
    } = method;
    quote! {
        fn #name(
            &self,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type>;
    }
}

fn generate_class(definition: &Class) -> TokenStream {
    let Class {
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
    } = definition;
    let multiplied_class = iter::repeat(class);
    let transitive_extends_1 = transitive_extends.iter();
    let transitive_extends = transitive_extends.iter();
    let methods = methods.iter().map(generate_class_method);
    let static_methods = static_methods.iter().map(generate_static_class_method);
    let native_method_functions = native_methods
        .iter()
        .map(|method| generate_class_native_method_function(method, class));
    let static_native_method_functions = static_native_methods
        .iter()
        .map(|method| generate_static_class_native_method_function(method, class));
    let native_methods = native_methods.iter().map(generate_class_native_method);
    let static_native_methods = static_native_methods
        .iter()
        .map(generate_static_class_native_method);
    let constructors = constructors.iter().map(generate_constructor);
    let implementations = implements
        .iter()
        .map(|interface| generate_interface_implementation(interface, class, super_class));
    let public = generate_public(*public);
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
            impl<'a> ::rust_jni::Cast<'a, #transitive_extends<'a>> for #multiplied_class<'a> {
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

fn generate_constructor(method: &Constructor) -> TokenStream {
    let Constructor {
        name,
        public,
        argument_names,
        argument_types,
    } = method;
    let argument_names_1 = argument_names.iter();
    let argument_names = argument_names.iter();
    let argument_types_1 = argument_types.iter();
    let argument_types = argument_types.iter();
    let public = generate_public(*public);
    quote! {
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

fn generate_class_method(method: &ClassMethod) -> TokenStream {
    let ClassMethod {
        name,
        java_name,
        return_type,
        public,
        argument_names,
        argument_types,
    } = method;
    let argument_names_1 = argument_names.iter();
    let argument_names = argument_names.iter();
    let argument_types_1 = argument_types.iter();
    let argument_types = argument_types.iter();
    let public = generate_public(*public);
    quote! {
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

fn generate_static_class_method(method: &ClassMethod) -> TokenStream {
    let ClassMethod {
        name,
        java_name,
        return_type,
        public,
        argument_names,
        argument_types,
    } = method;
    let argument_names_1 = argument_names.iter();
    let argument_names = argument_names.iter();
    let argument_types_1 = argument_types.iter();
    let argument_types = argument_types.iter();
    let public = generate_public(*public);
    quote! {
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

fn generate_class_native_method(method: &NativeMethod) -> TokenStream {
    let NativeMethod {
        rust_name,
        return_type,
        public,
        argument_names,
        argument_types,
        code,
        ..
    } = method;
    let public = generate_public(*public);
    quote! {
        #public fn #rust_name(
            &self,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            #code
        }
    }
}

fn generate_static_class_native_method(method: &NativeMethod) -> TokenStream {
    let NativeMethod {
        rust_name,
        return_type,
        public,
        argument_names,
        argument_types,
        code,
        ..
    } = method;
    let public = generate_public(*public);
    quote! {
        #public fn #rust_name(
            env: &'a ::rust_jni::JniEnv<'a>,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            #code
        }
    }
}

fn generate_class_native_method_function(method: &NativeMethod, class_name: &Ident) -> TokenStream {
    let NativeMethod {
        rust_name,
        java_name,
        return_type,
        argument_names,
        argument_types_no_lifetime,
        ..
    } = method;
    let argument_names_1 = argument_names.iter();
    let argument_names_2 = argument_names.iter();
    let argument_names_3 = argument_names.iter();
    let argument_names = argument_names.iter();
    let argument_types_no_lifetime_1 = argument_types_no_lifetime.iter();
    let argument_types_no_lifetime = argument_types_no_lifetime.iter();
    quote! {
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
                    .#rust_name(
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
    method: &NativeMethod,
    class_name: &Ident,
) -> TokenStream {
    let NativeMethod {
        name,
        rust_name,
        java_name,
        return_type,
        argument_names,
        argument_types_no_lifetime,
        ..
    } = method;
    let argument_names_1 = argument_names.iter();
    let argument_names_2 = argument_names.iter();
    let argument_names_3 = argument_names.iter();
    let argument_names = argument_names.iter();
    let argument_types_no_lifetime_1 = argument_types_no_lifetime.iter();
    let argument_types_no_lifetime = argument_types_no_lifetime.iter();
    let class_mismatch_error = format!(
        "Native method {} does not belong to class {}",
        name.to_string(),
        class_name.to_string()
    );
    quote! {
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

                #class_name::#rust_name(
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

fn generate_interface_method_implementation(
    method: &InterfaceMethodImplementation,
    interface: &TokenStream,
    super_class: &TokenStream,
) -> TokenStream {
    let InterfaceMethodImplementation {
        name,
        argument_names,
        argument_types,
        return_type,
        class_has_method,
    } = method;
    let argument_names_1 = argument_names.iter();
    let argument_names = argument_names.iter();
    let class_cast = if *class_has_method {
        quote! {Self}
    } else {
        quote! { <#super_class as #interface> }
    };
    quote! {
        fn #name(
            &self,
            #(#argument_names: #argument_types,)*
            token: &::rust_jni::NoException<'a>,
        ) -> ::rust_jni::JavaResult<'a, #return_type> {
            #class_cast::#name(
                self, #(#argument_names_1,)* token
            )
        }
    }
}

fn generate_interface_implementation(
    interface: &InterfaceImplementation,
    class: &Ident,
    super_class: &TokenStream,
) -> TokenStream {
    let InterfaceImplementation { interface, methods } = interface;
    let methods = methods
        .iter()
        .map(|method| generate_interface_method_implementation(method, interface, super_class));
    quote! {
        impl<'a> #interface<'a> for #class<'a> {
            #(
                #methods
            )*
        }
    }
}

fn generate_public(public: bool) -> TokenStream {
    if public {
        quote! {pub}
    } else {
        quote! {}
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
        let expected = quote! {};
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn multiple_definitions() {
        let input = GeneratorData {
            definitions: vec![
                GeneratorDefinition::Interface(Interface {
                    interface: Ident::new("test_if1", Span::call_site()),
                    public: false,
                    extends: vec![],
                    methods: vec![],
                }),
                GeneratorDefinition::Interface(Interface {
                    interface: Ident::new("test_if2", Span::call_site()),
                    public: false,
                    extends: vec![],
                    methods: vec![],
                }),
                GeneratorDefinition::Class(Class {
                    class: Ident::new("test1", Span::call_site()),
                    public: false,
                    super_class: quote! {c::d::test3},
                    transitive_extends: vec![],
                    implements: vec![],
                    signature: Literal::string("test/sign1"),
                    full_signature: Literal::string("test/signature1"),
                    methods: vec![],
                    static_methods: vec![],
                    constructors: vec![],
                    native_methods: vec![],
                    static_native_methods: vec![],
                }),
                GeneratorDefinition::Class(Class {
                    class: Ident::new("test2", Span::call_site()),
                    public: false,
                    super_class: quote! {c::d::test4},
                    transitive_extends: vec![],
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
        let expected = quote! {
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
        assert_tokens_equals(generate(&input), expected);
    }
}

#[cfg(test)]
mod generate_interface_tests {
    use super::*;

    #[test]
    fn interface() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Interface(Interface {
                interface: Ident::new("test1", Span::call_site()),
                public: false,
                extends: vec![],
                methods: vec![],
            })],
        };
        let expected = quote! {
            trait test1<'a> {
            }
        };
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn public() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Interface(Interface {
                interface: Ident::new("test1", Span::call_site()),
                public: true,
                extends: vec![],
                methods: vec![],
            })],
        };
        let expected = quote! {
            pub trait test1<'a> {
            }
        };
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn extends() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Interface(Interface {
                interface: Ident::new("test1", Span::call_site()),
                public: false,
                extends: vec![quote! {c::d::test2}, quote! {e::f::test3}],
                methods: vec![],
            })],
        };
        let expected = quote! {
            trait test1<'a> : c::d::test2<'a> + e::f::test3<'a> {
            }
        };
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn methods() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Interface(Interface {
                interface: Ident::new("test1", Span::call_site()),
                public: false,
                extends: vec![],
                methods: vec![
                    InterfaceMethod {
                        name: Ident::new("test_method_1", Span::call_site()),
                        return_type: quote! {return_type_1},
                        argument_names: vec![
                            Ident::new("arg1", Span::call_site()),
                            Ident::new("arg2", Span::call_site()),
                        ],
                        argument_types: vec![quote! {type1}, quote! {type2}],
                    },
                    InterfaceMethod {
                        name: Ident::new("test_method_2", Span::call_site()),
                        return_type: quote! {return_type_2},
                        argument_names: vec![],
                        argument_types: vec![],
                    },
                ],
            })],
        };
        let expected = quote! {
            trait test1<'a> {
                fn test_method_1(
                    &self,
                    arg1: type1,
                    arg2: type2,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_1>;
                fn test_method_2(
                    &self,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_2>;
            }
        };
        assert_tokens_equals(generate(&input), expected);
    }
}

#[cfg(test)]
mod generate_class_tests {
    use super::*;

    #[test]
    fn class() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
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
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn public() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: true,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
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
        let expected = quote! {
            #[derive(Debug)]
            pub struct test1<'env> {
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn transitive_extends() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![quote! {c::d::test2}, quote! {c::d::test3}],
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
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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

            impl<'a> ::rust_jni::Cast<'a, c::d::test3<'a>> for test1<'a> {
                #[doc(hidden)]
                fn cast<'b>(&'b self) -> &'b c::d::test3<'a> {
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn methods() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![
                    ClassMethod {
                        name: Ident::new("test_method_1", Span::call_site()),
                        java_name: Literal::string("testMethod1"),
                        return_type: quote! {return_type_1},
                        public: false,
                        argument_names: vec![
                            Ident::new("arg1", Span::call_site()),
                            Ident::new("arg2", Span::call_site()),
                        ],
                        argument_types: vec![quote! {type1}, quote! {type2}],
                    },
                    ClassMethod {
                        name: Ident::new("test_method_2", Span::call_site()),
                        java_name: Literal::string("testMethod2"),
                        return_type: quote! {return_type_2},
                        public: true,
                        argument_names: vec![],
                        argument_types: vec![],
                    },
                ],
                static_methods: vec![],
                native_methods: vec![],
                static_native_methods: vec![],
                constructors: vec![],
            })],
        };
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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

                fn test_method_1(
                    &self,
                    arg1: type1,
                    arg2: type2,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_1> {
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn(type1, type2,) -> return_type_1
                        >
                        (
                            self,
                            "testMethod1",
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                pub fn test_method_2(
                    &self,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_2> {
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn() -> return_type_2
                        >
                        (
                            self,
                            "testMethod2",
                            (),
                            token,
                        )
                    }
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn static_methods() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![
                    ClassMethod {
                        name: Ident::new("test_method_1", Span::call_site()),
                        java_name: Literal::string("testMethod1"),
                        return_type: quote! {return_type_1},
                        public: false,
                        argument_names: vec![
                            Ident::new("arg1", Span::call_site()),
                            Ident::new("arg2", Span::call_site()),
                        ],
                        argument_types: vec![quote! {type1}, quote! {type2}],
                    },
                    ClassMethod {
                        name: Ident::new("test_method_2", Span::call_site()),
                        java_name: Literal::string("testMethod2"),
                        return_type: quote! {return_type_2},
                        public: true,
                        argument_names: vec![],
                        argument_types: vec![],
                    },
                ],
                native_methods: vec![],
                static_native_methods: vec![],
                constructors: vec![],
            })],
        };
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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

                fn test_method_1(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: type1,
                    arg2: type2,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_1> {
                    unsafe {
                        ::rust_jni::__generator::call_static_method::<Self, _, _,
                            fn(type1, type2,) -> return_type_1
                        >
                        (
                            env,
                            "testMethod1",
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                pub fn test_method_2(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_2> {
                    unsafe {
                        ::rust_jni::__generator::call_static_method::<Self, _, _,
                            fn() -> return_type_2
                        >
                        (
                            env,
                            "testMethod2",
                            (),
                            token,
                        )
                    }
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn constructors() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![],
                native_methods: vec![],
                static_native_methods: vec![],
                constructors: vec![
                    Constructor {
                        name: Ident::new("test_method_1", Span::call_site()),
                        public: false,
                        argument_names: vec![
                            Ident::new("arg1", Span::call_site()),
                            Ident::new("arg2", Span::call_site()),
                        ],
                        argument_types: vec![quote! {type1}, quote! {type2}],
                    },
                    Constructor {
                        name: Ident::new("test_method_2", Span::call_site()),
                        public: true,
                        argument_names: vec![],
                        argument_types: vec![],
                    },
                ],
            })],
        };
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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

                fn test_method_1(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: type1,
                    arg2: type2,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, Self> {
                    unsafe {
                        ::rust_jni::__generator::call_constructor::<Self, _,
                            fn(type1, type2,)
                        >
                        (
                            env,
                            (arg1, arg2,),
                            token,
                        )
                    }
                }

                pub fn test_method_2(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, Self> {
                    unsafe {
                        ::rust_jni::__generator::call_constructor::<Self, _,
                            fn()
                        >
                        (
                            env,
                            (),
                            token,
                        )
                    }
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn native_methods() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![],
                native_methods: vec![
                    NativeMethod {
                        name: Ident::new("test_method_1", Span::call_site()),
                        rust_name: Ident::new("test_method_1_rust", Span::call_site()),
                        java_name: Ident::new("testMethod1", Span::call_site()),
                        return_type: quote! {return_type_1},
                        public: false,
                        argument_names: vec![
                            Ident::new("arg1", Span::call_site()),
                            Ident::new("arg2", Span::call_site()),
                        ],
                        argument_types: vec![quote! {type1<'a>}, quote! {type2<'a>}],
                        argument_types_no_lifetime: vec![quote! {type1}, quote! {type2}],
                        code: quote! {test code 1},
                    },
                    NativeMethod {
                        name: Ident::new("test_method_2", Span::call_site()),
                        rust_name: Ident::new("test_method_2_rust", Span::call_site()),
                        java_name: Ident::new("testMethod2", Span::call_site()),
                        return_type: quote! {return_type_2},
                        public: true,
                        argument_names: vec![],
                        argument_types: vec![],
                        argument_types_no_lifetime: vec![],
                        code: quote! {test code 2},
                    },
                ],
                static_native_methods: vec![],
                constructors: vec![],
            })],
        };
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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

                fn test_method_1_rust(
                    &self,
                    arg1: type1<'a>,
                    arg2: type2<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_1> {
                    test code 1
                }

                pub fn test_method_2_rust(
                    &self,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_2> {
                    test code 2
                }
            }

            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn testMethod1<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                object: ::jni_sys::jobject,
                arg1: <type1 as ::rust_jni::JavaType>::__JniType,
                arg2: <type2 as ::rust_jni::JavaType>::__JniType,
            ) -> <return_type_1 as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::test_jni_argument_type(arg1);
                ::rust_jni::__generator::test_jni_argument_type(arg2);
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    {
                        let value =
                            <type1 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg1);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }
                    {
                        let value =
                            <type2 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg2);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }

                    let object = <test1 as ::rust_jni::__generator::FromJni>::__from_jni(env, object);
                    object
                        .test_method_1_rust(
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
            pub unsafe extern "C" fn testMethod2<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                object: ::jni_sys::jobject,
            ) -> <return_type_2 as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    let object = <test1 as ::rust_jni::__generator::FromJni>::__from_jni(env, object);
                    object
                        .test_method_2_rust(
                            &token,
                        )
                        .map(|value| {
                            let result = ::rust_jni::__generator::ToJni::__to_jni(&value);
                            ::std::mem::forget(value);
                            result
                        })
                })
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn static_native_methods() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![],
                native_methods: vec![],
                static_native_methods: vec![
                    NativeMethod {
                        name: Ident::new("test_method_1", Span::call_site()),
                        rust_name: Ident::new("test_method_1_rust", Span::call_site()),
                        java_name: Ident::new("testMethod1", Span::call_site()),
                        return_type: quote! {return_type_1},
                        public: false,
                        argument_names: vec![
                            Ident::new("arg1", Span::call_site()),
                            Ident::new("arg2", Span::call_site()),
                        ],
                        argument_types: vec![quote! {type1<'a>}, quote! {type2<'a>}],
                        argument_types_no_lifetime: vec![quote! {type1}, quote! {type2}],
                        code: quote! {test code 1},
                    },
                    NativeMethod {
                        name: Ident::new("test_method_2", Span::call_site()),
                        rust_name: Ident::new("test_method_2_rust", Span::call_site()),
                        java_name: Ident::new("testMethod2", Span::call_site()),
                        return_type: quote! {return_type_2},
                        public: true,
                        argument_names: vec![],
                        argument_types: vec![],
                        argument_types_no_lifetime: vec![],
                        code: quote! {test code 2},
                    },
                ],
                constructors: vec![],
            })],
        };
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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

                fn test_method_1_rust(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    arg1: type1<'a>,
                    arg2: type2<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_1> {
                    test code 1
                }

                pub fn test_method_2_rust(
                    env: &'a ::rust_jni::JniEnv<'a>,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_2> {
                    test code 2
                }
            }

            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn testMethod1<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                raw_class: ::jni_sys::jclass,
                arg1: <type1 as ::rust_jni::JavaType>::__JniType,
                arg2: <type2 as ::rust_jni::JavaType>::__JniType,
            ) -> <return_type_1 as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::test_jni_argument_type(arg1);
                ::rust_jni::__generator::test_jni_argument_type(arg2);
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    {
                        let value =
                            <type1 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg1);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }
                    {
                        let value =
                            <type2 as ::rust_jni::__generator::FromJni>
                                ::__from_jni(env, arg2);
                        ::rust_jni::__generator::test_from_jni_type(&value);
                        ::std::mem::forget(value);
                    }

                    let class = test1::get_class(env, &token)?;
                    let raw_class = <::rust_jni::java::lang::Class as ::rust_jni::__generator::FromJni>::__from_jni(env, raw_class);
                    if !class.is_same_as(&raw_class, &token) {
                        panic!("Native method test_method_1 does not belong to class test1");
                    }

                    test1::test_method_1_rust(
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
            pub unsafe extern "C" fn testMethod2<'a>(
                raw_env: *mut ::jni_sys::JNIEnv,
                raw_class: ::jni_sys::jclass,
            ) -> <return_type_2 as ::rust_jni::JavaType>::__JniType {
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    let class = test1::get_class(env, &token)?;
                    let raw_class = <::rust_jni::java::lang::Class as ::rust_jni::__generator::FromJni>::__from_jni(env, raw_class);
                    if !class.is_same_as(&raw_class, &token) {
                        panic!("Native method test_method_2 does not belong to class test1");
                    }

                    test1::test_method_2_rust(
                        env,
                        &token,
                    )
                    .map(|value| {
                        let result = ::rust_jni::__generator::ToJni::__to_jni(&value);
                        ::std::mem::forget(value);
                        result
                    })
                })
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn implements() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![
                    InterfaceImplementation {
                        interface: quote! {e::f::test3},
                        methods: vec![],
                    },
                    InterfaceImplementation {
                        interface: quote! {e::f::test4},
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
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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
        assert_tokens_equals(generate(&input), expected);
    }

    #[test]
    fn implements_with_methods() {
        let input = GeneratorData {
            definitions: vec![GeneratorDefinition::Class(Class {
                class: Ident::new("test1", Span::call_site()),
                public: false,
                super_class: quote! {c::d::test2},
                transitive_extends: vec![],
                implements: vec![InterfaceImplementation {
                    interface: quote! {e::f::test3},
                    methods: vec![
                        InterfaceMethodImplementation {
                            name: Ident::new("test_method_1", Span::call_site()),
                            return_type: quote! {return_type_1},
                            argument_names: vec![
                                Ident::new("arg1", Span::call_site()),
                                Ident::new("arg2", Span::call_site()),
                            ],
                            argument_types: vec![quote! {type1}, quote! {type2}],
                            class_has_method: false,
                        },
                        InterfaceMethodImplementation {
                            name: Ident::new("test_method_2", Span::call_site()),
                            return_type: quote! {return_type_2},
                            argument_names: vec![],
                            argument_types: vec![],
                            class_has_method: true,
                        },
                    ],
                }],
                signature: Literal::string("test/sign1"),
                full_signature: Literal::string("test/signature1"),
                methods: vec![],
                static_methods: vec![],
                native_methods: vec![],
                static_native_methods: vec![],
                constructors: vec![],
            })],
        };
        let expected = quote! {
            #[derive(Debug)]
            struct test1<'env> {
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
                fn test_method_1(
                    &self,
                    arg1: type1,
                    arg2: type2,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_1> {
                    <c::d::test2 as e::f::test3>::test_method_1(
                        self, arg1, arg2, token
                    )
                }

                fn test_method_2(
                    &self,
                    token: &::rust_jni::NoException<'a>,
                ) -> ::rust_jni::JavaResult<'a, return_type_2> {
                    Self::test_method_2(
                        self, token
                    )
                }
            }
        };
        assert_tokens_equals(generate(&input), expected);
    }
}
