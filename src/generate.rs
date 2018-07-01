// TODO: improve these macros to exclude some repetition at call-site.
// For example, investigate https://docs.rs/macro-attr.

// TODO: use public rustdoc links for `rust-jni` types.

// TODO(https://github.com/rust-lang/rust/issues/29599): use concat_idents!($class, Interface)
// to not make the user specify the interface name.

// TODO(https://github.com/rust-lang/rust/issues/29661): remove $super_super_class once default
// associated types are stabilized.

// TODO: invent some way to not repeat the interface declaration in class declarations.

/// Generate Java class mapping.
#[macro_export]
macro_rules! java_class {
    (
        package = $package:expr,
        class = $class:ident,
        java_link = $java_link:expr,
        rust_link = $rust_link:expr,
        extends = $($super_class:ident)::+,
        super_link = $rust_super_link:expr,
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
        super_classes = ($(
            $($super_super_class:ident)::+,
            link = $super_super_class_link:expr,
        ),*),
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
