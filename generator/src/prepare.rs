use generate::{self, GeneratorData, GeneratorDefinition};
use java_name::*;
use parse::*;
use proc_macro2::*;
use std::collections::{HashMap, HashSet};
use std::iter;

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

fn annotation_value(annotations: &[Annotation], name: &str) -> Option<TokenStream> {
    let values = annotations
        .iter()
        .filter_map(|annotation| {
            if annotation.name == name.to_string() {
                Some(annotation.value.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if values.len() > 1 {
        panic!(
            "Only one @{} annotation per definition can be provided.",
            name
        );
    }
    if values.is_empty() {
        None
    } else {
        Some(values[0].clone())
    }
}

fn annotation_value_ident(annotations: &[Annotation], name: &str) -> Option<Ident> {
    annotation_value(annotations, name).map(|value| match value.into_iter().next().unwrap() {
        TokenTree::Ident(identifier) => identifier,
        _ => unreachable!(),
    })
}

fn to_generator_method(method: JavaClassMethod) -> generate::ClassMethod {
    let JavaClassMethod {
        name,
        public,
        return_type,
        arguments,
        annotations,
        ..
    } = method;
    let java_name = Literal::string(&name.to_string());
    generate::ClassMethod {
        name: annotation_value_ident(&annotations, "RustName").unwrap_or(name),
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

fn to_generator_interface_method(method: JavaInterfaceMethod) -> generate::InterfaceMethod {
    let JavaInterfaceMethod {
        name,
        return_type,
        arguments,
        annotations,
        ..
    } = method;
    generate::InterfaceMethod {
        name: annotation_value_ident(&annotations, "RustName").unwrap_or(name),
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
) -> generate::InterfaceMethodImplementation {
    let JavaInterfaceMethod {
        name,
        return_type,
        arguments,
        annotations,
        ..
    } = method;
    let class_has_method = class_methods.iter().any(|class_method| {
        class_method.name == name
            && class_method.return_type == return_type
            && class_method.arguments == arguments
    });
    let interface = interface.clone().with_double_colons();
    generate::InterfaceMethodImplementation {
        name: annotation_value_ident(&annotations, "RustName").unwrap_or(name),
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
) -> generate::NativeMethod {
    let JavaNativeMethod {
        name,
        public,
        return_type,
        arguments,
        code,
        annotations,
        ..
    } = method;
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
    let rust_name = annotation_value_ident(&annotations, "RustName").unwrap_or(name.clone());
    generate::NativeMethod {
        name,
        rust_name,
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

fn to_generator_constructor(constructor: JavaConstructor) -> generate::Constructor {
    let JavaConstructor {
        public,
        arguments,
        annotations,
        ..
    } = constructor;
    let name = Ident::new("init", Span::call_site());
    generate::Constructor {
        name: annotation_value_ident(&annotations, "RustName").unwrap_or(name),
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

pub fn to_generator_data(definitions: JavaDefinitions) -> GeneratorData {
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
                            .map(|name| generate::InterfaceImplementation {
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
                        GeneratorDefinition::Class(generate::Class {
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
                        GeneratorDefinition::Interface(generate::Interface {
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
        assert_generator_data_equals(
            to_generator_data(JavaDefinitions {
                definitions: vec![],
                metadata: Metadata {
                    definitions: vec![],
                },
            }),
            GeneratorData {
                definitions: vec![],
            },
        );
    }

    #[test]
    fn metadata_only() {
        assert_generator_data_equals(
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
            },
        );
    }

    #[test]
    fn one_class() {
        assert_generator_data_equals(
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
                definitions: vec![GeneratorDefinition::Class(generate::Class {
                    class: Ident::new("test1", Span::call_site()),
                    public: false,
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
            },
        );
    }

    #[test]
    fn one_class_no_extends() {
        assert_generator_data_equals(
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
                definitions: vec![GeneratorDefinition::Class(generate::Class {
                    class: Ident::new("test1", Span::call_site()),
                    public: false,
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
            },
        );
    }

    #[test]
    fn one_class_extends_recursive() {
        assert_generator_data_equals(
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
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test2", Span::call_site()),
                        public: false,
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
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test1", Span::call_site()),
                        public: false,
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
            },
        );
    }

    #[test]
    fn one_class_implements() {
        assert_generator_data_equals(
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
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test4", Span::call_site()),
                        public: false,
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test1", Span::call_site()),
                        public: false,
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![
                            generate::InterfaceImplementation {
                                interface: quote!{::e::f::test3},
                                methods: vec![],
                            },
                            generate::InterfaceImplementation {
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
            },
        );
    }

    #[test]
    fn one_class_implements_recursive() {
        assert_generator_data_equals(
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
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test3", Span::call_site()),
                        public: false,
                        extends: vec![quote!{::e::f::test4}],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test1", Span::call_site()),
                        public: false,
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![
                            generate::InterfaceImplementation {
                                interface: quote!{::e::f::test3},
                                methods: vec![],
                            },
                            generate::InterfaceImplementation {
                                interface: quote!{::e::f::test4},
                                methods: vec![],
                            },
                            generate::InterfaceImplementation {
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
            },
        );
    }

    #[test]
    fn one_class_implements_recursive_duplicated() {
        assert_generator_data_equals(
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
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test4", Span::call_site()),
                        public: false,
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test3", Span::call_site()),
                        public: false,
                        extends: vec![quote!{::g::h::test4}],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test1", Span::call_site()),
                        public: false,
                        super_class: quote!{::java::lang::Object},
                        transitive_extends: vec![quote!{::java::lang::Object}],
                        implements: vec![
                            generate::InterfaceImplementation {
                                interface: quote!{::e::f::test3},
                                methods: vec![],
                            },
                            generate::InterfaceImplementation {
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
            },
        );
    }

    #[test]
    fn one_class_public() {
        assert_generator_data_equals(
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
                definitions: vec![GeneratorDefinition::Class(generate::Class {
                    class: Ident::new("test1", Span::call_site()),
                    public: true,
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
            },
        );
    }

    #[test]
    fn one_interface() {
        assert_generator_data_equals(
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
                definitions: vec![GeneratorDefinition::Interface(generate::Interface {
                    interface: Ident::new("test1", Span::call_site()),
                    public: false,
                    extends: vec![],
                    methods: vec![],
                })],
            },
        );
    }

    #[test]
    fn one_interface_extends() {
        assert_generator_data_equals(
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
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test3", Span::call_site()),
                        public: false,
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test1", Span::call_site()),
                        public: false,
                        extends: vec![quote!{::c::d::test2}, quote!{::e::f::test3}],
                        methods: vec![],
                    }),
                ],
            },
        );
    }

    #[test]
    fn one_interface_public() {
        assert_generator_data_equals(
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
                definitions: vec![GeneratorDefinition::Interface(generate::Interface {
                    interface: Ident::new("test1", Span::call_site()),
                    public: true,
                    extends: vec![],
                    methods: vec![],
                })],
            },
        );
    }

    #[test]
    fn multiple() {
        assert_generator_data_equals(
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
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test_if1", Span::call_site()),
                        public: false,
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Interface(generate::Interface {
                        interface: Ident::new("test_if2", Span::call_site()),
                        public: false,
                        extends: vec![],
                        methods: vec![],
                    }),
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test1", Span::call_site()),
                        public: false,
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
                    GeneratorDefinition::Class(generate::Class {
                        class: Ident::new("test2", Span::call_site()),
                        public: false,
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
            },
        );
    }
}

#[cfg(test)]
fn assert_generator_data_equals(left: GeneratorData, right: GeneratorData) {
    assert_eq!(format!("{:?}", left), format!("{:?}", right),);
}
