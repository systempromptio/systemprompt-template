package io.systemprompt.mobile

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LaunchTest {
    @get:Rule
    val compose = createAndroidComposeRule<MainActivity>()

    @Test
    fun launchShowsTheAppName() {
        val appName = compose.activity.getString(R.string.app_name)
        compose.onNodeWithTag(ROOT_TITLE_TAG).assertIsDisplayed().assertTextEquals(appName)
    }
}
