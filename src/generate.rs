// TODO: improve these macros to exclude some repetition at call-site.
// For example, investigate https://docs.rs/macro-attr.

// TODO: use public rustdoc links for `rust-jni` types.

// TODO(https://github.com/rust-lang/rust/issues/29599): use concat_idents!($class, Interface)
// to not make the user specify the interface name.

// TODO(https://github.com/rust-lang/rust/issues/29661): remove $super_super_class once default
// associated types are stabilized.

// TODO: invent some way to not repeat the interface declaration in class declarations.

/// Generate Java interface wrapper.
#[macro_export]
macro_rules! java_interface {
    (
        interface = $interface:ident,
        link = $link:expr,
        extends = ($($($extended_interface:ident)::+),*),
        methods = ($(
            doc = $method_documentation:expr,
            link = $method_link:expr,
            java_name = $java_method_name:expr,
            $method_name:ident
                ($($method_argument_name:ident: $method_argument_type:ty),*)
                -> $method_result:ty,
        )*),
    ) => {
        /// Rust wrapper type for the
        #[doc = $link]
        /// Java interface.
        pub trait $interface<'env>: $($($extended_interface)::* <'env>),* {
            $(
                #[doc = $method_documentation]
                ///
                #[doc = $method_link]
                fn $method_name(
                    &self,
                    $($method_argument_name: $method_argument_type,)*
                    token: &::rust_jni::NoException<'env>,
                ) -> ::rust_jni::JavaResult<'env, $method_result>;
            )*
        }
    };
}

/// Helper macro.
///
/// DO NOT USE MANUALLY!
#[doc(hidden)]
#[macro_export]
macro_rules! __generate_super_interface {
    (
        class = $class:ident,
        class_link = $class_link:expr,
        interface_link = $implemented_interface_link:expr,
        extends = $super_class:path,
        name = $($implemented_interface:ident)::+,
        name_path = $implemented_interface_path:path,
        methods = ($(
            $interface_method_name:ident
                ($(
                    $interface_method_argument_name:ident:
                    $interface_method_argument_type:ty
                ),*) -> $interface_method_result:ty,
        )*),
    ) => {
        /// Implement
        #[doc = $implemented_interface_link]
        /// interface for
        #[doc = $class_link]
        ///.
        impl<'env> $($implemented_interface)::* <'env> for $class<'env> {
            $(
                fn $interface_method_name(
                    &self,
                    $($interface_method_argument_name: $interface_method_argument_type),*,
                    token: &::rust_jni::NoException<'env>,
                ) -> ::rust_jni::JavaResult<'env, $interface_method_result> {
                    <$super_class as $implemented_interface>
                        ::$interface_method_name(
                            self, $($interface_method_argument_name),*, token
                        )
                }
            )*
        }
    };
}

/// Helper macro.
///
/// DO NOT USE MANUALLY!
#[macro_export]
#[doc(hidden)]
macro_rules! __generate_super_interfaces {
    (
        class = $class:ident,
        class_link = $class_link:expr,
        extends = $super_class:path,
        interfaces = ($(
            link = $implemented_interface_link:expr,
            name = $($implemented_interface:ident)::+,
            methods = ($(
                $interface_method_name:ident
                    ($(
                        $interface_method_argument_name:ident:
                        $interface_method_argument_type:ty
                    ),*) -> $interface_method_result:ty,
            )*),
        )*),
    ) => {
        $(
            __generate_super_interface!(
                class = $class,
                class_link = $class_link,
                interface_link = $implemented_interface_link,
                extends = $super_class,
                name = $($implemented_interface)::+,
                name_path = $($implemented_interface)::+,
                methods = ($(
                    $interface_method_name
                        ($(
                            $interface_method_argument_name:
                            $interface_method_argument_type
                        ),*) -> $interface_method_result,
                )*),
            );
        )*
    };
}

/// Generate Java class wrapper.
#[macro_export]
macro_rules! java_class {
    (
        package = $package:expr,
        class = $class:ident,
        link = $java_link:expr,
        rust_link = $rust_link:expr,
        extends = $($super_class:ident)::+,
        super_link = $rust_super_link:expr,
        implements = ($(
            name = $($implemented_interface:ident)::+,
            link = $implemented_interface_link:expr,
            methods = ($(
                $interface_method_name:ident
                    ($(
                        $interface_method_argument_name:ident:
                        $interface_method_argument_type:ty
                    ),*) -> $interface_method_result:ty,
            )*),
        )*),
        constructors = ($(
            doc = $constructor_documentation:expr,
            link = $constructor_link:expr,
            $constructor_name:ident
                ($($constructor_argument_name:ident: $constructor_argument_type:ty),*),
        )*),
        methods = ($(
            doc = $method_documentation:expr,
            link = $method_link:expr,
            java_name = $java_method_name:expr,
            $method_name:ident
                ($($method_argument_name:ident: $method_argument_type:ty),*)
                -> $method_result:ty,
        )*),
        static_methods = ($(
            doc = $static_method_documentation:expr,
            link = $static_method_link:expr,
            java_name = $java_static_method_name:expr,
            $static_method_name:ident
                ($($static_method_argument_name:ident: $static_method_argument_type:ty),*)
                -> $static_method_result:ty,
        )*),
        native_methods = ($(
            function_name = $native_function_name:ident,
            $native_method_name:ident
                ($($native_method_argument_name:ident: $native_method_argument_type:ty),*)
                -> $native_method_result:ty,
        )*),
        static_native_methods = ($(
            function_name = $static_native_function_name:ident,
            $static_native_method_name:ident
                ($($static_native_method_argument_name:ident: $static_native_method_argument_type:ty),*)
                -> $static_native_method_result:ty,
        )*),
        super_classes = ($(
            $($super_super_class:ident)::+,
            link = $super_super_class_link:expr
        ),*),
        super_interfaces = ($(
            name = $($implemented_super_interface:ident)::+,
            link = $implemented_super_interface_link:expr,
            methods = ($(
                $super_interface_method_name:ident
                    ($(
                        $super_interface_method_argument_name:ident:
                        $super_interface_method_argument_type:ty
                    ),*) -> $super_interface_method_result:ty,
            )*),
        )*),
    ) => {
        /// Rust wrapper type for the
        #[doc = $java_link]
        /// Java class.
        #[derive(Debug)]
        pub struct $class<'env> {
            object: $($super_class)::* <'env>,
        }

        // Tools for mapping Rust and JNI types.

        /// Make
        #[doc = $rust_link]
        /// mappable to [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'a> ::rust_jni::JavaType for $class<'a> {
            #[doc(hidden)]
            type __JniType = <::rust_jni::java::lang::Object<'a> as ::rust_jni::JavaType>::__JniType;

            #[doc(hidden)]
            fn __signature() -> &'static str {
                concat!("L", $package, "/", stringify!($class), ";")
            }
        }

        /// Make
        #[doc = $rust_link]
        /// convertible to [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        #[doc(hidden)]
        impl<'a> ::rust_jni::__generator::ToJni for $class<'a> {
            unsafe fn __to_jni(&self) -> Self::__JniType {
                self.raw_object()
            }
        }

        /// Make
        #[doc = $rust_link]
        /// convertible from [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        #[doc(hidden)]
        impl<'env> ::rust_jni::__generator::FromJni<'env> for $class<'env> {
            unsafe fn __from_jni(env: &'env ::rust_jni::JniEnv<'env>, value: Self::__JniType) -> Self {
                Self {
                    object: <$($super_class)::* as ::rust_jni::__generator::FromJni<'env>>::__from_jni(env, value),
                }
            }
        }

        // Tools for the object hierarchy.

        /// Make
        #[doc = $rust_link]
        /// castable to itself.
        impl<'env> ::rust_jni::Cast<'env, $class<'env>> for $class<'env> {
            #[doc(hidden)]
            fn cast<'a>(&'a self) -> &'a $class<'env> {
                self
            }
        }

        /// Make
        #[doc = $rust_link]
        /// castable to it's superclass
        #[doc = $rust_super_link]
        ///.
        impl<'env> ::rust_jni::Cast<'env, $($super_class)::*<'env>> for $class<'env> {
            #[doc(hidden)]
            fn cast<'a>(&'a self) -> &'a $($super_class)::*<'env> {
                self
            }
        }

        $(
            /// Make
            #[doc = $rust_link]
            /// castable to it's superclass
            #[doc = $super_super_class_link]
            ///.
            impl<'env> ::rust_jni::Cast<'env, $($super_super_class)::* <'env>> for $class<'env> {
                #[doc(hidden)]
                fn cast<'a>(&'a self) -> &'a $($super_super_class)::* <'env> {
                    self
                }
            }
        )*

        /// Allow
        #[doc = $rust_link]
        /// to be used in place of it's superclass
        #[doc = $rust_super_link]
        ///.
        ///
        /// Since all Java class wrappers implement
        /// [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html), to superclass,
        /// this works transitively for all parent classes.
        impl<'env> ::std::ops::Deref for $class<'env> {
            type Target = $($super_class)::* <'env>;

            fn deref(&self) -> &Self::Target {
                &self.object
            }
        }

        impl<'env> $class<'env> {
            // Non-Java class methods.

            /// Get the Java class object for
            #[doc = $rust_link]
            ///.
            ///
            /// [`Object::getClass` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#getClass())
            pub fn get_class(env: &'env ::rust_jni::JniEnv<'env>, token: &::rust_jni::NoException<'env>)
                -> ::rust_jni::JavaResult<'env, ::rust_jni::java::lang::Class<'env>> {
                ::rust_jni::java::lang::Class::find(env, concat!($package, "/", stringify!($class)), token)
            }

            /// Clone the
            #[doc = $rust_link]
            ///. This is not a deep clone of the Java object,
            /// but a Rust-like clone of the value. Since Java objects are reference counted, this
            /// will increment the reference count.
            ///
            /// This method has a different signature from the one in the
            /// [`Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html) trait
            /// because cloning a Java object is only safe when there is no pending exception and
            /// because cloning a java object cat throw an exception.
            ///
            /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/functions.html#newlocalref)
            pub fn clone(&self, token: &::rust_jni::NoException<'env>) -> ::rust_jni::JavaResult<'env, Self>
            where
                Self: Sized,
            {
                <Self as ::rust_jni::Cast<$($super_class)::*>>::cast(self)
                    .clone(token)
                    .map(|object| Self { object })
            }

            /// Convert the object to a string.
            ///
            /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
            //
            // This function is needed because Java classes implement
            // [`ToString`](https://doc.rust-lang.org/std/string/trait.ToString.html) trait through
            // the [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) trait, which
            // has a
            // [`to_string`](https://doc.rust-lang.org/std/string/trait.ToString.html#tymethod.to_string)
            // method which takes precedence over methods inherited from
            // [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html)-s.
            pub fn to_string(&self, token: &::rust_jni::NoException<'env>)
                -> ::rust_jni::JavaResult<'env, ::rust_jni::java::lang::String<'env>> {
                <$class as ::rust_jni::Cast<::rust_jni::java::lang::Object>>::cast(self).to_string(token)
            }

            // Declared methods.

            $(
                #[doc = $constructor_documentation]
                ///
                #[doc = $constructor_link]
                pub fn $constructor_name(
                    env: &'env ::rust_jni::JniEnv<'env>,
                    $($constructor_argument_name: $constructor_argument_type,)*
                    token: &::rust_jni::NoException<'env>,
                ) -> ::rust_jni::JavaResult<'env, Self> {
                    // Safe because method arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_constructor::<Self, _, fn($($constructor_argument_type,)*)>
                        (
                            env,
                            ($($constructor_argument_name,)*),
                            token,
                        )
                    }
                }
            )*

            $(
                #[doc = $method_documentation]
                ///
                #[doc = $method_link]
                pub fn $method_name(
                    &self,
                    $($method_argument_name: $method_argument_type,)*
                    token: &::rust_jni::NoException<'env>,
                ) -> ::rust_jni::JavaResult<'env, $method_result> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_method::<_, _, _,
                            fn($($method_argument_type,)*) -> $method_result
                        >
                        (
                            self,
                            $java_method_name,
                            ($($method_argument_name,)*),
                            token,
                        )
                    }
                }
            )*

            $(
                #[doc = $static_method_documentation]
                ///
                #[doc = $static_method_link]
                pub fn $static_method_name(
                    env: &'env ::rust_jni::JniEnv<'env>,
                    $($static_method_argument_name: $static_method_argument_type,)*
                    token: &::rust_jni::NoException<'env>,
                ) -> ::rust_jni::JavaResult<'env, $static_method_result> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        ::rust_jni::__generator::call_static_method::<Self, _, _,
                            fn($($static_method_argument_type,)*) -> $static_method_result
                        >
                        (
                            env,
                            $java_static_method_name,
                            ($($static_method_argument_name,)*),
                            token,
                        )
                    }
                }
            )*
        }

        $(
            /// Implement
            #[doc = $implemented_interface_link]
            /// interface for
            #[doc = $rust_link]
            ///.
            impl<'env> $($implemented_interface)::* <'env> for $class<'env> {
                $(
                    fn $interface_method_name(
                        &self,
                        $($interface_method_argument_name: $interface_method_argument_type),*,
                        token: &::rust_jni::NoException<'env>,
                    ) -> ::rust_jni::JavaResult<'env, $interface_method_result> {
                            Self::$interface_method_name(
                                self, $($interface_method_argument_name),*, token
                            )
                    }
                )*
            }
        )*

        __generate_super_interfaces!(
            class = $class,
            class_link = $rust_link,
            extends = $($super_class)::+,
            interfaces = ($(
                link = $implemented_super_interface_link,
                name = $($implemented_super_interface)::+,
                methods = ($(
                    $super_interface_method_name
                        ($(
                            $super_interface_method_argument_name:
                            $super_interface_method_argument_type
                        ),*) -> $super_interface_method_result,
                )*),
            )*),
        );

        // Native method stubs.

        $(
            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn $native_function_name(
                raw_env: *mut ::jni_sys::JNIEnv,
                object: ::jni_sys::jobject,
                $($native_method_argument_name: <$native_method_argument_type as ::rust_jni::JavaType>::__JniType,)*
            ) -> <$native_method_result as ::rust_jni::JavaType>::__JniType {
                // TODO: make sure `$native_method_result: ::rust_jni::__generator::FromJni`.
                // Compile-time check that declared arguments implement the `JniArgumentType`
                // trait.
                $(::rust_jni::__generator::test_jni_argument_type($native_method_argument_name);)*
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    // Compile-time check that declared arguments implement the `FromJni` trait.
                    $(
                        {
                            let value =
                                <$native_method_argument_type as ::rust_jni::__generator::FromJni>
                                    ::__from_jni(env, $native_method_argument_name);
                            ::rust_jni::__generator::test_from_jni_type(&value);
                            ::std::mem::forget(value);
                        }
                    )*

                    let object = <$class as ::rust_jni::__generator::FromJni>::__from_jni(env, object);
                    object
                        .$native_method_name(
                            $(::rust_jni::__generator::FromJni::__from_jni(env, $native_method_argument_name),)*
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
        )*

        $(
            #[no_mangle]
            #[doc(hidden)]
            pub unsafe extern "C" fn $static_native_function_name(
                raw_env: *mut ::jni_sys::JNIEnv,
                raw_class: ::jni_sys::jclass,
                $($static_native_method_argument_name: <$static_native_method_argument_type as ::rust_jni::JavaType>::__JniType,)*
            ) -> <$static_native_method_result as ::rust_jni::JavaType>::__JniType {
                // TODO: make sure `$native_method_result: ::rust_jni::__generator::FromJni`.
                // Compile-time check that declared arguments implement the `JniArgumentType`
                // trait.
                $(::rust_jni::__generator::test_jni_argument_type($static_native_method_argument_name);)*
                ::rust_jni::__generator::native_method_wrapper(raw_env, |env, token| {
                    // Compile-time check that declared arguments implement the `FromJni` trait.
                    $(
                        {
                            let value =
                                <$static_native_method_argument_type as ::rust_jni::__generator::FromJni>
                                    ::__from_jni(env, $static_native_method_argument_name);
                            ::rust_jni::__generator::test_from_jni_type(&value);
                            ::std::mem::forget(value);
                        }
                    )*

                    let class = $class::get_class(env, &token)?;
                    let raw_class = <::rust_jni::java::lang::Class as ::rust_jni::__generator::FromJni>::__from_jni(env, raw_class);
                    if !class.is_same_as(&raw_class, &token) {
                        // This should never happen, as native method's link name has the class,
                        // so it must be bound to a correct clas by the JVM.
                        // Still, this is a good test to ensure that the system
                        // is in a consistent state.
                        panic!(concat!(
                            "Native method ",
                            stringify!($static_native_function_name),
                            " does not belong to class ",
                            $package, "/", stringify!($class),
                        ));
                    }

                    $class::$static_native_method_name(
                        env,
                        $(::rust_jni::__generator::FromJni::__from_jni(env, $static_native_method_argument_name),)*
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
        )*

        // Common traits for convenience.

        /// Allow displaying
        #[doc = $rust_link]
        ///.
        ///
        /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
        ///
        /// This is mostly a convenience for debugging. Always prefer using
        /// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env> ::std::fmt::Display for $class<'env> {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                <$class as ::rust_jni::Cast<::rust_jni::java::lang::Object>>::cast(self)
                    .fmt(formatter)
            }
        }

        /// Allow comparing
        #[doc = $rust_link]
        /// to Java objects. Java objects are compared by-reference to preserve
        /// original Java semantics. To compare objects by value, call the
        /// [`equals`](struct.Object.html#method.equals) method.
        ///
        /// Will panic if there is a pending exception in the current thread.
        ///
        /// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
        /// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env, T> PartialEq<T> for $class<'env> where T: ::rust_jni::Cast<'env, ::rust_jni::java::lang::Object<'env>> {
            fn eq(&self, other: &T) -> bool {
                <$class as ::rust_jni::Cast<::rust_jni::java::lang::Object>>::cast(self)
                    .eq(other)
            }
        }

        /// Allow comparing
        #[doc = $rust_link]
        /// to Java objects. Java objects are compared by-reference to preserve
        /// original Java semantics. To compare objects by value, call the
        /// [`equals`](struct.Object.html#method.equals) method.
        ///
        /// Will panic if there is a pending exception in the current thread.
        ///
        /// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
        /// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env> Eq for $class<'env> {}
    };
}
