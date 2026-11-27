package io.systemprompt.mobile.feature.signin

import androidx.annotation.StringRes
import io.systemprompt.mobile.R
import uniffi.systemprompt_mobile.AppRelease
import uniffi.systemprompt_mobile.InstanceDiscovery
import uniffi.systemprompt_mobile.MobileException

/** Everything the instance screen draws. */
data class InstanceViewState(val versions: LibraryVersions, val discovery: DiscoveryState = DiscoveryState.Empty)

/** Where the screen is with the address the person entered. */
sealed interface DiscoveryState {
    /** Nothing asked yet: "enter an instance". */
    data object Empty : DiscoveryState

    data object Discovering : DiscoveryState

    data class Found(val instance: InstanceDiscovery) : DiscoveryState {
        val android: AppRelease? get() = instance.android
        val ios: AppRelease? get() = instance.ios
    }

    /** A message from the string catalogue and the one value it is formatted with, when it takes one. */
    data class Failed(@param:StringRes val message: Int, val argument: String? = null) : DiscoveryState
}

/** The message each error of the shared library is shown as. */
fun MobileException.toFailure(): DiscoveryState.Failed = when (this) {
    is MobileException.InvalidAddress -> DiscoveryState.Failed(R.string.sign_in_instance_error_invalid_address)

    is MobileException.Network -> DiscoveryState.Failed(R.string.sign_in_instance_error_network)

    is MobileException.NotFound -> DiscoveryState.Failed(R.string.sign_in_instance_error_not_found)

    is MobileException.Refused ->
        DiscoveryState.Failed(R.string.sign_in_instance_error_refused, status.toString())

    is MobileException.VersionTooOld -> DiscoveryState.Failed(R.string.sign_in_instance_error_version_too_old)

    is MobileException.Internal ->
        DiscoveryState.Failed(R.string.sign_in_instance_error_internal, correlationId)
}
