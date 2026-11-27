import SwiftUI

/// The id the launch tests look for; the Android app uses the same one.
let rootTitleTag = "root.title"

/// The blank screen the scaffold shows: the app name on the themed background.
struct RootScreen: View {
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        let theme = AppTheme(colorScheme: colorScheme)
        ZStack {
            theme.background.ignoresSafeArea()
            Text("app_name")
                .font(.title)
                .foregroundStyle(theme.onBackground)
                .accessibilityIdentifier(rootTitleTag)
        }
    }
}
