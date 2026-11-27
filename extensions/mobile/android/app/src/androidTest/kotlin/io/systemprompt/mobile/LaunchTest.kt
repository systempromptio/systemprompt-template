package io.systemprompt.mobile

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.test.ext.junit.runners.AndroidJUnit4
import io.systemprompt.mobile.feature.signin.InstanceScreenTags
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.systemprompt_mobile.diagnostics

@RunWith(AndroidJUnit4::class)
class LaunchTest {
    @get:Rule
    val compose = createAndroidComposeRule<MainActivity>()

    @Test
    fun launchShowsTheAppName() {
        val appName = compose.activity.getString(R.string.app_name)
        compose.onNodeWithTag(InstanceScreenTags.TITLE).assertIsDisplayed().assertTextEquals(appName)
    }

    /** The first call into the shared Rust library: the screen shows the core version the library reports. */
    @Test
    fun launchShowsTheCoreVersionOfTheSharedLibrary() {
        val coreVersion = diagnostics().coreVersion
        check(coreVersion.isNotBlank()) { "the shared library reports no core version" }
        compose.onNodeWithTag(InstanceScreenTags.CORE_VERSION).assertIsDisplayed().assertTextEquals(coreVersion)
    }
}
