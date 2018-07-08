use jni::ToJni;
use jni_sys;
use raw::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::slice;
use version::{self, JniVersion};

/// Verbose options for starting a Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JvmVerboseOption {
    /// Verbose class option.
    ///
    /// Passed to the JVM as `-verbose:class`.
    Class,
    /// Verbose GC option.
    ///
    /// Passed to the JVM as `-verbose:gc`.
    Gc,
    /// Verbose JNI option.
    ///
    /// Passed to the JVM as `-verbose:jni`.
    Jni,
}

fn verbose_option_to_string(option: &JvmVerboseOption) -> &'static str {
    match option {
        JvmVerboseOption::Class => "class",
        JvmVerboseOption::Gc => "gc",
        JvmVerboseOption::Jni => "jni",
    }
}

#[cfg(test)]
mod verbose_option_to_string_tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(verbose_option_to_string(&JvmVerboseOption::Class), "class");
        assert_eq!(verbose_option_to_string(&JvmVerboseOption::Gc), "gc");
        assert_eq!(verbose_option_to_string(&JvmVerboseOption::Jni), "jni");
    }
}

/// Options for starting a Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
// TODO(#13): support vfprintf, exit, abort options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JvmOption {
    /// Verbose option.
    ///
    /// Passed to the JVM as `-verbose:${verbose_option}`.
    Verbose(JvmVerboseOption),
    /// System property option string. Must have a key and a value.
    ///
    /// Is formatted as `-D{key}=${value}`.
    SystemProperty(String, String),
    /// Enable checking JNI calls.
    ///
    /// Passed to the JVM as `-check:jni`.
    CheckedJni,
    /// Unknown option.
    /// Needed for forward compability and to set custom options.
    /// The string value is passed to the JVM without change.
    Unknown(String),
}

impl JvmOption {
    /// Unsafe because one can pass a non-UTF-8 or non-null-terminated option string.
    unsafe fn from_raw(option: &jni_sys::JavaVMOption) -> Self {
        // TODO(#14): support platform encodings other than UTF-8.
        let option_string = CStr::from_ptr((*option).optionString).to_str().unwrap();
        let system_property_prefix = "-D";
        match option_string {
            "-verbose:gc" => JvmOption::Verbose(JvmVerboseOption::Gc),
            "-verbose:jni" => JvmOption::Verbose(JvmVerboseOption::Jni),
            "-verbose:class" => JvmOption::Verbose(JvmVerboseOption::Class),
            "-Xcheck:jni" => JvmOption::CheckedJni,
            option if option.starts_with(system_property_prefix) => {
                let parts: Vec<&str> = option
                    .split_at(system_property_prefix.len())
                    .1
                    .splitn(2, "=")
                    .collect();
                if parts.len() != 2 {
                    JvmOption::Unknown(option.to_owned())
                } else {
                    JvmOption::SystemProperty(parts[0].to_owned(), parts[1].to_owned())
                }
            }
            option => JvmOption::Unknown(option.to_owned()),
        }
    }
}

#[cfg(test)]
mod jvm_option_tests {
    use super::*;

    #[test]
    fn from_raw_checked_jni() {
        let option_string = CStr::from_bytes_with_nul(b"-Xcheck:jni\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::CheckedJni
        );
    }

    #[test]
    fn from_raw_verbose() {
        let option_string = CStr::from_bytes_with_nul(b"-verbose:jni\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::Verbose(JvmVerboseOption::Jni)
        );

        let option_string = CStr::from_bytes_with_nul(b"-verbose:gc\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::Verbose(JvmVerboseOption::Gc)
        );

        let option_string = CStr::from_bytes_with_nul(b"-verbose:class\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::Verbose(JvmVerboseOption::Class)
        );
    }

    #[test]
    fn from_raw_system_property() {
        let option_string = CStr::from_bytes_with_nul(b"-Dkey=value\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::SystemProperty("key".to_owned(), "value".to_owned())
        );
    }

    #[test]
    fn from_raw_unknown() {
        let option_string = CStr::from_bytes_with_nul(b"tyhb\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::Unknown("tyhb".to_owned())
        );

        let option_string = CStr::from_bytes_with_nul(b"-Dkey~value\0").unwrap();
        let option = &jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        };
        assert_eq!(
            unsafe { JvmOption::from_raw(option) },
            JvmOption::Unknown("-Dkey~value".to_owned())
        );
    }

    #[test]
    fn to_string() {
        assert_eq!(option_to_string(&JvmOption::CheckedJni), "-Xcheck:jni");
        assert_eq!(
            option_to_string(&JvmOption::Verbose(JvmVerboseOption::Gc)),
            "-verbose:gc"
        );
        assert_eq!(
            option_to_string(&JvmOption::Verbose(JvmVerboseOption::Jni)),
            "-verbose:jni"
        );
        assert_eq!(
            option_to_string(&JvmOption::Verbose(JvmVerboseOption::Class)),
            "-verbose:class"
        );
        assert_eq!(
            option_to_string(&JvmOption::SystemProperty(
                "key".to_owned(),
                "value".to_owned()
            )),
            "-Dkey=value"
        );
        assert_eq!(
            option_to_string(&JvmOption::Unknown("qwer".to_owned())),
            "qwer"
        );
    }
}

fn option_to_string(option: &JvmOption) -> String {
    match option {
        JvmOption::CheckedJni => "-Xcheck:jni".to_owned(),
        JvmOption::Verbose(option) => format!("-verbose:{}", verbose_option_to_string(option)),
        JvmOption::SystemProperty(key, value) => format!("-D{}={}", key, value),
        JvmOption::Unknown(value) => value.clone(),
    }
}

#[cfg(test)]
mod option_to_string_tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(option_to_string(&JvmOption::CheckedJni), "-Xcheck:jni");
        assert_eq!(
            option_to_string(&JvmOption::Verbose(JvmVerboseOption::Gc)),
            "-verbose:gc"
        );
        assert_eq!(
            option_to_string(&JvmOption::Verbose(JvmVerboseOption::Jni)),
            "-verbose:jni"
        );
        assert_eq!(
            option_to_string(&JvmOption::Verbose(JvmVerboseOption::Class)),
            "-verbose:class"
        );
        assert_eq!(
            option_to_string(&JvmOption::SystemProperty(
                "key".to_owned(),
                "value".to_owned()
            )),
            "-Dkey=value"
        );
        assert_eq!(
            option_to_string(&JvmOption::Unknown("qwer".to_owned())),
            "qwer"
        );
    }
}

/// Arguments for creating a Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
///
/// # Example
/// ```
/// use rust_jni::{InitArguments, JniVersion, JvmOption, JvmVerboseOption};
///
/// let options = InitArguments::get_default(JniVersion::V8).unwrap()
///     .with_option(JvmOption::Unknown("-Xgc:parallel".to_owned()))
///     .with_option(JvmOption::Verbose(JvmVerboseOption::Gc));
///
/// assert_eq!(options.version(), JniVersion::V8);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitArguments {
    version: JniVersion,
    options: Vec<JvmOption>,
    ignore_unrecognized: bool,
}

impl InitArguments {
    /// Get default Java VM init arguments for a JNI version.
    /// If the requested JNI version is not supported, returns
    /// [`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None).
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getdefaultjavavminitargs)
    pub fn get_default(version: JniVersion) -> Option<Self> {
        let arguments = Self::get_default_or_closest_supported(version);
        if arguments.version == version {
            Some(arguments)
        } else {
            None
        }
    }

    /// Get default Java VM init arguments for a JNI version.
    /// If the requested JNI version is not supported, returns default arguments for the closest
    /// supported JNI version. The new version can be obtained with the
    /// [`InitArguments::version()`](struct.InitArguments.html#method.version) method.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getdefaultjavavminitargs)
    pub fn get_default_or_closest_supported(version: JniVersion) -> Self {
        let mut raw_arguments = jni_sys::JavaVMInitArgs {
            version: version::to_raw(version),
            nOptions: 0,
            options: ptr::null_mut(),
            ignoreUnrecognized: jni_sys::JNI_FALSE,
        };
        // Safe because we pass a pointer to a correct data structure.
        unsafe {
            // It is fine if the requested version is not supported, we'll just use
            // a supported one.
            JNI_GetDefaultJavaVMInitArgs(
                &mut raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
            );
        }
        // Safe because raw arguments were correctly initialized by the
        // `JNI_GetDefaultJavaVMInitArgs`.
        unsafe { Self::from_raw(&raw_arguments).with_option(JvmOption::CheckedJni) }
    }

    /// Unsafe because one can pass incorrect options.
    unsafe fn from_raw(raw_arguments: &jni_sys::JavaVMInitArgs) -> InitArguments {
        let options = slice::from_raw_parts(raw_arguments.options, raw_arguments.nOptions as usize)
            .iter()
            .map(|value| JvmOption::from_raw(value))
            .collect();
        InitArguments {
            version: version::from_raw(raw_arguments.version),
            ignore_unrecognized: to_bool(raw_arguments.ignoreUnrecognized),
            options,
        }
    }

    /// Get default init arguments for the latest supported JNI version.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getdefaultjavavminitargs)
    pub fn get_latest_default() -> Self {
        Self::get_default_or_closest_supported(JniVersion::V8)
    }

    /// Add init options to the Java VM init arguments.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn with_options(mut self, options: &[JvmOption]) -> Self {
        self.options.extend_from_slice(options);
        self
    }

    /// Add an init option to the Java VM init arguments.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn with_option(self, option: JvmOption) -> Self {
        self.with_options(&[option])
    }

    /// Disable checking JNI calls for correctness.
    pub fn unchecked(self) -> Self {
        InitArguments {
            version: self.version,
            ignore_unrecognized: self.ignore_unrecognized,
            options: self.options
                .iter()
                .filter(|&option| *option != JvmOption::CheckedJni)
                .cloned()
                .collect(),
        }
    }

    /// Enable checking JNI calls for correctness.
    ///
    /// This is a default. Only needed to be called if checking JNI calls was explicitly disabled.
    pub fn checked(self) -> Self {
        self.with_option(JvmOption::CheckedJni)
    }

    /// Request for JVM to ignore unrecognized options on startup.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn ignore_unrecognized_options(mut self) -> Self {
        self.ignore_unrecognized = true;
        self
    }

    /// Request for JVM to fail in presence of unrecognized options on startup.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn fail_on_unrecognized_options(mut self) -> Self {
        self.ignore_unrecognized = false;
        self
    }

    /// Return the JNI version these arguments will request when creating a Java VM.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn version(&self) -> JniVersion {
        self.version
    }
}

/// A wrapper around `jni_sys::JavaVMInitArgs` with a lifetime to ensure
/// there's no access to freed memory.
pub struct RawInitArguments<'a> {
    pub raw_arguments: jni_sys::JavaVMInitArgs,
    _buffer: PhantomData<&'a Vec<CString>>,
}

pub fn to_raw<'a, 'b, 'c: 'a + 'b>(
    arguments: &InitArguments,
    strings_buffer: &'a mut Vec<CString>,
    options_buffer: &'b mut Vec<jni_sys::JavaVMOption>,
) -> RawInitArguments<'c> {
    *strings_buffer = arguments
        .options
        .iter()
        .map(|_| "")
        .map(CString::new)
        .map(Result::unwrap)
        .collect();
    *options_buffer = arguments
        .options
        .iter()
        .zip(strings_buffer.iter_mut())
        .map(|(option, ref mut buffer)| {
            // TODO(#14): support platform encodings other than UTF-8.
            let buffer: &mut CString = buffer;
            *buffer = CString::new(option_to_string(option)).unwrap();
            jni_sys::JavaVMOption {
                optionString: buffer.as_ptr() as *mut i8,
                extraInfo: ptr::null_mut(),
            }
        })
        .collect();
    RawInitArguments {
        raw_arguments: jni_sys::JavaVMInitArgs {
            version: version::to_raw(arguments.version),
            nOptions: options_buffer.len() as i32,
            options: options_buffer.as_mut_ptr(),
            // Safe because `bool` conversion is safe internally.
            ignoreUnrecognized: unsafe { bool::__to_jni(&arguments.ignore_unrecognized) },
        },
        _buffer: PhantomData::<&'c Vec<CString>>,
    }
}

#[cfg(test)]
pub unsafe fn from_raw(raw_arguments: &jni_sys::JavaVMInitArgs) -> InitArguments {
    InitArguments::from_raw(raw_arguments)
}

#[cfg(test)]
pub fn test(version: JniVersion) -> InitArguments {
    InitArguments {
        version: version,
        options: vec![],
        ignore_unrecognized: true,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn default_options() -> Vec<JvmOption> {
        vec![
            JvmOption::SystemProperty("key".to_owned(), "value".to_owned()),
            JvmOption::Unknown("qwer".to_owned()),
        ]
    }

    fn resulting_options() -> Vec<JvmOption> {
        vec![
            JvmOption::SystemProperty("key".to_owned(), "value".to_owned()),
            JvmOption::Unknown("qwer".to_owned()),
            JvmOption::CheckedJni,
        ]
    }

    pub fn default_args() -> InitArguments {
        InitArguments {
            version: JniVersion::V4,
            options: default_options(),
            ignore_unrecognized: false,
        }
    }

    fn check_arguments(version: JniVersion) {
        let actual_arguments = get_default_java_vm_init_args_call_input();
        assert_eq!(actual_arguments.version, version::to_raw(version));
        assert_eq!(actual_arguments.nOptions, 0);
        assert_eq!(actual_arguments.options, ptr::null_mut());
        assert_eq!(actual_arguments.ignoreUnrecognized, jni_sys::JNI_FALSE);
    }

    #[test]
    fn to_raw_test() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: default_options(),
            ignore_unrecognized: false,
        };
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let raw_arguments = to_raw(&arguments, &mut strings_buffer, &mut options_buffer);
        assert_eq!(
            raw_arguments.raw_arguments.version,
            version::to_raw(JniVersion::V4)
        );
        assert_eq!(raw_arguments.raw_arguments.nOptions, 2);
        assert_eq!(
            raw_arguments.raw_arguments.ignoreUnrecognized,
            jni_sys::JNI_FALSE
        );
        let raw_options = unsafe {
            slice::from_raw_parts(
                raw_arguments.raw_arguments.options,
                raw_arguments.raw_arguments.nOptions as usize,
            )
        };
        for (raw_option, option) in raw_options.iter().zip(default_options().into_iter()) {
            assert_eq!(option, unsafe { JvmOption::from_raw(raw_option) });
        }
    }

    #[test]
    fn get_default_supported() {
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let mut raw_arguments = to_raw(&default_args(), &mut strings_buffer, &mut options_buffer);
        let _locked = setup_get_default_java_vm_init_args_call(GetDefaultJavaVMInitArgsCall::new(
            &mut raw_arguments.raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
        ));
        assert_eq!(
            InitArguments::get_default(JniVersion::V4),
            Some(InitArguments {
                version: JniVersion::V4,
                options: resulting_options(),
                ignore_unrecognized: false
            })
        );
        check_arguments(JniVersion::V4);
    }

    #[test]
    fn get_default_unsupported() {
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let mut raw_arguments = to_raw(&default_args(), &mut strings_buffer, &mut options_buffer);
        let _locked = setup_get_default_java_vm_init_args_call(GetDefaultJavaVMInitArgsCall::new(
            &mut raw_arguments.raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
        ));
        assert_eq!(InitArguments::get_default(JniVersion::V1), None);
        check_arguments(JniVersion::V1);
    }

    #[test]
    fn get_default_or_closest_supported() {
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let mut raw_arguments = to_raw(&default_args(), &mut strings_buffer, &mut options_buffer);
        let _locked = setup_get_default_java_vm_init_args_call(GetDefaultJavaVMInitArgsCall::new(
            &mut raw_arguments.raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
        ));
        assert_eq!(
            InitArguments::get_default_or_closest_supported(JniVersion::V1),
            InitArguments {
                version: JniVersion::V4,
                options: resulting_options(),
                ignore_unrecognized: false
            }
        );
        check_arguments(JniVersion::V1);
    }

    #[test]
    fn get_latest_default() {
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let mut raw_arguments = to_raw(&default_args(), &mut strings_buffer, &mut options_buffer);
        let _locked = setup_get_default_java_vm_init_args_call(GetDefaultJavaVMInitArgsCall::new(
            &mut raw_arguments.raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
        ));
        assert_eq!(
            InitArguments::get_latest_default(),
            InitArguments {
                version: JniVersion::V4,
                options: resulting_options(),
                ignore_unrecognized: false
            }
        );
        check_arguments(JniVersion::V8);
    }

    #[test]
    fn with_options() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![JvmOption::CheckedJni],
            ignore_unrecognized: false,
        };
        assert_eq!(
            arguments.with_options(&[
                JvmOption::Verbose(JvmVerboseOption::Gc),
                JvmOption::Unknown("test".to_owned()),
            ]),
            InitArguments {
                version: JniVersion::V4,
                options: vec![
                    JvmOption::CheckedJni,
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                    JvmOption::Unknown("test".to_owned()),
                ],
                ignore_unrecognized: false,
            }
        );
    }

    #[test]
    fn with_option() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![JvmOption::CheckedJni],
            ignore_unrecognized: false,
        };
        assert_eq!(
            arguments.with_option(JvmOption::Verbose(JvmVerboseOption::Gc)),
            InitArguments {
                version: JniVersion::V4,
                options: vec![
                    JvmOption::CheckedJni,
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                ],
                ignore_unrecognized: false,
            }
        );
    }

    #[test]
    fn unchecked() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![
                JvmOption::Verbose(JvmVerboseOption::Gc),
                JvmOption::CheckedJni,
            ],
            ignore_unrecognized: false,
        };
        assert_eq!(
            arguments.unchecked(),
            InitArguments {
                version: JniVersion::V4,
                options: vec![JvmOption::Verbose(JvmVerboseOption::Gc)],
                ignore_unrecognized: false,
            }
        );
    }

    #[test]
    fn checked() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![JvmOption::Verbose(JvmVerboseOption::Gc)],
            ignore_unrecognized: false,
        };
        assert_eq!(
            arguments.checked(),
            InitArguments {
                version: JniVersion::V4,
                options: vec![
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                    JvmOption::CheckedJni,
                ],
                ignore_unrecognized: false,
            }
        );
    }

    #[test]
    fn ignore_unrecognized_options() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![],
            ignore_unrecognized: false,
        };
        assert_eq!(
            arguments.ignore_unrecognized_options(),
            InitArguments {
                version: JniVersion::V4,
                options: vec![],
                ignore_unrecognized: true,
            }
        );
    }

    #[test]
    fn fail_on_unrecognized_options() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![],
            ignore_unrecognized: true,
        };
        assert_eq!(
            arguments.fail_on_unrecognized_options(),
            InitArguments {
                version: JniVersion::V4,
                options: vec![],
                ignore_unrecognized: false,
            }
        );
    }

    #[test]
    fn version() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![],
            ignore_unrecognized: true,
        };
        assert_eq!(arguments.version(), JniVersion::V4);
    }
}

fn to_bool(value: jni_sys::jboolean) -> bool {
    match value {
        jni_sys::JNI_TRUE => true,
        jni_sys::JNI_FALSE => false,
        value => panic!("Unexpected jboolean value {}", value),
    }
}
