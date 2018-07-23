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
            class a.b.TestClass1 extends java.lang.Object {}
            public class TestClass2 extends TestClass1 {}
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
