# Features

One directory per screen group lands here: `SignIn`, `Chat`, `Runs`,
`Approvals`, `Inbox`, `Documents`, `Preferences`. There are none yet. The
Android project's `feature/` directory is its counterpart, one module per
group under the same names.

A feature shapes and draws its screens and forwards actions to the state
machine the shared Rust library provides. Features do not import each other,
and each is opt-in. `../../shared/screens.yaml` lists the screens and states
each one draws.

The directory is a file-system-synchronised group of the app target: a Swift
file added under it is compiled with no edit to `project.pbxproj`. This file
is excluded from the target.
