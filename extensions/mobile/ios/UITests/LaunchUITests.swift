import XCTest

final class LaunchUITests: XCTestCase {
    @MainActor
    func testLaunchShowsTheAppName() {
        let app = XCUIApplication()
        app.launch()
        XCTAssertTrue(app.staticTexts["root.title"].waitForExistence(timeout: 10))
    }
}
