package io.systemprompt.mobile.feature.signin

import androidx.annotation.StringRes
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import io.systemprompt.mobile.R
import uniffi.systemprompt_mobile.AppRelease

private val lineGap = 8.dp

/** What a discovered instance says about itself, and where it offers each platform's app. */
@Composable
internal fun FoundInstance(found: DiscoveryState.Found) {
    Column(verticalArrangement = Arrangement.spacedBy(lineGap)) {
        Text(
            text = found.instance.displayName ?: stringResource(R.string.sign_in_instance_unnamed),
            style = MaterialTheme.typography.titleLarge,
            modifier = Modifier.testTag(InstanceScreenTags.DISPLAY_NAME)
        )
        Text(text = found.instance.address, style = MaterialTheme.typography.bodyMedium)
        found.instance.signInHint?.let { hint ->
            Text(text = hint, style = MaterialTheme.typography.bodyLarge)
        }
        OfferedApp(
            release = found.android,
            offered = R.string.sign_in_instance_android_app,
            notOffered = R.string.sign_in_instance_android_app_none
        )
        OfferedApp(
            release = found.ios,
            offered = R.string.sign_in_instance_ios_app,
            notOffered = R.string.sign_in_instance_ios_app_none
        )
    }
}

@Composable
private fun OfferedApp(release: AppRelease?, @StringRes offered: Int, @StringRes notOffered: Int) {
    if (release == null) {
        Text(text = stringResource(notOffered), style = MaterialTheme.typography.bodyMedium)
        return
    }
    Column {
        Text(text = stringResource(offered, release.location), style = MaterialTheme.typography.bodyMedium)
        release.minVersion?.let { version ->
            Text(
                text = stringResource(R.string.sign_in_instance_app_min_version, version),
                style = MaterialTheme.typography.bodySmall
            )
        }
    }
}
