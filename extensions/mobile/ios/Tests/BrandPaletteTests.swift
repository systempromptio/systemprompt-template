import XCTest

@testable import Mobile

final class BrandPaletteTests: XCTestCase {
    func testLightAndDarkDiffer() {
        XCTAssertNotEqual(BrandPalette.light, BrandPalette.dark)
    }

    func testEveryColourIsOpaque() {
        for colour in BrandPalette.light.colours + BrandPalette.dark.colours {
            XCTAssertEqual(colour >> 24, 0xFF)
        }
    }
}
