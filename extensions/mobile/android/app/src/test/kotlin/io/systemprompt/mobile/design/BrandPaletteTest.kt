package io.systemprompt.mobile.design

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class BrandPaletteTest {
    @Test
    fun lightAndDarkDiffer() {
        assertNotEquals(BrandPalette.Light, BrandPalette.Dark)
    }

    @Test
    fun everyColourIsOpaque() {
        for (colour in BrandPalette.Light.colours + BrandPalette.Dark.colours) {
            assertEquals(OPAQUE, colour ushr ALPHA_SHIFT)
        }
    }

    private companion object {
        const val OPAQUE = 0xFFL
        const val ALPHA_SHIFT = 24
    }
}
