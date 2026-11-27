package io.systemprompt.mobile.feature.signin

import uniffi.systemprompt_mobile.InstanceDiscovery
import uniffi.systemprompt_mobile.Instances
import uniffi.systemprompt_mobile.MobileException
import uniffi.systemprompt_mobile.diagnostics
import uniffi.systemprompt_mobile_app.libraryVersion

/** What the shared library is: the binding library's version and the core it was built from. */
data class LibraryVersions(val library: String, val core: String)

/**
 * The part of the generated API the instance screen calls. The view model
 * depends on this and not on the generated classes, so a JVM unit test can
 * stand in for the shared library, which only loads on a device.
 */
interface InstanceDirectory {
    fun versions(): LibraryVersions

    /** Reads the instance's discovery document; the instance becomes the active one. */
    @Throws(MobileException::class)
    suspend fun discover(address: String): InstanceDiscovery
}

/** The shared library itself. */
class BoundInstanceDirectory : InstanceDirectory {
    private val instances = Instances()

    override fun versions(): LibraryVersions =
        LibraryVersions(library = libraryVersion(), core = diagnostics().coreVersion)

    override suspend fun discover(address: String): InstanceDiscovery = instances.discover(address)
}
