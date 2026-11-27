package io.systemprompt.mobile

import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.systemprompt.mobile.feature.signin.InstanceScreenTags
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Discovery end to end: an address typed into the screen is read by the shared
 * Rust library from a stub instance on the host
 * (extensions/mobile/shared/stub-instance.py), and the screen shows the name
 * the stub gives. The runner arguments stubInstance and stubInstanceName say
 * where the stub listens and what it is called; the defaults are the stub's
 * own, as an emulator reaches them.
 */
@RunWith(AndroidJUnit4::class)
class InstanceDiscoveryTest {
    @get:Rule
    val compose = createAndroidComposeRule<MainActivity>()

    @Test
    fun anAddressTypedInFindsTheStubInstance() {
        val arguments = InstrumentationRegistry.getArguments()
        val address = arguments.getString("stubInstance") ?: DEFAULT_ADDRESS
        val name = arguments.getString("stubInstanceName") ?: DEFAULT_NAME

        compose.onNodeWithTag(InstanceScreenTags.ADDRESS).performTextInput(address)
        compose.onNodeWithTag(InstanceScreenTags.DISCOVER).performClick()
        compose.waitUntil(DISCOVERY_TIMEOUT_MILLIS) {
            compose.onAllNodesWithTag(InstanceScreenTags.DISPLAY_NAME).fetchSemanticsNodes().isNotEmpty() ||
                compose.onAllNodesWithTag(InstanceScreenTags.ERROR).fetchSemanticsNodes().isNotEmpty()
        }

        compose.onNodeWithTag(InstanceScreenTags.DISPLAY_NAME).assertTextEquals(name)
    }

    private companion object {
        const val DEFAULT_ADDRESS = "http://10.0.2.2:8765"
        const val DEFAULT_NAME = "Stub Instance"
        const val DISCOVERY_TIMEOUT_MILLIS = 30_000L
    }
}
