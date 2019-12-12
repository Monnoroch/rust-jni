// This file is textually included, not imported as a module.
// Thus we need to disable "dead code" warnings as these macros are
// actually used.

macro_rules! call_jni_method {
    ($env:expr, $method:ident) => {
        {
            let raw_env = $env.raw_env().as_ptr();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env)
        }
    };
    ($env:expr, $method:ident, $($argument:expr),*) => {
        {
            let raw_env = $env.raw_env().as_ptr();
            let jni_fn = ((**raw_env).$method).unwrap();
            jni_fn(raw_env, $($argument),*)
        }
    };
}

// It's actually used.
#[allow(unused_macros)]
macro_rules! call_jni_object_method {
    ($token:ident, $object:ident, $method:ident) => {
        call_jni_method!($token.env(), $method, $object.raw_object().as_ptr())
    };
    ($token:ident, $object:ident, $method:ident, $($argument:expr),*) => {
        call_jni_method!($token.env(), $method, $object.raw_object().as_ptr(), $($argument),*)
    };
}

// It's actually used.
#[allow(unused_macros)]
macro_rules! call_nullable_jni_method {
    ($token:expr, $method:ident, $($argument:expr),*) => {
        $token.with_owned(#[inline(always)] |token| {
            let result = call_jni_method!($token.env(), $method, $($argument),*);
            match NonNull::new(result) {
                None => CallOutcome::Err(token.exchange()),
                Some(result) => CallOutcome::Ok((result, token)),
            }
        })
    }
}
