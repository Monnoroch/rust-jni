#![allow(dead_code)]
extern crate rust_jni;
extern crate rust_jni_generator;

#[cfg(test)]
mod a {
    mod b {
        use rust_jni::java;

        #[allow(unused_imports)]
        use rust_jni_generator::*;

        java_generate! {
            public class a.b.TestClass1 extends java.lang.Object {}
            public class a.b.TestClass2 extends TestClass1 {}
        }

        // TODO(#76): generate this.
        impl<'a> ::rust_jni::Cast<'a, java::lang::Object<'a>> for TestClass2<'a> {
            #[doc(hidden)]
            fn cast<'b>(&'b self) -> &'b java::lang::Object<'a> {
                self
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
