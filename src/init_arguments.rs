use crate::error::JniError;
use crate::jni_bool;
use crate::version::JniVersion;
use cfg_if::cfg_if;
use jni_sys;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::slice;

/// Verbose options for starting a Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl JvmVerboseOption {
    fn to_string(&self) -> &'static str {
        match self {
            JvmVerboseOption::Class => "class",
            JvmVerboseOption::Gc => "gc",
            JvmVerboseOption::Jni => "jni",
        }
    }
}

#[cfg(test)]
mod verbose_option_to_string_tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(JvmVerboseOption::Class.to_string(), "class");
        assert_eq!(JvmVerboseOption::Gc.to_string(), "gc");
        assert_eq!(JvmVerboseOption::Jni.to_string(), "jni");
    }
}

/// Options for starting a Java VM.
///
/// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JvmOption {
    /// Unknown option.
    /// Needed for forward compability and to set custom options.
    /// The string value is passed to the JVM without change.
    Unknown(String),
    /// Enable checking JNI calls.
    ///
    /// Passed to the JVM as `-check:jni`.
    CheckedJni,
    /// Verbose option.
    ///
    /// Passed to the JVM as `-verbose:${verbose_option}`.
    Verbose(JvmVerboseOption),
}

impl JvmOption {
    /// Unsafe because one can pass a non-UTF-8 or non-null-terminated option string.
    unsafe fn from_raw(option: &jni_sys::JavaVMOption) -> Self {
        // TODO(#14): support platform encodings other than UTF-8.
        let option_string = CStr::from_ptr((*option).optionString).to_str().unwrap();
        match option_string {
            "-Xcheck:jni" => JvmOption::CheckedJni,
            "-verbose:gc" => JvmOption::Verbose(JvmVerboseOption::Gc),
            "-verbose:jni" => JvmOption::Verbose(JvmVerboseOption::Jni),
            "-verbose:class" => JvmOption::Verbose(JvmVerboseOption::Class),
            option => JvmOption::Unknown(option.to_owned()),
        }
    }

    fn to_string(&self) -> CString {
        match self {
            JvmOption::Unknown(value) => CString::new(value.as_str()),
            JvmOption::CheckedJni => CString::new("-Xcheck:jni"),
            JvmOption::Verbose(option) => CString::new(format!("-verbose:{}", option.to_string())),
        }
        .unwrap()
    }
}

#[cfg(test)]
mod jvm_option_tests {
    use super::*;

    fn raw_vm_option(option_string: &CStr) -> jni_sys::JavaVMOption {
        jni_sys::JavaVMOption {
            optionString: option_string.as_ptr() as *mut i8,
            extraInfo: ptr::null_mut(),
        }
    }

    #[test]
    fn from_raw_unknown() {
        let option_string = CStr::from_bytes_with_nul(b"tyhb\0").unwrap();
        let option = raw_vm_option(&option_string);
        assert_eq!(
            unsafe { JvmOption::from_raw(&option) },
            JvmOption::Unknown("tyhb".to_owned())
        );
    }

    #[test]
    fn from_raw_checked_jni() {
        let option_string = CStr::from_bytes_with_nul(b"-Xcheck:jni\0").unwrap();
        let option = raw_vm_option(&option_string);
        assert_eq!(
            unsafe { JvmOption::from_raw(&option) },
            JvmOption::CheckedJni
        );
    }

    #[test]
    fn from_raw_verbose() {
        let option_string = CStr::from_bytes_with_nul(b"-verbose:jni\0").unwrap();
        let option = raw_vm_option(&option_string);
        assert_eq!(
            unsafe { JvmOption::from_raw(&option) },
            JvmOption::Verbose(JvmVerboseOption::Jni)
        );

        let option_string = CStr::from_bytes_with_nul(b"-verbose:gc\0").unwrap();
        let option = raw_vm_option(&option_string);
        assert_eq!(
            unsafe { JvmOption::from_raw(&option) },
            JvmOption::Verbose(JvmVerboseOption::Gc)
        );

        let option_string = CStr::from_bytes_with_nul(b"-verbose:class\0").unwrap();
        let option = raw_vm_option(&option_string);
        assert_eq!(
            unsafe { JvmOption::from_raw(&option) },
            JvmOption::Verbose(JvmVerboseOption::Class)
        );
    }
}

#[cfg(test)]
mod option_to_string_tests {
    use super::*;

    #[test]
    fn to_string_unknown() {
        assert_eq!(
            JvmOption::Unknown("qwer".into()).to_string(),
            CString::new("qwer").unwrap()
        );
    }

    #[test]
    fn to_string_checked_jni() {
        assert_eq!(
            JvmOption::CheckedJni.to_string(),
            CString::new("-Xcheck:jni").unwrap()
        );
    }

    #[test]
    fn to_string_verbose() {
        assert_eq!(
            JvmOption::Verbose(JvmVerboseOption::Gc).to_string(),
            CString::new("-verbose:gc").unwrap()
        );
        assert_eq!(
            JvmOption::Verbose(JvmVerboseOption::Jni).to_string(),
            CString::new("-verbose:jni").unwrap()
        );
        assert_eq!(
            JvmOption::Verbose(JvmVerboseOption::Class).to_string(),
            CString::new("-verbose:class").unwrap()
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
/// let options = InitArguments::default()
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

/// Default JVM init arguments.
///
/// Defaut argumets are conservative towards safety and use JDK 8 as the most common one.
///
/// [JNI documentation](https://docs.oracle.com/en/java/javase/11/docs/specs/jni/invocation.html#jni_createjavavm)
impl Default for InitArguments {
    fn default() -> Self {
        InitArguments {
            version: JniVersion::V8,
            options: vec![],
            ignore_unrecognized: true,
        }
        // We enable CheckedJni by default for exatra safety.
        // It can always be explicitly disabled with .unchecked().
        .checked()
        // We enable failing on unrecognized JVM aruments by default for exatra safety.
        // It can always be explicitly disabled with .ignore_unrecognized_options().
        .fail_on_unrecognized_options()
    }
}

impl InitArguments {
    /// Get default init arguments for the latest supported JNI version.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getdefaultjavavminitargs)
    pub fn get_latest_default() -> Result<Self, JniError> {
        Self::get_default(JniVersion::V10)
    }

    /// Get default Java VM init arguments for a JNI version.
    /// If the requested JNI version is not supported, returns [`JniError`](enum.JniError.html).
    ///
    /// Unlike [`InitArguments::default()`](struct.InitArguments.html#impl-Default), gets the defaut arguments
    /// from a JNI call.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getdefaultjavavminitargs)
    pub fn get_default(version: JniVersion) -> Result<Self, JniError> {
        let mut raw_arguments = jni_sys::JavaVMInitArgs {
            version: version.to_raw(),
            nOptions: 0,
            options: ptr::null_mut(),
            ignoreUnrecognized: jni_sys::JNI_FALSE,
        };
        // Safe because we pass a pointer to a correct data structure.
        unsafe {
            // It is fine if the requested version is not supported, we'll just use
            // a supported one.
            let error = JniError::from_raw(JNI_GetDefaultJavaVMInitArgs(
                &mut raw_arguments as *mut jni_sys::JavaVMInitArgs as *mut c_void,
            ));
            if error.is_some() {
                return Err(error.unwrap());
            }
        }
        // Safe because raw arguments were correctly initialized by the
        // `JNI_GetDefaultJavaVMInitArgs`.
        let init_arguments = unsafe { Self::from_raw(&raw_arguments) };
        // Version must be the same as the one specified before.
        // See [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_getdefaultjavavminitargs)
        // for details.
        if version != init_arguments.version {
            return Err(JniError::UnsupportedVersion);
        }
        Ok(init_arguments
            // We enable CheckedJni by default for exatra safety.
            // It can always be explicitly disabled with .unchecked().
            .checked()
            // We enable failing on unrecognized JVM aruments by default for exatra safety.
            // It can always be explicitly disabled with .ignore_unrecognized_options().
            .fail_on_unrecognized_options())
    }

    /// Unsafe because one can pass incorrect options.
    pub(crate) unsafe fn from_raw(raw_arguments: &jni_sys::JavaVMInitArgs) -> InitArguments {
        let options = slice::from_raw_parts(raw_arguments.options, raw_arguments.nOptions as usize)
            .iter()
            .map(|value| JvmOption::from_raw(value))
            .collect();
        InitArguments {
            version: JniVersion::from_raw(raw_arguments.version),
            ignore_unrecognized: jni_bool::to_rust(raw_arguments.ignoreUnrecognized),
            options,
        }
    }

    /// Set requested JNI version in the Java VM init arguments.
    ///
    /// [JNI documentation](https://docs.oracle.com/javase/10/docs/specs/jni/invocation.html#jni_createjavavm)
    pub fn with_version(mut self, version: JniVersion) -> Self {
        self.version = version;
        self
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
            options: self
                .options
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

#[cfg(test)]
pub mod init_arguments_manipulation_tests {
    use super::*;

    pub fn default_args() -> InitArguments {
        InitArguments {
            version: JniVersion::V4,
            options: vec![],
            ignore_unrecognized: false,
        }
    }

    #[test]
    fn default() {
        assert_eq!(
            InitArguments::default(),
            InitArguments {
                version: JniVersion::V8,
                options: vec![JvmOption::CheckedJni],
                ignore_unrecognized: false,
            }
        );
    }

    #[test]
    fn with_version() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            ..default_args()
        };
        assert_eq!(
            arguments.with_version(JniVersion::V8),
            InitArguments {
                version: JniVersion::V8,
                ..default_args()
            }
        );
    }

    #[test]
    fn with_options() {
        let arguments = InitArguments {
            options: vec![JvmOption::CheckedJni],
            ..default_args()
        };
        assert_eq!(
            arguments.with_options(&[
                JvmOption::Verbose(JvmVerboseOption::Gc),
                JvmOption::Unknown("test".to_owned()),
            ]),
            InitArguments {
                options: vec![
                    JvmOption::CheckedJni,
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                    JvmOption::Unknown("test".to_owned()),
                ],
                ..default_args()
            }
        );
    }

    #[test]
    fn with_option() {
        let arguments = InitArguments {
            options: vec![JvmOption::CheckedJni],
            ..default_args()
        };
        assert_eq!(
            arguments.with_option(JvmOption::Verbose(JvmVerboseOption::Gc)),
            InitArguments {
                options: vec![
                    JvmOption::CheckedJni,
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                ],
                ..default_args()
            }
        );
    }

    #[test]
    fn unchecked() {
        let arguments = InitArguments {
            options: vec![
                JvmOption::Verbose(JvmVerboseOption::Gc),
                JvmOption::CheckedJni,
            ],
            ..default_args()
        };
        assert_eq!(
            arguments.unchecked(),
            InitArguments {
                options: vec![JvmOption::Verbose(JvmVerboseOption::Gc)],
                ..default_args()
            }
        );
    }

    #[test]
    fn checked() {
        let arguments = InitArguments {
            options: vec![JvmOption::Verbose(JvmVerboseOption::Gc)],
            ..default_args()
        };
        assert_eq!(
            arguments.checked(),
            InitArguments {
                options: vec![
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                    JvmOption::CheckedJni,
                ],
                ..default_args()
            }
        );
    }

    #[test]
    fn ignore_unrecognized_options() {
        let arguments = InitArguments {
            ignore_unrecognized: false,
            ..default_args()
        };
        assert_eq!(
            arguments.ignore_unrecognized_options(),
            InitArguments {
                ignore_unrecognized: true,
                ..default_args()
            }
        );
    }

    #[test]
    fn fail_on_unrecognized_options() {
        let arguments = InitArguments {
            ignore_unrecognized: true,
            ..default_args()
        };
        assert_eq!(
            arguments.fail_on_unrecognized_options(),
            InitArguments {
                ignore_unrecognized: false,
                ..default_args()
            }
        );
    }

    #[test]
    fn version() {
        let arguments = InitArguments {
            version: JniVersion::V6,
            ..default_args()
        };
        assert_eq!(arguments.version(), JniVersion::V6);
    }
}

#[cfg(test)]
pub(crate) mod init_arguments_creation_tests {
    use super::*;
    use serial_test_derive::serial;

    pub(crate) fn default_args() -> InitArguments {
        InitArguments {
            version: JniVersion::V4,
            options: vec![],
            ignore_unrecognized: false,
        }
    }

    #[test]
    #[serial]
    fn get_default() {
        let resulting_arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![
                JvmOption::Unknown("qwer".to_owned()),
                JvmOption::Verbose(JvmVerboseOption::Gc),
            ],
            ..default_args()
        };
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let raw_resulting_arguments =
            resulting_arguments.to_raw(&mut strings_buffer, &mut options_buffer);

        let mock = ffi::mock::JNI_GetDefaultJavaVMInitArgs_context();
        mock.expect()
            .times(1)
            .withf(move |arguments: &*mut ::std::os::raw::c_void| {
                let arguments = *arguments as *mut jni_sys::JavaVMInitArgs;
                // We know that this pointer points to a valid value.
                match unsafe { arguments.as_mut() } {
                    None => false,
                    Some(arguments) => {
                        if arguments.version != JniVersion::V4.to_raw()
                            || arguments.nOptions != 0
                            || arguments.options != ptr::null_mut()
                            || arguments.ignoreUnrecognized != jni_sys::JNI_FALSE
                        {
                            false
                        } else {
                            *arguments = raw_resulting_arguments.raw_arguments;
                            true
                        }
                    }
                }
            })
            .return_const(jni_sys::JNI_OK);
        assert_eq!(
            InitArguments::get_default(JniVersion::V4),
            Ok(InitArguments {
                version: JniVersion::V4,
                options: vec![
                    JvmOption::Unknown("qwer".to_owned()),
                    JvmOption::Verbose(JvmVerboseOption::Gc),
                    JvmOption::CheckedJni,
                ],
                ..default_args()
            })
        );
    }

    #[test]
    #[serial]
    fn get_default_error() {
        let mock = ffi::mock::JNI_GetDefaultJavaVMInitArgs_context();
        mock.expect().times(1).return_const(jni_sys::JNI_ERR);
        assert_eq!(
            InitArguments::get_default(JniVersion::V4),
            Err(JniError::Unknown(jni_sys::JNI_ERR))
        );
    }

    #[test]
    #[serial]
    fn get_default_changed_version() {
        let resulting_arguments = InitArguments {
            version: JniVersion::V4,
            ..default_args()
        };
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let raw_resulting_arguments =
            resulting_arguments.to_raw(&mut strings_buffer, &mut options_buffer);

        let mock = ffi::mock::JNI_GetDefaultJavaVMInitArgs_context();
        mock.expect()
            .times(1)
            .withf(move |arguments: &*mut ::std::os::raw::c_void| {
                let arguments = *arguments as *mut jni_sys::JavaVMInitArgs;
                // We know that this pointer points to a valid value.
                match unsafe { arguments.as_mut() } {
                    None => false,
                    Some(arguments) => {
                        if arguments.version != JniVersion::V8.to_raw() {
                            false
                        } else {
                            *arguments = raw_resulting_arguments.raw_arguments;
                            true
                        }
                    }
                }
            })
            .return_const(jni_sys::JNI_OK);
        assert_eq!(
            InitArguments::get_default(JniVersion::V8),
            Err(JniError::UnsupportedVersion)
        );
    }

    #[test]
    #[serial]
    fn get_latest_default() {
        let resulting_arguments = InitArguments {
            version: JniVersion::V10,
            ..default_args()
        };
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let raw_resulting_arguments =
            resulting_arguments.to_raw(&mut strings_buffer, &mut options_buffer);

        let mock = ffi::mock::JNI_GetDefaultJavaVMInitArgs_context();
        mock.expect()
            .times(1)
            .withf(move |arguments: &*mut ::std::os::raw::c_void| {
                let arguments = *arguments as *mut jni_sys::JavaVMInitArgs;
                // We know that this pointer points to a valid value.
                match unsafe { arguments.as_mut() } {
                    None => false,
                    Some(arguments) => {
                        if arguments.version != JniVersion::V10.to_raw() {
                            false
                        } else {
                            *arguments = raw_resulting_arguments.raw_arguments;
                            true
                        }
                    }
                }
            })
            .return_const(jni_sys::JNI_OK);
        assert_eq!(
            InitArguments::get_latest_default(),
            Ok(InitArguments {
                version: JniVersion::V10,
                options: vec![JvmOption::CheckedJni],
                ..default_args()
            })
        );
    }
}

/// A wrapper around `jni_sys::JavaVMInitArgs` with a lifetime to ensure
/// there's no access to freed memory.
pub(crate) struct RawInitArguments<'a> {
    pub raw_arguments: jni_sys::JavaVMInitArgs,
    _buffer: PhantomData<&'a Vec<CString>>,
}

/// Test implementation of Send to allow passing
/// these objects to mock arguments matchers.
#[cfg(test)]
unsafe impl<'a> Send for RawInitArguments<'a> {}

impl InitArguments {
    pub(crate) fn to_raw<'a, 'b, 'c: 'a + 'b>(
        &self,
        strings_buffer: &'a mut Vec<CString>,
        options_buffer: &'b mut Vec<jni_sys::JavaVMOption>,
    ) -> RawInitArguments<'c> {
        *strings_buffer = self
            .options
            .iter()
            .map(|_| "")
            .map(CString::new)
            .map(Result::unwrap)
            .collect();
        *options_buffer = self
            .options
            .iter()
            .zip(strings_buffer.iter_mut())
            .map(|(option, ref mut buffer)| {
                // TODO(#14): support platform encodings other than UTF-8.
                let buffer: &mut CString = buffer;
                *buffer = option.to_string();
                jni_sys::JavaVMOption {
                    optionString: buffer.as_ptr() as *mut i8,
                    extraInfo: ptr::null_mut(),
                }
            })
            .collect();
        RawInitArguments {
            raw_arguments: jni_sys::JavaVMInitArgs {
                version: self.version.to_raw(),
                nOptions: options_buffer.len() as i32,
                options: options_buffer.as_mut_ptr(),
                ignoreUnrecognized: jni_bool::to_jni(self.ignore_unrecognized),
            },
            _buffer: PhantomData::<&'c Vec<CString>>,
        }
    }
}

#[cfg(test)]
mod init_arguments_to_raw_tests {
    use super::*;

    #[test]
    fn to_raw_test() {
        let arguments = InitArguments {
            version: JniVersion::V4,
            options: vec![
                JvmOption::Unknown("qwer".to_owned()),
                JvmOption::Verbose(JvmVerboseOption::Gc),
            ],
            ignore_unrecognized: false,
        };
        let mut strings_buffer = vec![];
        let mut options_buffer = vec![];
        let raw_arguments = arguments.to_raw(&mut strings_buffer, &mut options_buffer);
        assert_eq!(raw_arguments.raw_arguments.version, JniVersion::V4.to_raw());
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
        assert_eq!(raw_options.len(), 2);
        for (raw_option, option) in raw_options.iter().zip(arguments.options.into_iter()) {
            assert_eq!(option, unsafe { JvmOption::from_raw(raw_option) });
        }
    }
}

#[cfg(test)]
// JNI API.
#[allow(non_snake_case)]
mod ffi {
    use mockall::*;

    #[automock(mod mock;)]
    // We're not using the non-test function.
    #[allow(dead_code)]
    extern "C" {
        pub fn JNI_GetDefaultJavaVMInitArgs(
            arguments: *mut ::std::os::raw::c_void,
        ) -> jni_sys::jint;
    }
}

cfg_if! {
    if #[cfg(test)] {
        use self::ffi::mock::JNI_GetDefaultJavaVMInitArgs;
    } else if #[cfg(all(not(test), feature = "libjvm"))] {
        use jni_sys::JNI_GetDefaultJavaVMInitArgs;
    } else if #[cfg(all(not(test), not(feature = "libjvm")))] {
        /// This is a stub for when we can't link to libjvm.
        // JNI API.
        #[allow(non_snake_case)]
        pub unsafe extern "system" fn JNI_GetDefaultJavaVMInitArgs(
            _arguments: *mut c_void,
        ) -> jni_sys::jint {
            jni_sys::JNI_OK
        }
    }
}
