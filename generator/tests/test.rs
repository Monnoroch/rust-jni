#![allow(dead_code)]
extern crate rust_jni_generator;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use rust_jni_generator::*;

    java_generate!{
        class a.b.TestClass1 {}
        public class TestClass2 {}
    }

    #[test]
    fn test() {}
}