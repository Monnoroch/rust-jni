//! # A library for safe interoperation between Rust and Java
//!
//! [`rust-jni`](index.html) provides tools to safely make calls from Rust to Java
//! and from Java to Rust using
//! [JNI](https://docs.oracle.com/javase/10/docs/specs/jni/index.html).
//!
//! The main philosofy of this library is to push as many errors to compile-time as possible
//! and panic whenever it's impossible to have a compile error.
// TODO: a complete example.

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

mod attach_arguments;
mod generate;
mod init_arguments;
mod java_string;
mod jni;
mod raw;
mod version;

pub use attach_arguments::AttachArguments;
pub use init_arguments::{InitArguments, JvmOption, JvmVerboseOption};
pub use jni::{Cast, Exception, JavaResult, JavaType, JavaVM, JniEnv, JniError, NoException};
pub use version::JniVersion;

pub mod java {
    pub mod lang {
        pub use crate::jni::class::Class;
        pub use crate::jni::string::String;
        pub use crate::jni::throwable::Throwable;
        pub use crate::jni::Object;
    }
}

/// Tools used by the Java class wrapper code generator.
///
/// SHOULD NOT BE USED MANUALLY.
#[doc(hidden)]
pub mod __generator {
    pub use crate::jni::method_calls::call_constructor;
    pub use crate::jni::method_calls::call_method;
    pub use crate::jni::method_calls::call_static_method;
    pub use crate::jni::native_method::native_method_wrapper;
    pub use crate::jni::native_method::test_from_jni_type;
    pub use crate::jni::native_method::test_jni_argument_type;
    pub use crate::jni::FromJni;
    pub use crate::jni::ToJni;
}
