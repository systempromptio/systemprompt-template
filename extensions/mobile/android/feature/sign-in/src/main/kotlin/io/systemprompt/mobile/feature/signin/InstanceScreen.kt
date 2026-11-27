package io.systemprompt.mobile.feature.signin

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import io.systemprompt.mobile.R

/** The ids the tests look for; the iOS app uses the same ones. */
object InstanceScreenTags {
    const val TITLE = "root.title"
    const val ADDRESS = "sign_in.instance.address"
    const val DISCOVER = "sign_in.instance.discover"
    const val DISCOVERING = "sign_in.instance.discovering"
    const val DISPLAY_NAME = "sign_in.instance.display_name"
    const val ERROR = "sign_in.instance.error"
    const val LIBRARY_VERSION = "sign_in.instance.library_version"
    const val CORE_VERSION = "sign_in.instance.core_version"
}

private val screenPadding = 24.dp
private val gap = 16.dp

/** Sign-in, "enter an instance": an address, what the instance says about itself, and the library's versions. */
@Composable
fun InstanceScreen(state: InstanceViewState, onDiscover: (String) -> Unit, modifier: Modifier = Modifier) {
    var address by rememberSaveable { mutableStateOf("") }
    val busy = state.discovery == DiscoveryState.Discovering
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(
            modifier = Modifier
                .safeDrawingPadding()
                .verticalScroll(rememberScrollState())
                .padding(screenPadding),
            verticalArrangement = Arrangement.spacedBy(gap)
        ) {
            Text(
                text = stringResource(R.string.app_name),
                style = MaterialTheme.typography.headlineMedium,
                modifier = Modifier.testTag(InstanceScreenTags.TITLE)
            )
            Text(
                text = stringResource(R.string.sign_in_instance_prompt),
                style = MaterialTheme.typography.bodyLarge
            )
            OutlinedTextField(
                value = address,
                onValueChange = { address = it },
                label = { Text(stringResource(R.string.sign_in_instance_address_label)) },
                placeholder = { Text(stringResource(R.string.sign_in_instance_address_placeholder)) },
                singleLine = true,
                keyboardOptions = KeyboardOptions(
                    capitalization = KeyboardCapitalization.None,
                    autoCorrectEnabled = false,
                    keyboardType = KeyboardType.Uri,
                    imeAction = ImeAction.Go
                ),
                keyboardActions = KeyboardActions(onGo = { onDiscover(address) }),
                modifier = Modifier
                    .fillMaxWidth()
                    .testTag(InstanceScreenTags.ADDRESS)
            )
            Button(
                onClick = { onDiscover(address) },
                enabled = !busy,
                modifier = Modifier.testTag(InstanceScreenTags.DISCOVER)
            ) {
                Text(stringResource(R.string.sign_in_instance_discover))
            }
            DiscoveryResult(state.discovery)
            LibraryVersionLines(state.versions)
        }
    }
}

@Composable
private fun DiscoveryResult(discovery: DiscoveryState) {
    when (discovery) {
        DiscoveryState.Empty -> Unit
        DiscoveryState.Discovering -> Text(
            text = stringResource(R.string.sign_in_instance_discovering),
            modifier = Modifier.testTag(InstanceScreenTags.DISCOVERING)
        )
        is DiscoveryState.Found -> FoundInstance(discovery)
        is DiscoveryState.Failed -> Text(
            text = stringResource(discovery.message, *discovery.arguments.toTypedArray()),
            color = MaterialTheme.colorScheme.error,
            modifier = Modifier.testTag(InstanceScreenTags.ERROR)
        )
    }
}

@Composable
private fun LibraryVersionLines(versions: LibraryVersions) {
    Column {
        Text(
            text = stringResource(R.string.sign_in_instance_library_version_label),
            style = MaterialTheme.typography.labelMedium
        )
        Text(
            text = versions.library,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.testTag(InstanceScreenTags.LIBRARY_VERSION)
        )
        Text(
            text = stringResource(R.string.sign_in_instance_core_version_label),
            style = MaterialTheme.typography.labelMedium
        )
        Text(
            text = versions.core,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.testTag(InstanceScreenTags.CORE_VERSION)
        )
    }
}
