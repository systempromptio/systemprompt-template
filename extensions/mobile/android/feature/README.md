# feature

One Gradle module per screen group lands here: `sign-in`, `chat`, `runs`,
`approvals`, `inbox`, `documents`, `preferences`. `sign-in` is the first: it
holds the "enter an instance" screen, and is a source directory `:app`
compiles (`app/build.gradle.kts`) until it becomes a module. The iOS
project's `Features/` group is its counterpart, one directory per group under
the same names.

A feature module shapes and draws its screens and forwards actions to the
shared Rust library through the generated bindings; its view models hold
view state and nothing else. Features do not import each
other, and each is opt-in: a build includes only the features its composition
names. `../../shared/screens.yaml` lists the screens and states each one
draws.
