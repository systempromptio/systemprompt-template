import SwiftUI

/// The app's theme: the brand palette for the system's light or dark setting.
struct AppTheme {
    let palette: BrandPalette

    init(colorScheme: ColorScheme) {
        palette = colorScheme == .dark ? .dark : .light
    }

    var background: Color { Color(argb: palette.background) }
    var onBackground: Color { Color(argb: palette.onBackground) }
    var primary: Color { Color(argb: palette.primary) }
    var onPrimary: Color { Color(argb: palette.onPrimary) }
}

extension Color {
    /// A colour from a packed ARGB value, eight bits per channel.
    init(argb: UInt32) {
        let channel: (UInt32) -> Double = { shift in
            Double((argb >> shift) & 0xFF) / 255
        }
        self.init(.sRGB, red: channel(16), green: channel(8), blue: channel(0), opacity: channel(24))
    }
}
