import SwiftUI

/// The app's entry point: one window that hosts the SwiftUI tree and nothing else.
@main
struct MobileApp: App {
    var body: some Scene {
        WindowGroup {
            RootScreen()
        }
    }
}
