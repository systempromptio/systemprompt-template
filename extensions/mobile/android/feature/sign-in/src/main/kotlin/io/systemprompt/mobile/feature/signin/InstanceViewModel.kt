package io.systemprompt.mobile.feature.signin

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import kotlinx.coroutines.launch
import uniffi.systemprompt_mobile.MobileException

/**
 * The instance screen's view state. It holds nothing else: the address is
 * parsed, the request made and the active instance remembered by the shared
 * library behind [InstanceDirectory].
 */
class InstanceViewModel(private val directory: InstanceDirectory) : ViewModel() {
    var state by mutableStateOf(InstanceViewState(versions = directory.versions()))
        private set

    fun discover(address: String) {
        if (state.discovery == DiscoveryState.Discovering) return
        state = state.copy(discovery = DiscoveryState.Discovering)
        viewModelScope.launch {
            val outcome = try {
                DiscoveryState.Found(directory.discover(address))
            } catch (error: MobileException) {
                error.toFailure()
            }
            state = state.copy(discovery = outcome)
        }
    }

    companion object {
        fun factory(directory: InstanceDirectory): ViewModelProvider.Factory = viewModelFactory {
            initializer { InstanceViewModel(directory) }
        }
    }
}
