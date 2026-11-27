# feature

One Gradle module per screen group lands here: `sign-in`, `chat`, `runs`,
`approvals`, `inbox`, `documents`, `preferences`. There are none yet. The
iOS project's `Features/` group is its counterpart, one directory per group
under the same names.

A feature module shapes and draws its screens and forwards actions to the
state machine the shared Rust library provides. Features do not import each
other, and each is opt-in: a build includes only the features its composition
names. `../../shared/screens.yaml` lists the screens and states each one
draws.
