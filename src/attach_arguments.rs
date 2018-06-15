/// Arguments for attaching a thread to the JVM.
///
/// [JNI documentation](https://docs.oracle.com/javase/9/docs/specs/jni/invocation.html#attachcurrentthread)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachArguments<'a> {
    thread_name: Option<&'a str>,
    // TODO(#1): support thread groups.
}

impl<'a> AttachArguments<'a> {
    /// Create attach arguments with the default thread name.
    pub fn new() -> Self {
        AttachArguments { thread_name: None }
    }

    /// Create attach arguments with a specified thread name.
    pub fn named(thread_name: &'a str) -> Self {
        AttachArguments {
            thread_name: Some(thread_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AttachArguments;

    #[test]
    fn new() {
        assert_eq!(
            AttachArguments::new(),
            AttachArguments { thread_name: None }
        );
    }

    #[test]
    fn named() {
        assert_eq!(
            AttachArguments::named("test-name"),
            AttachArguments {
                thread_name: Some("test-name")
            }
        );
    }
}
