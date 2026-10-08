// The Kotlin the UniFFI plugin generates from eranos-uniffi for every
// target, beside the library it builds for each: the host's for the JVM, the
// Android ABIs', and a static library for each Apple target. Generated, it
// compiles with warnings allowed.
import ch.ubique.uniffi.plugin.extensions.useRustUpLinker
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.multiplatform.library)
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.uniffi)
}

// The plugin builds eranos-uniffi for each target and generates the bindings
// from the host's library, into the package the layer imports; its bindgen
// reads the crate's metadata through `cargo`, which it finds on `PATH` or
// through `CARGO`.
uniffi {
    generateFromLibrary {
        packageName = "eranos.internal"
    }
}

cargo {
    packageDirectory = layout.projectDirectory.dir("../../../crates/eranos-uniffi")
    ndkVersion = "30.0.16248370"
}

// Compiled for JVM 17, the oldest the package runs on. The Apple targets are
// those the plugin builds for; Android API level 26 is the libraries'
// `minSdk`.
kotlin {
    jvmToolchain(25)
    jvm {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }
    android {
        namespace = "eranos.internal"
        compileSdk = 37
        minSdk = 26
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
        // The rules an app's shrinker needs for JNA, carried to every app
        // that depends on the library.
        optimization {
            consumerKeepRules.apply {
                publish = true
                file("consumer-rules.pro")
            }
        }
    }
    listOf(iosArm64(), iosSimulatorArm64(), iosX64(), macosArm64()).forEach { target ->
        target.compilations.getByName("main") {
            useRustUpLinker()
        }
    }
}
