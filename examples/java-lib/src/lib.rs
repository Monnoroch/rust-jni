extern crate jni_sys;
#[macro_use]
extern crate rust_jni;

pub mod rustjni {
    pub mod test {
        use rust_jni::java;

        java_interface!(
            interface = TestInterface,
            link = "",
            extends = (),
            methods = (
                doc = "",
                link = "",
                java_name = "testInterfaceFunction",
                test_interface_function(value: i32) -> i64,
            ),
        );

        java_interface!(
            interface = TestInterfaceExtended,
            link = "",
            extends = (TestInterface),
            methods = (
                doc = "",
                link = "",
                java_name = "testInterfaceExtendedFunction",
                test_interface_extended_function(value: i32) -> i64,
            ),
        );

        java_class!(
            package = "rustjni/test",
            class = TestClass,
            link = "",
            rust_link = "",
            extends = java::lang::Object,
            super_link = "",
            implements = (
                name = TestInterface,
                link = "",
                methods = (
                    test_interface_function(value: i32) -> i64,
                ),
            ),
            constructors = (
                doc = "",
                link = "",
                init(),
            ),
            methods = (
                doc = "",
                link = "",
                java_name = "testClassFunction",
                test_class_function(value: i32) -> i64,
                doc = "",
                link = "",
                java_name = "testInterfaceFunction",
                test_interface_function(value: i32) -> i64,
            ),
            static_methods = (
                doc = "",
                link = "",
                java_name = "create",
                create() -> Self,
            ),
            native_methods = (),
            static_native_methods = (),
            super_classes = (),
            super_interfaces = (),
        );

        java_class!(
            package = "rustjni/test",
            class = TestClassExtended,
            link = "",
            rust_link = "",
            extends = TestClass,
            super_link = "",
            implements = (
                name = TestInterfaceExtended,
                link = "",
                methods = (
                    test_interface_extended_function(value: i32) -> i64,
                ),
            ),
            constructors = (
                doc = "",
                link = "",
                init(),
            ),
            methods = (
                doc = "",
                link = "",
                java_name = "testClassExtendedFunction",
                test_class_extended_function(value: i32) -> i64,
                doc = "",
                link = "",
                java_name = "testInterfaceExtendedFunction",
                test_interface_extended_function(value: i32) -> i64,
            ),
            static_methods = (
                doc = "",
                link = "",
                java_name = "create",
                create() -> Self,
            ),
            native_methods = (),
            static_native_methods = (),
            super_classes = (
                java::lang::Object,
                link = ""
            ),
            super_interfaces = (
                name = TestInterface,
                link = "",
                methods = (
                    test_interface_function(value: i32) -> i64,
                ),
            ),
        );

        java_class!(
            package = "rustjni/test",
            class = TestClassExtendedFinal,
            link = "",
            rust_link = "",
            extends = TestClassExtended,
            super_link = "",
            implements = (),
            constructors = (
                doc = "",
                link = "",
                init(),
            ),
            methods = (
                doc = "",
                link = "",
                java_name = "testClassExtendedFinalFunction",
                test_class_extended_final_function(value: i32) -> i64,
            ),
            static_methods = (
                doc = "",
                link = "",
                java_name = "create",
                create() -> Self,
            ),
            native_methods = (),
            static_native_methods = (),
            super_classes = (
                java::lang::Object,
                link = "",
                TestClass,
                link = ""
            ),
            super_interfaces = (
                name = TestInterface,
                link = "",
                methods = (
                    test_interface_function(value: i32) -> i64,
                ),
                name = TestInterfaceExtended,
                link = "",
                methods = (
                    test_interface_extended_function(value: i32) -> i64,
                ),
            ),
        );

        java_class!(
            package = "rustjni/test",
            class = TestMethodsClass,
            link = "",
            rust_link = "",
            extends = java::lang::Object,
            super_link = "",
            implements = (),
            constructors = (
                doc = "",
                link = "",
                init(),
            ),
            methods = (
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_void() -> (),
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_bool(value1: bool, value2: bool, value3: bool) -> bool,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_char(value1: char, value2: char, value3: char) -> char,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_byte(value1: u8, value2: u8, value3: u8) -> u8,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_short(value1: i16, value2: i16, value3: i16) -> i16,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_int(value1: i32, value2: i32, value3: i32) -> i32,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_long(value1: i64, value2: i64, value3: i64) -> i64,
                // TODO(#25): enable when fixed.
                // doc = "",
                // link = "",
                // java_name = "testFunction",
                // test_function_float(value1: f32, value2: f32, value3: f32) -> f32,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_double(value1: f64, value2: f64, value3: f64) -> f64,
                doc = "",
                link = "",
                java_name = "testFunction",
                test_function_class(value1: &TestMethodsClass, value2: &TestMethodsClass, value3: &TestMethodsClass) -> TestMethodsClass<'env>,
            ),
            static_methods = (
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_void() -> (),
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_bool(value1: bool, value2: bool, value3: bool) -> bool,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_char(value1: char, value2: char, value3: char) -> char,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_byte(value1: u8, value2: u8, value3: u8) -> u8,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_short(value1: i16, value2: i16, value3: i16) -> i16,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_int(value1: i32, value2: i32, value3: i32) -> i32,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_long(value1: i64, value2: i64, value3: i64) -> i64,
                // TODO(#25): enable when fixed.
                // doc = "",
                // link = "",
                // java_name = "testStaticFunction",
                // test_static_function_float(value1: f32, value2: f32, value3: f32) -> f32,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_double(value1: f64, value2: f64, value3: f64) -> f64,
                doc = "",
                link = "",
                java_name = "testStaticFunction",
                test_static_function_class(value1: &TestMethodsClass, value2: &TestMethodsClass, value3: &TestMethodsClass) -> TestMethodsClass<'env>,
            ),
            native_methods = (
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__,
                test_native_function_void() -> (),
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__ZZZ,
                test_native_function_bool(value1: bool, value2: bool, value3: bool) -> bool,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__CCC,
                test_native_function_char(value1: char, value2: char, value3: char) -> char,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__BBB,
                test_native_function_byte(value1: u8, value2: u8, value3: u8) -> u8,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__SSS,
                test_native_function_short(value1: i16, value2: i16, value3: i16) -> i16,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__III,
                test_native_function_int(value1: i32, value2: i32, value3: i32) -> i32,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__JJJ,
                test_native_function_long(value1: i64, value2: i64, value3: i64) -> i64,
                // TODO(#25): enable when fixed.
                // function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__FFF,
                // test_native_function_float(value1: f32, value2: f32, value3: f32) -> f32,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__DDD,
                test_native_function_double(value1: f64, value2: f64, value3: f64) -> f64,
                function_name = Java_rustjni_test_TestMethodsClass_testNativeFunction__Lrustjni_test_TestMethodsClass_2Lrustjni_test_TestMethodsClass_2Lrustjni_test_TestMethodsClass_2,
                test_native_function_class(value1: TestMethodsClass, value2: TestMethodsClass, value3: TestMethodsClass) -> TestMethodsClass<'static>,
            ),
            static_native_methods = (
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__,
                test_static_native_function_void() -> (),
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__ZZZ,
                test_static_native_function_bool(value1: bool, value2: bool, value3: bool) -> bool,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__CCC,
                test_static_native_function_char(value1: char, value2: char, value3: char) -> char,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__BBB,
                test_static_native_function_byte(value1: u8, value2: u8, value3: u8) -> u8,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__SSS,
                test_static_native_function_short(value1: i16, value2: i16, value3: i16) -> i16,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__III,
                test_static_native_function_int(value1: i32, value2: i32, value3: i32) -> i32,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__JJJ,
                test_static_native_function_long(value1: i64, value2: i64, value3: i64) -> i64,
                // TODO(#25): enable when fixed.
                // function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__FFF,
                // test_static_native_function_float(value1: f32, value2: f32, value3: f32) -> f32,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__DDD,
                test_static_native_function_double(value1: f64, value2: f64, value3: f64) -> f64,
                function_name = Java_rustjni_test_TestMethodsClass_testStaticNativeFunction__Lrustjni_test_TestMethodsClass_2Lrustjni_test_TestMethodsClass_2Lrustjni_test_TestMethodsClass_2,
                test_static_native_function_class(value1: TestMethodsClass, value2: TestMethodsClass, value3: TestMethodsClass) -> TestMethodsClass<'static>,
            ),
            super_classes = (),
            super_interfaces = (),
        );

        impl<'env> TestMethodsClass<'env> {
            pub fn test_native_function_test(
                &self,
                value: TestMethodsClass<'env>,
                _token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, TestMethodsClass<'env>> {
                Ok(value)
            }

            pub fn test_native_function_void(
                &self,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, ()> {
                Self::test_static_function_void(self.env(), token)
            }

            pub fn test_native_function_bool(
                &self,
                value1: bool,
                value2: bool,
                value3: bool,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, bool> {
                Self::test_static_function_bool(self.env(), value1, value2, value3, token)
            }

            pub fn test_native_function_char(
                &self,
                value1: char,
                value2: char,
                value3: char,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, char> {
                Self::test_static_function_char(self.env(), value1, value2, value3, token)
            }

            pub fn test_native_function_byte(
                &self,
                value1: u8,
                value2: u8,
                value3: u8,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, u8> {
                Self::test_static_function_byte(self.env(), value1, value2, value3, token)
            }

            pub fn test_native_function_short(
                &self,
                value1: i16,
                value2: i16,
                value3: i16,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, i16> {
                Self::test_static_function_short(self.env(), value1, value2, value3, token)
            }

            pub fn test_native_function_int(
                &self,
                value1: i32,
                value2: i32,
                value3: i32,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, i32> {
                Self::test_static_function_int(self.env(), value1, value2, value3, token)
            }

            pub fn test_native_function_long(
                &self,
                value1: i64,
                value2: i64,
                value3: i64,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, i64> {
                Self::test_static_function_long(self.env(), value1, value2, value3, token)
            }

            // TODO(#25): enable when fixed.
            // pub fn test_native_function_float(
            //     &self,
            //     value1: f32,
            //     value2: f32,
            //     value3: f32,
            //     token: &::rust_jni::NoException<'env>,
            // ) -> ::rust_jni::JavaResult<'env, f32> {
            //     Self::test_static_function_float(self.env(), value1, value2, value3, token)
            // }

            pub fn test_native_function_double(
                &self,
                value1: f64,
                value2: f64,
                value3: f64,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, f64> {
                Self::test_static_function_double(self.env(), value1, value2, value3, token)
            }

            pub fn test_native_function_class(
                &self,
                value1: TestMethodsClass<'env>,
                value2: TestMethodsClass<'env>,
                value3: TestMethodsClass<'env>,
                token: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, TestMethodsClass<'env>> {
                Self::test_static_function_class(self.env(), &value1, &value2, &value3, token)
            }

            pub fn test_static_native_function_void(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, ()> {
                Ok(())
            }

            pub fn test_static_native_function_bool(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: bool,
                value2: bool,
                _value3: bool,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, bool> {
                Ok(value2)
            }

            pub fn test_static_native_function_char(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: char,
                value2: char,
                _value3: char,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, char> {
                Ok(value2)
            }

            pub fn test_static_native_function_byte(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: u8,
                value2: u8,
                _value3: u8,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, u8> {
                Ok(value2)
            }

            pub fn test_static_native_function_short(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: i16,
                value2: i16,
                _value3: i16,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, i16> {
                Ok(value2)
            }

            pub fn test_static_native_function_int(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: i32,
                value2: i32,
                _value3: i32,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, i32> {
                Ok(value2)
            }

            pub fn test_static_native_function_long(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: i64,
                value2: i64,
                _value3: i64,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, i64> {
                Ok(value2)
            }

            // TODO(#25): enable when fixed.
            // pub fn test_static_native_function_float(
            //     _env: &'env ::rust_jni::JniEnv<'env>,
            //     _value1: f32,
            //     value2: f32,
            //     _value3: f32,
            //     _: &::rust_jni::NoException<'env>,
            // ) -> ::rust_jni::JavaResult<'env, f32> {
            //     Ok(value2)
            // }

            pub fn test_static_native_function_double(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: f64,
                value2: f64,
                _value3: f64,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, f64> {
                Ok(value2)
            }

            pub fn test_static_native_function_class(
                _env: &'env ::rust_jni::JniEnv<'env>,
                _value1: TestMethodsClass<'env>,
                value2: TestMethodsClass<'env>,
                _value3: TestMethodsClass<'env>,
                _: &::rust_jni::NoException<'env>,
            ) -> ::rust_jni::JavaResult<'env, TestMethodsClass<'env>> {
                Ok(value2)
            }
        }
    }
}
