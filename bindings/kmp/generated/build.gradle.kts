// The Kotlin gobley generates from eranos-kmp for every target, beside the
// library its Cargo plugin builds for each: the host's for the JVM, the four
// ABIs' for Android, and a static library for each Apple target. Generated,
// it compiles with warnings allowed.
import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.jvm
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.gobley.cargo)
    alias(libs.plugins.gobley.uniffi)
    alias(libs.plugins.kotlin.atomicfu)
    alias(libs.plugins.kotlin.multiplatform)
}

// Compiled by JDK 21 for JVM 17, the oldest the package runs on. The Apple
// targets are those that Kotlin/Native, gobley, and rustup's standard
// libraries all have, but for macOS on x86_64, which Kotlin deprecates; the
// layer declares the same.
kotlin {
    jvmToolchain(21)
    jvm {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }
    androidTarget {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }
    iosArm64()
    iosSimulatorArm64()
    iosX64()
    macosArm64()
    tvosArm64()
    tvosSimulatorArm64()
    watchosDeviceArm64()
    watchosSimulatorArm64()
}

// Android API level 26, the libraries' `minSdk`, with the NDK the Kotlin
// package builds with, and the rules an app's shrinker needs for JNA.
android {
    namespace = "eranos.internal"
    compileSdk = 36
    ndkVersion = "30.0.16248370"
    defaultConfig {
        minSdk = 26
        consumerProguardFiles("consumer-rules.pro")
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

// The JVM target carries the host's library alone, in a JAR of its own:
// gobley builds every desktop's otherwise, with toolchains the host lacks.
cargo {
    packageDirectory = layout.projectDirectory.dir("../../../crates/eranos-kmp")
    builds.jvm {
        embedRustLibrary = (rustTarget == GobleyHost.current.rustTarget)
    }
}

// The library's namespace names the generated header, which gobley looks for
// under the crate's name unless told. gobley's own shrinker rules never
// reach the Android library's consumers, so consumer-rules.pro holds them.
uniffi {
    generateProguardRules = false
    generateFromLibrary {
        namespace = "eranos"
        packageName = "eranos.internal"
    }
}
