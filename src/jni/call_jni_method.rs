macro_rules! call_jni_method {
    ($env:expr, $method:ident) => {
        {
            let raw_env = $env.raw_env();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env)
        }
    };
    ($env:expr, $method:ident, $($argument:expr),*) => {
        {
            let raw_env = $env.raw_env();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env, $($argument),*)
        }
    };
}

// It's actually used.
#[allow(unused_macros)]
macro_rules! call_nullable_jni_method {
    ($env:expr, $method:ident, $token:expr, $($argument:expr),*) => {
        with_checked_exception($token, |token| {
            let result =
                call_jni_method!($env, $method, $($argument),*);
            from_nullable($env, result, token)
        })
    }
}
