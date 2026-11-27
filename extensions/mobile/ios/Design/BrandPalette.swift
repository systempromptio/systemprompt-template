/// The colours a theme is drawn from, as opaque ARGB values. Placeholder data:
/// a build step will generate these from the product's brand and theme
/// declarations, the same ones the Android app is generated from.
struct BrandPalette: Equatable, Sendable {
    let background: UInt32
    let onBackground: UInt32
    let primary: UInt32
    let onPrimary: UInt32

    /// Every colour of the palette, for checks that hold for all of them.
    var colours: [UInt32] {
        [background, onBackground, primary, onPrimary]
    }

    static let light = BrandPalette(
        background: 0xFFFF_FFFF,
        onBackground: 0xFF11_1418,
        primary: 0xFF1F_4FD8,
        onPrimary: 0xFFFF_FFFF
    )

    static let dark = BrandPalette(
        background: 0xFF11_1418,
        onBackground: 0xFFF4_F6F8,
        primary: 0xFF9D_B7FF,
        onPrimary: 0xFF11_1418
    )
}
