# Design

The theme and the components every screen is drawn from, over an asset
catalogue and colour set generated from the product's brand and theme
declarations. The Android project's `design/` directory is its counterpart:
the same names, the same split.

It holds the placeholder palette and the theme today. Design code never
imports a feature. This file is excluded from the app target in
`Mobile.xcodeproj`.
