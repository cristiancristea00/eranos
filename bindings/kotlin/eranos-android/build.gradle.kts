// The layer for Android: the sources of the JVM module, compiled as they
// are there, over the generated code for Android.
plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "eranos"
    compileSdk = 36
    defaultConfig {
        minSdk = 26
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    sourceSets {
        getByName("main") {
            kotlin.directories.add("../eranos/src/main/kotlin")
        }
    }
}

kotlin {
    explicitApi()
    compilerOptions {
        allWarningsAsErrors.set(true)
    }
}

dependencies {
    implementation(project(":generated-android"))
}
