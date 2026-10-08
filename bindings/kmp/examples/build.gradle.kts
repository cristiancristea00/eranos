// The Kotlin package's ports of the examples, compiled from its sources for
// the JVM against the package's bindings: each prints the same report as its
// Rust example, which `cargo xtask gates` compares byte for byte, run on the
// library of eranos-uniffi whose identifiers count up.
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlin.multiplatform)
}

// Compiled by JDK 25 for JVM 17, as the layer is, with warnings as errors.
kotlin {
    jvmToolchain(25)
    compilerOptions {
        allWarningsAsErrors.set(true)
    }
    jvm {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }
    sourceSets {
        jvmMain {
            kotlin.srcDir("../../kotlin/examples/src/main/kotlin")
            dependencies {
                implementation(project(":eranos"))
                implementation(libs.coroutines)
            }
        }
    }
}

// The ports and every library they run with, in one directory, from which
// the gates start each port with `java`, the library on JNA's path.
tasks.register<Sync>("ports") {
    description = "Gathers the ports and the libraries they run with."
    group = "build"
    from(tasks.named("jvmJar"))
    from(configurations.named("jvmRuntimeClasspath"))
    into(layout.buildDirectory.dir("ports"))
}
