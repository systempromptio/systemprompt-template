# design

The theme and the components every screen is drawn from, over colour and
string resources generated from the product's brand and theme declarations.
The iOS project's `Design/` group is its counterpart: the same names, the
same split.

It is a source directory today, compiled by `:app`
(`app/build.gradle.kts`), and holds the placeholder palette and the theme.
It becomes a Gradle module of its own when the first feature module needs it.
Design code never imports a feature.
