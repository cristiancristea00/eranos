// The layer apps depend on, written by hand, for the JVM; eranos-android
// compiles the same sources for Android. It depends on the generated code
// as an implementation detail, so no generated type reaches an app.
plugins {
    alias(libs.plugins.binary.compatibility.validator)
    alias(libs.plugins.dokka)
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.kover)
}

// Compiled by JDK 25 for JVM 17, the oldest the package runs on, in explicit
// API mode and with warnings as errors.
kotlin {
    jvmToolchain(25)
    explicitApi()
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        allWarningsAsErrors.set(true)
    }
}

java {
    targetCompatibility = JavaVersion.VERSION_17
}

val detektCli = configurations.create("detektCli")

dependencies {
    implementation(project(":generated"))
    testImplementation(platform(libs.junit.bom))
    testImplementation(libs.junit.jupiter)
    testImplementation(libs.coroutines.test)
    testRuntimeOnly(libs.junit.launcher)
    detektCli(libs.detekt.cli)
}

// Every public declaration is documented, and the documentation builds
// without a warning.
dokka {
    dokkaPublications.html {
        failOnWarning.set(true)
    }
    dokkaSourceSets.configureEach {
        reportUndocumented.set(true)
    }
}

// Every line of the layer runs in its tests (docs/TESTING.md § 5).
kover {
    reports {
        verify {
            rule {
                minBound(100)
            }
        }
    }
}

// The tests load the host library that `cargo build --package eranos-uniffi`
// writes: from the workspace's `target/debug`, or from the directory the
// property `eranos.library` names, which the gates set to `debug` under
// cargo's target directory. JNA loads it through JNI, which the JDK allows
// only with native access granted.
tasks.test {
    useJUnitPlatform()
    val library = providers.gradleProperty("eranos.library").orElse(rootDir.resolve("../../target/debug").path)
    systemProperty("jna.library.path", library.get())
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

// Detekt runs from its command-line tool, because its 1.23 Gradle plugin
// calls an API that Gradle 9 deprecates, which `--warning-mode=fail` turns
// into a failure. It runs without type resolution: its compiler, Kotlin 2.0,
// cannot read the metadata of the classes Kotlin 2.4 compiles, and reports
// calls into them as what they are not.
val detekt =
    tasks.register<JavaExec>("detekt") {
        description = "Runs every detekt rule on the layer and its tests."
        group = "verification"
        val sources = files("src/main/kotlin", "src/test/kotlin")
        val configuration = file("detekt.yml")
        val report = layout.buildDirectory.file("reports/detekt/detekt.txt")
        inputs.files(sources, configuration)
        outputs.file(report)
        classpath = detektCli
        mainClass.set("io.gitlab.arturbosch.detekt.cli.Main")
        argumentProviders.add {
            listOf(
                "--input",
                sources.joinToString(",") { it.path },
                "--config",
                configuration.path,
                "--build-upon-default-config",
                "--all-rules",
                "--report",
                "txt:${report.get().asFile.path}",
            )
        }
    }

tasks.check {
    dependsOn(detekt)
}
