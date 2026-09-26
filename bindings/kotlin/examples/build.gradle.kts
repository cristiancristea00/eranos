// The examples of eranos in Kotlin: each package ports the Rust example of
// its name and prints the same report, which `cargo xtask gates` compares
// byte for byte, run on the host library whose identifiers count up.
plugins {
    alias(libs.plugins.kotlin.jvm)
}

// Compiled by JDK 25 for JVM 17, as the layer is, with warnings as errors.
kotlin {
    jvmToolchain(25)
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
    implementation(project(":eranos"))
    implementation(libs.coroutines)
    detektCli(libs.detekt.cli)
}

// The ports and every library they run with, in one directory, from which
// the gates start each port with `java`, the host library on JNA's path.
tasks.register<Sync>("ports") {
    description = "Gathers the ports and the libraries they run with."
    group = "build"
    from(tasks.jar)
    from(configurations.runtimeClasspath)
    into(layout.buildDirectory.dir("ports"))
}

// Detekt runs from its command-line tool, as it does for the layer, with the
// layer's configuration and the examples' own.
val detekt =
    tasks.register<JavaExec>("detekt") {
        description = "Runs every detekt rule on the examples."
        group = "verification"
        val sources = files("src/main/kotlin")
        val configurations = files("../eranos/detekt.yml", "detekt.yml")
        val report = layout.buildDirectory.file("reports/detekt/detekt.txt")
        inputs.files(sources, configurations)
        outputs.file(report)
        classpath = detektCli
        mainClass.set("io.gitlab.arturbosch.detekt.cli.Main")
        argumentProviders.add {
            listOf(
                "--input",
                sources.joinToString(",") { it.path },
                "--config",
                configurations.joinToString(",") { it.path },
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
