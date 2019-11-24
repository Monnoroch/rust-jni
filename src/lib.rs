//! # A library for safe interoperation between Rust and Java
//!
//! [`rust-jni`](index.html) provides tools to safely make calls from Rust to Java
//! and from Java to Rust using
//! [JNI](https://docs.oracle.com/javase/10/docs/specs/jni/index.html).
//!
//! See also [JNI documentation from Android](https://developer.android.com/training/articles/perf-jni).
//!
//! The main philosofy of this library is to push as many errors to compile-time as possible
//! and panic whenever it's impossible to have a compile error.
// TODO: a complete example.

#![feature(fn_traits, unboxed_closures)]

#[cfg(test)]
#[macro_use]
pub mod testing;

mod attach_arguments;
mod class;
mod classes;
mod env;
mod error;
mod init_arguments;
mod java_class;
mod java_methods;
mod java_primitives;
mod java_string;
mod jni_bool;
mod jni_methods;
mod jni_types;
mod native_method;
mod object;
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
pub use java_class::{JavaClassExt, NullableJavaClassExt};
pub use java_methods::{
    call_constructor, call_method, call_static_method, FromObject, JniSignature,
};
pub use native_method::{native_method_implementation, static_native_method_implementation};
pub use result::JavaResult;
pub use token::{ConsumedNoException, Exception, NoException};
pub use version::JniVersion;
pub use vm::{JavaVM, JavaVMRef};

pub mod java {
    pub mod lang {
        //! Package java.lang.
        //!
        //! Provides classes that are fundamental to the design of the Java programming language.
        //!
        //! [`java.lang` javadoc](https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/lang/package-summary.html)

        pub use crate::class::Class;
        pub use crate::classes::exception::Exception;
        pub use crate::classes::null_pointer_exception::NullPointerException;
        pub use crate::object::Object;
        pub use crate::string::String;
        pub use crate::throwable::Throwable;
    }
}
