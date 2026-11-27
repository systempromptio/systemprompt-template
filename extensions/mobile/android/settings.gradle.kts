// The Android app of the mobile flavour. One module today (:app), which also
// compiles the sources under design/ and feature/ until each becomes a module
// of its own (README.md in each says what arrives).
pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        // Why: the Kotlin half of the system certificate verifier the shared
        // Rust library uses is published here and nowhere else. Its version
        // follows the crate in core/Cargo.lock (app/build.gradle.kts).
        maven {
            url = uri("https://github.com/rustls/rustls-platform-verifier/raw/maven-archive/android-release-support/maven/")
            content {
                includeGroup("org.rustls")
            }
        }
    }
}

rootProject.name = "mobile"
include(":app")
