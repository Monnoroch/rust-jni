#![allow(dead_code)]
extern crate rust_jni;
extern crate rust_jni_generator;

#[cfg(test)]
mod java {
    pub mod lang {
        pub use rust_jni::java::lang::*;
    }
}

#[cfg(test)]
mod a {
    mod b {
        #[allow(unused_imports)]
        use rust_jni_generator::*;

        java_generate! {
            public interface a.b.TestInterface1 {}
            public interface a.b.TestInterface2 extends a.b.TestInterface1 {}
            public interface a.b.TestInterface3 {}
            public interface a.b.TestInterface4 extends a.b.TestInterface2, a.b.TestInterface3 {}

            public class a.b.TestClass1 {}
            public class a.b.TestClass2 extends a.b.TestClass1 implements a.b.TestInterface1 {}
            public class a.b.TestClass3 extends a.b.TestClass2 implements a.b.TestInterface1, a.b.TestInterface4 {}
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
