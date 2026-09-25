// The hand-written layer of the Kotlin package, compiled from its sources for
// every target of the generated code, in explicit API mode and with warnings
// as errors. It depends on the generated code as an implementation detail,
// so no generated type reaches an app. The Kotlin package's gates lint,
// document, and measure the same sources; here their tests run on the JVM
// against gobley's bindings.
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.multiplatform)
}

// Compiled by JDK 21 for JVM 17, the oldest the package runs on, for the
// targets of the generated module.
kotlin {
    jvmToolchain(21)
    explicitApi()
    compilerOptions {
        allWarningsAsErrors.set(true)
    }
    jvm {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
        testRuns["test"].executionTask.configure {
            useJUnitPlatform()
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
    sourceSets {
        commonMain {
            kotlin.srcDir("../../kotlin/eranos/src/main/kotlin")
            dependencies {
                implementation(project(":generated"))
                implementation(libs.coroutines)
            }
        }
        jvmTest {
            kotlin.srcDir("../../kotlin/eranos/src/test/kotlin")
            dependencies {
                implementation(project.dependencies.platform(libs.junit.bom))
                implementation(libs.junit.jupiter)
                implementation(libs.coroutines.test)
                runtimeOnly(libs.junit.launcher)
            }
        }
    }
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
}
