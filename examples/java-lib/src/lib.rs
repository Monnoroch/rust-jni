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
            super_classes = (),
            super_interfaces = (),
        );
    }
}
