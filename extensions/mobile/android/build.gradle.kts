// Plugins are declared once here and applied per module. Every version comes
// from gradle/libs.versions.toml, which is also the dependency allowlist.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.detekt) apply false
    alias(libs.plugins.ktlint) apply false
}
