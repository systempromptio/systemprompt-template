package io.systemprompt.mobile.feature.signin

import io.systemprompt.mobile.R
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.systemprompt_mobile.AppRelease
import uniffi.systemprompt_mobile.InstanceDiscovery
import uniffi.systemprompt_mobile.MobileException

/** The view model over a stand-in for the shared library: what it shows for each answer the library can give. */
@OptIn(ExperimentalCoroutinesApi::class)
class InstanceViewModelTest {
    private val versions = LibraryVersions(library = "1.2.3", core = "4.5.6")
    private val found = InstanceDiscovery(
        address = "https://instance.example",
        displayName = "Example Instance",
        signInHint = "Use your work account",
        ios = null,
        android = AppRelease(
            appId = "io.systemprompt.mobile",
            minVersion = "0.70.0",
            location = "https://instance.example/v1/bridge/download/android"
        )
    )

    @Before
    fun runCoroutinesInline() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @After
    fun restoreTheMainDispatcher() {
        Dispatchers.resetMain()
    }

    @Test
    fun theLibraryVersionsAreShownBeforeAnythingIsAsked() {
        val model = InstanceViewModel(FakeDirectory(versions) { found })

        assertEquals(InstanceViewState(versions = versions, discovery = DiscoveryState.Empty), model.state)
    }

    @Test
    fun aDiscoveredInstanceIsShownWithItsApps() {
        val directory = FakeDirectory(versions) { found }
        val model = InstanceViewModel(directory)

        model.discover("https://instance.example")

        val shown = model.state.discovery as DiscoveryState.Found
        assertEquals(found, shown.instance)
        assertEquals(found.android, shown.android)
        assertEquals(null, shown.ios)
        assertEquals(listOf("https://instance.example"), directory.asked)
    }

    @Test
    fun everyErrorOfTheLibraryHasItsOwnMessage() {
        val expected = listOf(
            MobileException.InvalidAddress() to
                DiscoveryState.Failed(R.string.sign_in_instance_error_invalid_address),
            MobileException.Network() to DiscoveryState.Failed(R.string.sign_in_instance_error_network),
            MobileException.NotFound() to DiscoveryState.Failed(R.string.sign_in_instance_error_not_found),
            MobileException.Refused(503.toUShort()) to
                DiscoveryState.Failed(R.string.sign_in_instance_error_refused, "503"),
            MobileException.VersionTooOld() to
                DiscoveryState.Failed(R.string.sign_in_instance_error_version_too_old),
            MobileException.Internal("ref-1") to
                DiscoveryState.Failed(R.string.sign_in_instance_error_internal, "ref-1")
        )
        for ((error, failure) in expected) {
            val model = InstanceViewModel(FakeDirectory(versions) { throw error })

            model.discover("https://instance.example")

            assertEquals(failure, model.state.discovery)
        }
    }

    @Test
    fun aSecondRequestWaitsForTheFirst() {
        val answer = CompletableDeferred<InstanceDiscovery>()
        val directory = FakeDirectory(versions) { answer.await() }
        val model = InstanceViewModel(directory)

        model.discover("https://instance.example")
        model.discover("https://other.example")

        assertEquals(DiscoveryState.Discovering, model.state.discovery)
        assertEquals(listOf("https://instance.example"), directory.asked)

        answer.complete(found)

        assertEquals(DiscoveryState.Found(found), model.state.discovery)
    }
}

private class FakeDirectory(
    private val versions: LibraryVersions,
    private val answer: suspend (String) -> InstanceDiscovery
) : InstanceDirectory {
    val asked = mutableListOf<String>()

    override fun versions(): LibraryVersions = versions

    override suspend fun discover(address: String): InstanceDiscovery {
        asked += address
        return answer(address)
    }
}
