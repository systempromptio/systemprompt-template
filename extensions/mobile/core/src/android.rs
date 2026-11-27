//! The JNI entry point the Android app calls once at start-up.
//!
//! The HTTP client verifies certificates with the system's verifier, which on
//! Android is reached through the JVM: it needs the virtual machine and an
//! application context before the first HTTPS request. The app's
//! `NativeLibrary` object declares the native method this module exports.

use jni::EnvUnowned;
use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JObject};

/// `io.systemprompt.mobile.NativeLibrary.initialisePlatform(Context)`. A
/// failure is thrown to the caller as a `RuntimeException`.
#[expect(
    unsafe_code,
    reason = "the JVM binds a native method by its unmangled symbol name"
)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_systemprompt_mobile_NativeLibrary_initialisePlatform<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    context: JObject<'local>,
) {
    env.with_env(|env| rustls_platform_verifier::android::init_with_env(env, context))
        .resolve::<ThrowRuntimeExAndDefault>();
}
