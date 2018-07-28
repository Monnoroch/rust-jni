extern crate jni_sys;
#[macro_use]
extern crate rust_jni;
extern crate java;
extern crate rust_jni_generator;

pub mod rustjni {
    pub mod test {

        #[allow(unused_imports)]
        use rust_jni_generator::*;

        java_generate! {
            public interface rustjni.test.TestInterface {
                @RustName(test_interface_function)
                long testInterfaceFunction(int value);
            }

            public interface rustjni.test.TestInterfaceExtended extends rustjni.test.TestInterface {
                @RustName(test_interface_extended_function)
                long testInterfaceExtendedFunction(int value);
            }

            public class rustjni.test.TestClass implements rustjni.test.TestInterface {
                @RustName(init)
                public rustjni.test.TestClass();

                @RustName(test_class_function)
                public long testClassFunction(int value);
                @RustName(test_interface_function)
                long testInterfaceFunction(int value);

                public static rustjni.test.TestClass create();
            }

            public class rustjni.test.TestClassExtended extends rustjni.test.TestClass implements rustjni.test.TestInterfaceExtended {
                @RustName(init)
                public rustjni.test.TestClassExtended();

                @RustName(test_class_extended_function)
                public long testClassExtendedFunction(int value);
                @RustName(test_interface_extended_function)
                long testInterfaceExtendedFunction(int value);

                public static rustjni.test.TestClassExtended create();
            }

            public class rustjni.test.TestClassExtendedFinal extends rustjni.test.TestClassExtended {
                @RustName(init)
                public rustjni.test.TestClassExtendedFinal();

                @RustName(test_class_extended_final_function)
                public long testClassExtendedFinalFunction(int value);

                public static rustjni.test.TestClassExtendedFinal create();
            }

            public class rustjni.test.TestMethodsClass {
                @RustName(init)
                public rustjni.test.TestMethodsClass();

                @RustName(test_function_void)
                public void testFunction();

                @RustName(test_function_bool)
                public boolean testFunction(boolean value1, boolean value2, boolean value3);

                @RustName(test_function_char)
                public char testFunction(char value1, char value2, char value3);

                @RustName(test_function_byte)
                public byte testFunction(byte value1, byte value2, byte value3);

                @RustName(test_function_short)
                public short testFunction(short value1, short value2, short value3);

                @RustName(test_function_int)
                public int testFunction(int value1, int value2, int value3);

                @RustName(test_function_long)
                public long testFunction(long value1, long value2, long value3);

                // TODO(#25): enable when fixed.
                // @RustName(test_function_float)
                // public float testFunction(float value1, float value2, float value3);

                @RustName(test_function_double)
                public double testFunction(double value1, double value2, double value3);

                @RustName(test_function_class)
                public rustjni.test.TestMethodsClass testFunction(rustjni.test.TestMethodsClass value1, rustjni.test.TestMethodsClass value2, rustjni.test.TestMethodsClass value3);

                @RustName(test_static_function_void)
                public static void testStaticFunction();

                @RustName(test_static_function_bool)
                public static boolean testStaticFunction(boolean value1, boolean value2, boolean value3);

                @RustName(test_static_function_char)
                public static char testStaticFunction(char value1, char value2, char value3);

                @RustName(test_static_function_byte)
                public static byte testStaticFunction(byte value1, byte value2, byte value3);

                @RustName(test_static_function_short)
                public static short testStaticFunction(short value1, short value2, short value3);

                @RustName(test_static_function_int)
                public static int testStaticFunction(int value1, int value2, int value3);

                @RustName(test_static_function_long)
                public static long testStaticFunction(long value1, long value2, long value3);

                // TODO(#25): enable when fixed.
                // @RustName(test_static_function_float)
                // public static float testStaticFunction(float value1, float value2, float value3);

                @RustName(test_static_function_double)
                public static double testStaticFunction(double value1, double value2, double value3);

                @RustName(test_static_function_class)
                public static rustjni.test.TestMethodsClass testStaticFunction(rustjni.test.TestMethodsClass value1, rustjni.test.TestMethodsClass value2, rustjni.test.TestMethodsClass value3);

                @RustName(test_native_function_void)
                public native void testNativeFunction() {
                    Self::test_static_function_void(self.env(), token)
                };

                @RustName(test_native_function_bool)
                public native boolean testNativeFunction(boolean value1, boolean value2, boolean value3) {
                    Self::test_static_function_bool(self.env(), value1, value2, value3, token)
                };

                @RustName(test_native_function_char)
                public native char testNativeFunction(char value1, char value2, char value3) {
                    Self::test_static_function_char(self.env(), value1, value2, value3, token)
                };

                @RustName(test_native_function_byte)
                public native byte testNativeFunction(byte value1, byte value2, byte value3) {
                    Self::test_static_function_byte(self.env(), value1, value2, value3, token)
                };

                @RustName(test_native_function_short)
                public native short testNativeFunction(short value1, short value2, short value3) {
                    Self::test_static_function_short(self.env(), value1, value2, value3, token)
                };

                @RustName(test_native_function_int)
                public native int testNativeFunction(int value1, int value2, int value3) {
                    Self::test_static_function_int(self.env(), value1, value2, value3, token)
                };

                @RustName(test_native_function_long)
                public native long testNativeFunction(long value1, long value2, long value3) {
                    Self::test_static_function_long(self.env(), value1, value2, value3, token)
                };

                // TODO(#25): enable when fixed.
                // @RustName(test_native_function_float)
                // public native float testNativeFunction(float value1, float value2, float value3) {
                //     Self::test_static_function_float(self.env(), value1, value2, value3, token)
                // };

                @RustName(test_native_function_double)
                public native double testNativeFunction(double value1, double value2, double value3) {
                    Self::test_static_function_double(self.env(), value1, value2, value3, token)
                };

                @RustName(test_native_function_class)
                public native rustjni.test.TestMethodsClass testNativeFunction(rustjni.test.TestMethodsClass value1, rustjni.test.TestMethodsClass value2, rustjni.test.TestMethodsClass value3) {
                    Self::test_static_function_class(self.env(), &value1, &value2, &value3, token)
                };

                @RustName(test_static_native_function_void)
                public static native void testStaticNativeFunction() {
                    Ok(())
                };

                @RustName(test_static_native_function_bool)
                public static native boolean testStaticNativeFunction(boolean value1, boolean value2, boolean value3) {
                    Ok(value2)
                };

                @RustName(test_static_native_function_char)
                public static native char testStaticNativeFunction(char value1, char value2, char value3) {
                    Ok(value2)
                };

                @RustName(test_static_native_function_byte)
                public static native byte testStaticNativeFunction(byte value1, byte value2, byte value3) {
                    Ok(value2)
                };

                @RustName(test_static_native_function_short)
                public static native short testStaticNativeFunction(short value1, short value2, short value3) {
                    Ok(value2)
                };

                @RustName(test_static_native_function_int)
                public static native int testStaticNativeFunction(int value1, int value2, int value3) {
                    Ok(value2)
                };

                @RustName(test_static_native_function_long)
                public static native long testStaticNativeFunction(long value1, long value2, long value3) {
                    Ok(value2)
                };

                // TODO(#25): enable when fixed.
                // @RustName(test_static_native_function_float)
                // public static native float testStaticNativeFunction(float value1, float value2, float value3) {
                //     Ok(value2)
                // };

                @RustName(test_static_native_function_double)
                public static native double testStaticNativeFunction(double value1, double value2, double value3) {
                    Ok(value2)
                };

                @RustName(test_static_native_function_class)
                public static native rustjni.test.TestMethodsClass testStaticNativeFunction(rustjni.test.TestMethodsClass value1, rustjni.test.TestMethodsClass value2, rustjni.test.TestMethodsClass value3) {
                    Ok(value2)
                };
            }
        }
    }
}
