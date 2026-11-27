package io.systemprompt.mobile

import android.content.Context

/**
 * The shared Rust library. [start] loads it and hands it the application
 * context, which its certificate verifier needs before the first HTTPS
 * request; nothing in the generated bindings may be called before it.
 *
 * The library exports [initialisePlatform] under this object's qualified name,
 * so moving or renaming the object means renaming that export in
 * extensions/mobile/core/src/android.rs.
 */
object NativeLibrary {
    private const val NAME = "systemprompt_mobile_app"

    fun start(context: Context) {
        System.loadLibrary(NAME)
        initialisePlatform(context.applicationContext)
    }

    @JvmStatic
    private external fun initialisePlatform(context: Context)
}
