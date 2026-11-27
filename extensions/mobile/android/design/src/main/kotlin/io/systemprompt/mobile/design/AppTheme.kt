package io.systemprompt.mobile.design

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/** The app's theme: the brand palette for the system's light or dark setting. */
@Composable
fun AppTheme(darkTheme: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    val colorScheme = if (darkTheme) {
        BrandPalette.Dark.over(darkColorScheme())
    } else {
        BrandPalette.Light.over(lightColorScheme())
    }
    MaterialTheme(colorScheme = colorScheme, content = content)
}

private fun BrandPalette.over(base: ColorScheme): ColorScheme = base.copy(
    background = Color(background),
    onBackground = Color(onBackground),
    surface = Color(background),
    onSurface = Color(onBackground),
    primary = Color(primary),
    onPrimary = Color(onPrimary)
)
