import org.jetbrains.kotlin.gradle.dsl.JvmTarget

// The version of the certificate verifier's Kotlin half is the version of its
// Rust half: read from the binding crate's lockfile, so the two cannot differ.
abstract class RustlsVersion : ValueSource<String, RustlsVersion.Params> {
    interface Params : ValueSourceParameters {
        val lockFile: RegularFileProperty
    }

    override fun obtain(): String {
        val lines = parameters.lockFile.get().asFile.readLines()
        val name = lines.indexOfFirst { it.trim() == "name = \"$CRATE_NAME\"" }
        val version = lines.drop(name + 1)
            .takeIf { name >= 0 }
            ?.firstOrNull { it.trimStart().startsWith("version = ") }
            ?.substringAfter('"', "")
            ?.substringBefore('"', "")
            ?.takeIf { it.isNotEmpty() }
        return version ?: error("$CRATE_NAME not found in the binding crate's Cargo.lock")
    }

    companion object {
        const val CRATE_NAME = "rustls-platform-verifier-android"
    }
}

val rustlsPlatformVerifierVersion = providers.of(RustlsVersion::class.java) {
    parameters.lockFile.set(layout.projectDirectory.file("../../core/Cargo.lock"))
}

configurations.configureEach {
    resolutionStrategy.eachDependency {
        if (requested.group == "org.rustls" && requested.name == "rustls-platform-verifier") {
            useVersion(rustlsPlatformVerifierVersion.get())
            because("the Kotlin component must be the version of the ${RustlsVersion.CRATE_NAME} crate")
        }
    }
}

// What shared/build-android.sh makes before Gradle runs: the Kotlin generated
// from the shared Rust library. The libraries themselves land in
// src/main/jniLibs. Neither is tracked.
val generatedBindings = layout.buildDirectory.dir("generated/uniffi/kotlin")

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.detekt)
    alias(libs.plugins.ktlint)
}

android {
    namespace = "io.systemprompt.mobile"
    compileSdk = libs.versions.compileSdk.get().toInt()
    buildToolsVersion = libs.versions.buildTools.get()
    // Why: the Android plugin strips debug information from the shared Rust
    // library with this NDK's tools when it packages the app.
    ndkVersion = libs.versions.ndk.get()

    defaultConfig {
        applicationId = "io.systemprompt.mobile"
        minSdk = libs.versions.minSdk.get().toInt()
        targetSdk = libs.versions.targetSdk.get().toInt()
        versionCode = libs.versions.appVersionCode.get().toInt()
        versionName = libs.versions.appVersionName.get()
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
    }

    // Why: design/ and feature/<group>/ are the directories the design and
    // feature modules will own. Until each is a module, :app compiles their
    // sources, so the code already lives where the iOS project keeps its
    // Design/ and Features/ groups. The generated bindings are compiled the
    // same way.
    sourceSets {
        getByName("main") {
            kotlin.directories.add("../design/src/main/kotlin")
            kotlin.directories.add("../feature/sign-in/src/main/kotlin")
            kotlin.directories.add(generatedBindings.get().asFile.path)
            res.directories.add("../feature/sign-in/src/main/res")
        }
        getByName("test") {
            kotlin.directories.add("../feature/sign-in/src/test/kotlin")
        }
    }

    lint {
        warningsAsErrors = true
        abortOnError = true
        checkDependencies = true
        // Why: the pins move deliberately (shared/toolchain.env and the
        // catalog), not because a newer release or API level exists.
        disable += setOf("GradleDependency", "NewerVersionAvailable", "AndroidGradlePluginVersion", "OldTargetApi")
        // Why: a library's bundled lint checks built against an older lint
        // API are skipped by lint itself; that is the library's to fix.
        disable += "ObsoleteLintCustomCheck"
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
        allWarningsAsErrors = true
    }
}

detekt {
    buildUponDefaultConfig = true
    config.setFrom(rootProject.file("config/detekt/detekt.yml"))
    source.setFrom("src", "../design/src", "../feature")
}

ktlint {
    version.set(libs.versions.ktlint.get())
    android.set(true)
    ignoreFailures.set(false)
    // Why: the generated bindings are build output in another tool's style.
    filter {
        exclude { it.file.path.contains("/build/generated/") }
    }
}

// The app cannot compile without the generated bindings; say where they come
// from instead of failing on the first unresolved name.
val bindingsPresent = tasks.register("bindingsPresent") {
    val bindings = generatedBindings.map { it.asFile }
    val libraries = layout.projectDirectory.dir("src/main/jniLibs").asFile
    doLast {
        check(bindings.get().isDirectory && libraries.isDirectory) {
            "the shared Rust library and its Kotlin bindings are not built: " +
                "run extensions/mobile/shared/build-android.sh (just mobile-android)"
        }
    }
}
tasks.named("preBuild") { dependsOn(bindingsPresent) }

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.lifecycle.viewmodel)

    // What the generated bindings call the shared Rust library with, and the
    // Kotlin half of the certificate verifier that library uses.
    implementation(variantOf(libs.jna) { artifactType("aar") })
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.rustls.platform.verifier)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)

    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.test.ext.junit)
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}
