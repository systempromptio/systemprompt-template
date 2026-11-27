package io.systemprompt.mobile

import android.app.Application
import io.systemprompt.mobile.feature.signin.BoundInstanceDirectory
import io.systemprompt.mobile.feature.signin.InstanceDirectory

/** Starts the shared Rust library before anything else, and owns what lives as long as the process. */
class MobileApplication : Application() {
    /** The instances this app knows, held by the shared library. */
    val instances: InstanceDirectory by lazy { BoundInstanceDirectory() }

    override fun onCreate() {
        super.onCreate()
        NativeLibrary.start(this)
    }
}
