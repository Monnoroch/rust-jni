/// A macro for generating Java class boilerplate for Rust types, whcih is suitable for
/// `Object` type.
macro_rules! object_java_class {
    (
        $class:ident, $link:expr,
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
        )*
    ),
    ) => {
        /// Make
        #[doc = $link]
        /// convertible to [`jobject`](https://docs.rs/jni-sys/0.3.0/jni_sys/type.jobject.html).
        impl<'a> ToJni for $class<'a> {
            #[doc(hidden)]
            type JniType = jni_sys::jobject;

            #[doc(hidden)]
            fn signature() -> &'static str {
                <Self as JavaClassType>::signature()
            }

            unsafe fn to_jni(&self) -> Self::JniType {
                Object::cast(self).raw_object()
            }
        }

        /// Make
        #[doc = $link]
        /// castable to itself.
        impl<'env> Cast<'env, $class<'env>> for $class<'env> {
            fn cast<'a>(&'a self) -> &'a $class<'env> {
                self
            }
        }

        /// Allow comparing
        #[doc = $link]
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

        impl<'env> $class<'env> {
            /// Get the Java class object for
            #[doc = $link]
            ///.
            ///
            /// [`Object::getClass` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#getClass())
            pub fn get_class(env: &'env JniEnv<'env>, token: &NoException<'env>)
                -> JavaResult<'env, Class<'env>> {
                Class::find(env, concat!("java/lang", "/", stringify!($class)), token)
            }

            $(
                #[doc = $constructor_documentation]
                ///
                #[doc = $constructor_link]
                pub fn $constructor_name(
                    env: &'env JniEnv<'env>,
                    $($constructor_argument_name: $constructor_argument_type,)*
                    token: &NoException<'env>,
                ) -> JavaResult<'env, Self> {
                    // Safe because method arguments are correct.
                    unsafe {
                        call_constructor::<Self, _, fn($($constructor_argument_type,)*)>
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
                    token: &NoException<'env>,
                ) -> JavaResult<'env, $method_result> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        call_method::<Self, _, _,
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
        }
    };
}

// It's actually used.
#[allow(unused_macros)]
/// A macro for generating Java class boilerplate for Rust types, except for the `Object` type.
macro_rules! java_class {
    (
        $class:ident, $link:expr,
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
    ) => {
        object_java_class!(
            $class, $link,
            constructors = ($(
                doc = $constructor_documentation,
                link = $constructor_link,
                $constructor_name
                    ($($constructor_argument_name: $constructor_argument_type),*),
            )*),
            methods = ($(
                doc = $method_documentation,
                link = $method_link,
                java_name = $java_method_name,
                $method_name
                    ($($method_argument_name: $method_argument_type),*)
                    -> $method_result,
            )*),
        );

        /// Make
        #[doc = $link]
        /// convertible from [`Object`](struct.Object.html).
        impl<'env> JavaClassType<'env> for $class<'env> {
            #[doc(hidden)]
            fn signature() -> &'static str {
                concat!("L", "java/lang", "/", stringify!($class), ";")
            }

            #[doc(hidden)]
            fn from_object(object: Object<'env>) -> Self {
                Self {
                    object,
                }
            }
        }

        /// Allow
        #[doc = $link]
        /// to be used in place of an [`Object`](struct.Object.html).
        impl<'env> ::std::ops::Deref for $class<'env> {
            type Target = Object<'env>;

            fn deref(&self) -> &Self::Target {
                &self.object
            }
        }

        /// Make
        #[doc = $link]
        /// castable to [`Object`](struct.Object.html).
        impl<'env> Cast<'env, Object<'env>> for $class<'env> {
            fn cast<'a>(&'a self) -> &'a Object<'env> {
                self
            }
        }

        /// Allow comparing
        #[doc = $link]
        /// to Java objects. Java objects are compared by-reference to preserve
        /// original Java semantics. To compare objects by value, call the
        /// [`equals`](struct.Object.html#method.equals) method.
        ///
        /// Will panic if there is a pending exception in the current thread.
        ///
        /// This is mostly a convenience for using `assert_eq!()` in tests. Always prefer using
        /// [`is_same_as`](struct.Object.html#methods.is_same_as) to comparing with `==`, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env, T> PartialEq<T> for $class<'env> where T: Cast<'env, Object<'env>> {
            fn eq(&self, other: &T) -> bool {
                Object::cast(self).eq(other)
            }
        }

        /// Allow displaying
        #[doc = $link]
        ///.
        ///
        /// [`Object::toString` javadoc](https://docs.oracle.com/javase/10/docs/api/java/lang/Object.html#toString())
        ///
        /// This is mostly a convenience for debugging. Always prefer using
        /// [`to_string`](struct.Object.html#methods.to_string) to printing the object as is, because
        /// the former checks for a pending exception in compile-time rather than the run-time.
        impl<'env> fmt::Display for $class<'env> {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                Object::cast(self).fmt(formatter)
            }
        }

        impl<'env> $class<'env> {
            /// Clone the
            #[doc = $link]
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
            pub fn clone(&self, token: &NoException<'env>) -> JavaResult<'env, Self>
            where
                Self: Sized,
            {
                self.object
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
            pub fn to_string(&self, token: &NoException<'env>) -> JavaResult<'env, String<'env>> {
                Object::cast(self).to_string(token)
            }

            $(
                #[doc = $static_method_documentation]
                ///
                #[doc = $static_method_link]
                pub fn $static_method_name(
                    env: &'env JniEnv<'env>,
                    $($static_method_argument_name: $static_method_argument_type,)*
                    token: &NoException<'env>,
                ) -> JavaResult<'env, $static_method_result> {
                    // Safe because the method name and arguments are correct.
                    unsafe {
                        call_static_method::<Self, _, _,
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
    };
}

// It's actually used.
#[allow(unused_macros)]
#[cfg(test)]
macro_rules! generate_object_tests {
    ($class:ident, $signature:expr) => {
        #[test]
        fn signature() {
            assert_eq!(<$class as ToJni>::signature(), $signature);
            assert_eq!(<$class as JavaClassType>::signature(), $signature);
        }

        #[test]
        fn to_jni() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let raw_object = 0x91011 as jni_sys::jobject;
            let object = test_value(&env, raw_object);
            unsafe {
                assert_eq!(object.to_jni(), raw_object);
            }
            mem::forget(object);
        }

        #[test]
        fn from_jni() {
            let vm = test_vm(ptr::null_mut());
            let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
            let env = test_env(&vm, jni_env);
            let raw_object = 0x91011 as jni_sys::jobject;
            unsafe {
                let object = $class::from_jni(&env, raw_object);
                assert_eq!(object.raw_object(), raw_object);
                assert_eq!(object.env().raw_env(), jni_env);
                mem::forget(object);
            }
        }

        #[test]
        fn to_and_from() {
            let vm = test_vm(ptr::null_mut());
            let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
            let env = test_env(&vm, jni_env);
            let raw_object = 0x91011 as jni_sys::jobject;
            let object = test_value(&env, raw_object);
            unsafe {
                let object = $class::from_jni(&env, object.to_jni());
                assert_eq!(object.raw_object(), raw_object);
                assert_eq!(object.env().raw_env(), jni_env);
                mem::forget(object);
            }
            mem::forget(object);
        }

        #[test]
        fn from_and_to() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let raw_object = 0x91011 as jni_sys::jobject;
            unsafe {
                let object = $class::from_jni(&env, raw_object);
                assert_eq!(object.to_jni(), raw_object);
                mem::forget(object);
            }
        }

        #[test]
        fn from_object() {
            let vm = test_vm(ptr::null_mut());
            let jni_env = 0x5678 as *mut jni_sys::JNIEnv;
            let env = test_env(&vm, jni_env);
            let raw_object = 0x91011 as jni_sys::jobject;
            unsafe {
                let object = $class::from_object(test_object(&env, raw_object));
                assert_eq!(object.raw_object(), raw_object);
                assert_eq!(object.env().raw_env(), jni_env);
                mem::forget(object);
            }
        }

        #[test]
        fn drop() {
            const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::DeleteLocalRef(DeleteLocalRef {
                object: RAW_OBJECT,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            {
                let _object = test_value(&env, RAW_OBJECT);
                // Haven't called the destructor at this point yet.
                assert_eq!(calls.current_call, 0);
            }
        }

        #[test]
        fn clone() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x1234 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::NewLocalRef(NewLocalRef {
                object: RAW_OBJECT1,
                result: RAW_OBJECT2,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT1);
            let clone = object.clone(&NoException::test()).unwrap();
            calls.assert_eq(&clone, RAW_OBJECT2);
        }

        #[test]
        fn clone_null() {
            const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::NewLocalRef(NewLocalRef {
                    object: RAW_OBJECT,
                    result: ptr::null_mut() as jni_sys::jobject,
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            let exception = object.clone(&NoException::test()).unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }

        #[test]
        fn eq() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::ExceptionCheck(ExceptionCheck {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::IsSameObject(IsSameObject {
                    object1: RAW_OBJECT1,
                    object2: RAW_OBJECT2,
                    result: jni_sys::JNI_TRUE,
                }),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object1 = test_value(&env, RAW_OBJECT1);
            let object2 = test_value(&env, RAW_OBJECT2);
            assert!(object1 == object2);
        }

        #[test]
        fn eq_not_same() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::ExceptionCheck(ExceptionCheck {
                    result: jni_sys::JNI_FALSE,
                }),
                JniCall::IsSameObject(IsSameObject {
                    object1: RAW_OBJECT1,
                    object2: RAW_OBJECT2,
                    result: jni_sys::JNI_FALSE,
                }),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object1 = test_value(&env, RAW_OBJECT1);
            let object2 = test_value(&env, RAW_OBJECT2);
            assert!(object1 != object2);
        }

        #[test]
        #[should_panic(
            expected = "Comparing Java objects with a pending exception in the current thread"
        )]
        fn eq_pending_exception() {
            const RAW_OBJECT1: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            const RAW_OBJECT2: jni_sys::jobject = 0x93486 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object1 = test_value(&env, RAW_OBJECT1);
            let object2 = test_value(&env, RAW_OBJECT2);
            let _ = object1 == object2;
        }

        #[test]
        fn to_string() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                RAW_STRING
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::GetObjectClass(GetObjectClass {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            unsafe {
                let string = object.to_string(&NoException::test()).unwrap();
                calls.assert_eq(&string, RAW_STRING);
                assert_eq!(METHOD_CALLS, 1);
                assert_eq!(METHOD_ENV_ARGUMENT, calls.env);
            }
        }

        #[test]
        fn to_string_exception() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x248670 as jni_sys::jmethodID;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                _: *mut jni_sys::JNIEnv,
                _: jni_sys::jobject,
                _: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                ptr::null_mut()
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::GetObjectClass(GetObjectClass {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClear {}),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            let exception = object.to_string(&NoException::test()).unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }

        #[test]
        fn display() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x2835 as jni_sys::jmethodID;
            const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
            const LENGTH: usize = 5;
            const SIZE: usize = 11; // `"test-string".len()`.
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                assert_eq!(object, RAW_OBJECT);
                assert_eq!(method_id, METHOD_ID);
                METHOD_CALLS += 1;
                METHOD_ENV_ARGUMENT = env;
                RAW_STRING
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::ExceptionCheck(ExceptionCheck {
                        result: jni_sys::JNI_FALSE,
                    }),
                    JniCall::GetObjectClass(GetObjectClass {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                    JniCall::GetStringLength(GetStringLength {
                        string: RAW_STRING,
                        result: LENGTH as jni_sys::jsize,
                    }),
                    JniCall::GetStringUTFLength(GetStringUTFLength {
                        string: RAW_STRING,
                        result: SIZE as jni_sys::jsize,
                    }),
                    JniCall::GetStringUTFRegion(GetStringUTFRegion {
                        string: RAW_STRING,
                        start: 0,
                        len: LENGTH as jni_sys::jsize,
                        buffer: "test-string".to_owned(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_STRING }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            assert_eq!(format!("{}", object), "test-string");
        }

        #[test]
        #[should_panic(
            expected = "Displaying a Java object with a pending exception in the current thread"
        )]
        fn display_exception_pending() {
            let calls = test_raw_jni_env!(vec![JniCall::ExceptionCheck(ExceptionCheck {
                result: jni_sys::JNI_TRUE,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, ptr::null_mut());
            format!("{}", object);
        }

        #[test]
        fn display_exception_thrown() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const RAW_EXCEPTION_CLASS: jni_sys::jobject = 0x912376 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x923476 as jni_sys::jmethodID;
            const EXCEPTION_METHOD_ID: jni_sys::jmethodID = 0x8293659 as jni_sys::jmethodID;
            const RAW_STRING: jni_sys::jstring = 0x92385 as jni_sys::jstring;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            const LENGTH: usize = 5;
            const SIZE: usize = 11; // `"test-string".len()`.
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                METHOD_CALLS += 1;
                if METHOD_CALLS == 1 {
                    assert_eq!(object, RAW_OBJECT);
                    assert_eq!(method_id, METHOD_ID);
                    METHOD_ENV_ARGUMENT = env;
                } else {
                    assert_eq!(object, EXCEPTION);
                    assert_eq!(method_id, EXCEPTION_METHOD_ID);
                    assert_eq!(env, METHOD_ENV_ARGUMENT);
                }
                RAW_STRING
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::ExceptionCheck(ExceptionCheck {
                        result: jni_sys::JNI_FALSE,
                    }),
                    JniCall::GetObjectClass(GetObjectClass {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClear {}),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                    JniCall::GetObjectClass(GetObjectClass {
                        object: EXCEPTION,
                        result: RAW_EXCEPTION_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_EXCEPTION_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: EXCEPTION_METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred {
                        result: ptr::null_mut(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRef {
                        object: RAW_EXCEPTION_CLASS,
                    }),
                    JniCall::GetStringLength(GetStringLength {
                        string: RAW_STRING,
                        result: LENGTH as jni_sys::jsize,
                    }),
                    JniCall::GetStringUTFLength(GetStringUTFLength {
                        string: RAW_STRING,
                        result: SIZE as jni_sys::jsize,
                    }),
                    JniCall::GetStringUTFRegion(GetStringUTFRegion {
                        string: RAW_STRING,
                        start: 0,
                        len: LENGTH as jni_sys::jsize,
                        buffer: "test-string".to_owned(),
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_STRING }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            assert_eq!(
                format!("{}", object),
                "Object::toString threw an exception: test-string"
            );
        }

        #[test]
        fn display_exception_thrown_twice() {
            const RAW_OBJECT: jni_sys::jobject = 0x924858 as jni_sys::jobject;
            const RAW_CLASS: jni_sys::jobject = 0x239875 as jni_sys::jobject;
            const RAW_EXCEPTION_CLASS: jni_sys::jobject = 0x912376 as jni_sys::jobject;
            const METHOD_ID: jni_sys::jmethodID = 0x923476 as jni_sys::jmethodID;
            const EXCEPTION_METHOD_ID: jni_sys::jmethodID = 0x8293659 as jni_sys::jmethodID;
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            const EXCEPTION2: jni_sys::jobject = 0x2836 as jni_sys::jobject;
            static mut METHOD_CALLS: i32 = 0;
            static mut METHOD_ENV_ARGUMENT: *mut jni_sys::JNIEnv = ptr::null_mut();
            type VariadicFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
                ...
            ) -> jni_sys::jstring;
            type TestFn = unsafe extern "C" fn(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring;
            unsafe extern "C" fn method(
                env: *mut jni_sys::JNIEnv,
                object: jni_sys::jobject,
                method_id: jni_sys::jmethodID,
            ) -> jni_sys::jstring {
                METHOD_CALLS += 1;
                if METHOD_CALLS == 1 {
                    assert_eq!(object, RAW_OBJECT);
                    assert_eq!(method_id, METHOD_ID);
                    METHOD_ENV_ARGUMENT = env;
                } else {
                    assert_eq!(object, EXCEPTION);
                    assert_eq!(method_id, EXCEPTION_METHOD_ID);
                    assert_eq!(env, METHOD_ENV_ARGUMENT);
                }
                ptr::null_mut()
            }
            let raw_jni_env = jni_sys::JNINativeInterface_ {
                CallObjectMethod: Some(unsafe { mem::transmute::<TestFn, VariadicFn>(method) }),
                ..empty_raw_jni_env()
            };
            let calls = test_raw_jni_env!(
                vec![
                    JniCall::ExceptionCheck(ExceptionCheck {
                        result: jni_sys::JNI_FALSE,
                    }),
                    JniCall::GetObjectClass(GetObjectClass {
                        object: RAW_OBJECT,
                        result: RAW_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                    JniCall::ExceptionClear(ExceptionClear {}),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: RAW_CLASS }),
                    JniCall::GetObjectClass(GetObjectClass {
                        object: EXCEPTION,
                        result: RAW_EXCEPTION_CLASS,
                    }),
                    JniCall::GetMethodID(GetMethodID {
                        class: RAW_EXCEPTION_CLASS,
                        name: "toString".to_owned(),
                        signature: "()Ljava/lang/String;".to_owned(),
                        result: EXCEPTION_METHOD_ID,
                    }),
                    JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION2 }),
                    JniCall::ExceptionClear(ExceptionClear {}),
                    JniCall::DeleteLocalRef(DeleteLocalRef {
                        object: RAW_EXCEPTION_CLASS,
                    }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION2 }),
                    JniCall::DeleteLocalRef(DeleteLocalRef { object: EXCEPTION }),
                ],
                raw_jni_env
            );
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let object = test_value(&env, RAW_OBJECT);
            assert_eq!(
                format!("{}", object),
                "<Object::toString threw an exception which could not be formatted>"
            );
        }

        #[test]
        fn get_class() {
            const RAW_OBJECT: jni_sys::jobject = 0x91011 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![JniCall::FindClass(FindClass {
                name: concat!("java/lang/", stringify!($class)).to_owned(),
                result: RAW_OBJECT,
            })]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let class = $class::get_class(&env, &NoException::test()).unwrap();
            calls.assert_eq(&class, RAW_OBJECT);
        }

        #[test]
        fn get_class_not_found() {
            const EXCEPTION: jni_sys::jobject = 0x2835 as jni_sys::jobject;
            let calls = test_raw_jni_env!(vec![
                JniCall::FindClass(FindClass {
                    name: concat!("java/lang/", stringify!($class)).to_owned(),
                    result: ptr::null_mut(),
                }),
                JniCall::ExceptionOccurred(ExceptionOccurred { result: EXCEPTION }),
                JniCall::ExceptionClear(ExceptionClear {}),
            ]);
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, calls.env);
            let exception = $class::get_class(&env, &NoException::test()).unwrap_err();
            calls.assert_eq(&exception, EXCEPTION);
        }
    };
}

// It's actually used.
#[allow(unused_macros)]
#[cfg(test)]
macro_rules! generate_tests {
    ($class:ident, $signature:expr) => {
        generate_object_tests!($class, $signature);

        #[test]
        fn deref_super() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let object = test_value(&env, ptr::null_mut());
            // Will not compile if is not deref-able.
            &object as &dyn Deref<Target = Object>;
            mem::forget(object);
        }

        #[test]
        fn cast() {
            let vm = test_vm(ptr::null_mut());
            let env = test_env(&vm, ptr::null_mut());
            let object = test_value(&env, ptr::null_mut());
            assert_eq!(&object as *const _, object.cast() as *const _);
            assert_eq!(&object.object as *const _, object.cast() as *const _);
            mem::forget(object);
        }
    };
}
