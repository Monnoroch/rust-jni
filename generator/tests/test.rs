#![allow(dead_code)]
extern crate jni_sys;
extern crate rust_jni;
extern crate rust_jni_generator;

#[cfg(test)]
mod java {
    pub mod lang {
        pub use rust_jni::java::lang::*;
    }
}

#[cfg(test)]
mod e {
    pub mod f {
        #[allow(unused_imports)]
        use rust_jni_generator::*;

        java_generate! {
            public interface e.f.TestInterface1 {
                @RustName(primitive_interface_func_1)
                long primitiveInterfaceFunc1(int arg1, char arg2);
            }
        }
    }
}

#[cfg(test)]
mod c {
    pub mod d {
        #[allow(unused_imports)]
        use rust_jni_generator::*;

        java_generate! {
            public interface c.d.TestInterface2 extends e.f.TestInterface1 {}

            public class c.d.TestClass1 {
                public c.d.TestClass1(int arg);

                public long primitiveFunc1(int arg1, char arg2);
                c.d.TestClass1 objectFunc1(c.d.TestClass1 arg);

                public static long primitiveStaticFunc1(int arg1, char arg2);
                static c.d.TestClass1 objectStaticFunc1(c.d.TestClass1 arg);
            }
            public class c.d.TestClass2 extends c.d.TestClass1 implements e.f.TestInterface1 {
                public c.d.TestClass2(c.d.TestClass1 arg);

                long primitiveFunc2(int arg1, char arg2);
                public c.d.TestClass2 objectFunc2(c.d.TestClass1 arg);

                @RustName(primitive_interface_func_1)
                long primitiveInterfaceFunc1(int arg1, char arg2);
            }

            metadata {
                interface e.f.TestInterface1 {
                    @RustName(primitive_interface_func_1)
                    long primitiveInterfaceFunc1(int arg1, char arg2);
                }
            }
        }
    }
}

#[cfg(test)]
mod a {
    mod b {
        #[allow(unused_imports)]
        use rust_jni_generator::*;

        java_generate! {
            public interface a.b.TestInterface3 {
                long primitiveInterfaceFunc3(int arg1, char arg2);
                a.b.TestClass3 objectInterfaceFunc3(a.b.TestClass3 arg);
            }

            public interface a.b.TestInterface4 extends c.d.TestInterface2, a.b.TestInterface3 {
                @RustName(primitive_func_3)
                long primitiveFunc3(int arg1, char arg2);
                @RustName(object_func_3)
                c.d.TestClass2 objectFunc3(a.b.TestClass3 arg);
            }

            public class a.b.TestClass3 extends c.d.TestClass2 implements e.f.TestInterface1, a.b.TestInterface4 {
                @RustName(init)
                public a.b.TestClass3(int arg1, a.b.TestClass3 arg2);

                @RustName(primitive_func_3)
                long primitiveFunc3(int arg1, char arg2);
                @RustName(object_func_3)
                public c.d.TestClass2 objectFunc3(a.b.TestClass3 arg);

                @RustName(primitive_static_func_3)
                static long primitiveStaticFunc3(int arg1, char arg2);
                @RustName(object_static_func_3)
                public static c.d.TestClass2 objectStaticFunc3(a.b.TestClass3 arg);

                @RustName(primitive_native_func_3)
                public native long primitiveNativeFunc3(int arg1, char arg2) {
                    println!("{:?} {:?} {:?} {:?}", arg1, arg2, token, self);
                    Ok(0)
                };
                native a.b.TestClass3 objectNativeFunc3(a.b.TestClass3 arg) {
                    println!("{:?} {:?} {:?}", arg, token, self);
                    Ok(arg)
                };

                @RustName(primitive_static_native_func_3)
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
                    @RustName(primitive_interface_func_1)
                    long primitiveInterfaceFunc1(int arg1, char arg2);
                }
                interface c.d.TestInterface2 extends e.f.TestInterface1 {}

                class c.d.TestClass1;
                class c.d.TestClass2 extends c.d.TestClass1 implements e.f.TestInterface1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
