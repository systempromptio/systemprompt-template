package io.systemprompt.mobile.design

/**
 * The colours a theme is drawn from, as opaque ARGB values. Placeholder data:
 * a build step will generate these from the product's brand and theme
 * declarations, the same ones the iOS app is generated from.
 */
data class BrandPalette(val background: Long, val onBackground: Long, val primary: Long, val onPrimary: Long) {
    /** Every colour of the palette, for checks that hold for all of them. */
    val colours: List<Long>
        get() = listOf(background, onBackground, primary, onPrimary)

    companion object {
        val Light = BrandPalette(
            background = 0xFFFFFFFF,
            onBackground = 0xFF111418,
            primary = 0xFF1F4FD8,
            onPrimary = 0xFFFFFFFF
        )
        val Dark = BrandPalette(
            background = 0xFF111418,
            onBackground = 0xFFF4F6F8,
            primary = 0xFF9DB7FF,
            onPrimary = 0xFF111418
        )
    }
}
