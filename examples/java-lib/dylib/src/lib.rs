/// Only extern the Rust library with the actual code, as this shared librarie's purpose
/// is just to make the Rust library loadable by the JVM.
extern crate rust_jni_java_lib;
