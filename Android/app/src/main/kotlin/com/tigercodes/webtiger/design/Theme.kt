// What the generated tokens (Tokens.kt) are made of, and the app's Material theme.
package com.tigercodes.webtiger.design

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.graphics.Color

/** A colour with a light and a dark value, as Design/tokens.json writes them. */
@Immutable
data class TokenColor(val light: Color, val dark: Color) {
    /** The value for the theme the system is in. */
    val current: Color
        @Composable @ReadOnlyComposable
        get() = if (isSystemInDarkTheme()) dark else light
}

/** Material 3 in the system's light or dark theme, with the tokens' accent as its primary colour. */
@Composable
fun WebTigerTheme(content: @Composable () -> Unit) {
    val accent = Tokens.Palette.accent
    val colours = if (isSystemInDarkTheme()) darkColorScheme(primary = accent.dark) else lightColorScheme(primary = accent.light)
    MaterialTheme(colorScheme = colours, content = content)
}
