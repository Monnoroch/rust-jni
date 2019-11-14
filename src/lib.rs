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

#[cfg(test)]
#[macro_use]
pub mod testing;

mod attach_arguments;
mod class;
mod env;
mod error;
mod generate;
mod init_arguments;
mod java_string;
mod jni_bool;
mod method_calls;
mod methods;
mod native_method;
mod object;
mod primitives;
mod result;
mod string;
mod throwable;
mod token;
mod version;
mod vm;

pub use attach_arguments::AttachArguments;
pub use env::JniEnv;
pub use error::JniError;
pub use init_arguments::{InitArguments, JvmOption, JvmVerboseOption};
pub use result::JavaResult;
pub use token::{Exception, NoException};
pub use version::JniVersion;
pub use vm::{Cast, JavaType, JavaVM, JavaVMRef};

pub mod java {
    pub mod lang {
        pub use crate::class::Class;
        pub use crate::object::Object;
        pub use crate::string::String;
        pub use crate::throwable::Throwable;
    }
}

/// Tools used by the Java class wrapper code generator.
///
/// SHOULD NOT BE USED MANUALLY.
#[doc(hidden)]
pub mod __generator {
    pub use crate::method_calls::call_constructor;
    pub use crate::method_calls::call_method;
    pub use crate::method_calls::call_static_method;
    pub use crate::native_method::native_method_wrapper;
    pub use crate::native_method::test_from_jni_type;
    pub use crate::native_method::test_jni_argument_type;
    pub use crate::vm::FromJni;
    pub use crate::vm::ToJni;
}
