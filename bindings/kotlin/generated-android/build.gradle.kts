// The generated Kotlin for Android, beside the library of every Android ABI,
// which `cargo xtask gates` builds into src/main/jniLibs. JNA comes as its
// Android archive, which holds its native library for every ABI, and
// consumer-rules.pro, which the library carries to every app, keeps what JNA
// reads by name when the app shrinks its code.
plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "eranos.internal"
    compileSdk = 36
    // The NDK docs/DEVELOPMENT.md § 1 installs, which strips the libraries.
    ndkVersion = "30.0.16248370"
    defaultConfig {
        minSdk = 26
        consumerProguardFiles("consumer-rules.pro")
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    sourceSets {
        getByName("main") {
            kotlin.directories.add("../generated/src/main/kotlin")
        }
    }
}

dependencies {
    api("net.java.dev.jna:jna:${libs.versions.jna.get()}@aar")
    api(libs.coroutines)
}
