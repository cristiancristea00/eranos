// The Kotlin UniFFI generates, for the JVM, which `cargo xtask gates` copies
// into src/main/kotlin. Generated, it compiles with warnings allowed. Its
// public types extend JNA's and its asynchronous calls suspend through
// kotlinx.coroutines, so both are part of its API.
plugins {
    alias(libs.plugins.kotlin.jvm)
}

// Compiled by JDK 25 for JVM 17, the oldest the package runs on.
kotlin {
    jvmToolchain(25)
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

java {
    targetCompatibility = JavaVersion.VERSION_17
}

dependencies {
    api(libs.jna)
    api(libs.coroutines)
}
