//! # A library for safe interoperation between Rust and Java
//!
//! [`rust-jni`](index.html) provides tools to safely make calls from Rust to Java
//! and from Java to Rust using
//! [JNI](https://docs.oracle.com/javase/10/docs/specs/jni/index.html).
//!
//! The main philosofy of this library is to push as many errors to compile-time as possible
//! and panic whenever it's impossible to have a compile error.
// TODO: a complete example.

extern crate cesu8;
extern crate jni_sys;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;

mod attach_arguments;
mod init_arguments;
mod java_string;
mod jni;
mod methods;
mod primitives;
mod raw;
mod testing;
mod version;

pub use attach_arguments::AttachArguments;
pub use init_arguments::{InitArguments, JvmOption, JvmVerboseOption};
pub use jni::{Cast, Exception, JavaType, JavaVM, JniEnv, JniError, NoException};
pub use version::JniVersion;

pub mod java {
    pub mod lang {
        pub use jni::Class;
        pub use jni::Object;
        pub use jni::String;
        pub use jni::Throwable;
    }
}
