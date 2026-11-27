package io.systemprompt.mobile

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import io.systemprompt.mobile.design.AppTheme
import io.systemprompt.mobile.feature.signin.InstanceScreen
import io.systemprompt.mobile.feature.signin.InstanceViewModel

/** The single activity: it hosts the Compose tree and nothing else. */
class MainActivity : ComponentActivity() {
    private val instance: InstanceViewModel by viewModels {
        InstanceViewModel.factory((application as MobileApplication).instances)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            AppTheme {
                InstanceScreen(state = instance.state, onDiscover = instance::discover)
            }
        }
    }
}
