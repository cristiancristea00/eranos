import net.ltgt.gradle.errorprone.errorprone

plugins {
    `java-library`
    jacoco
    alias(libs.plugins.errorprone)
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(25)
    }
}

// The code UniFFI generates, which `cargo xtask gates` copies into
// src/generated/java. It compiles apart, with warnings allowed, and the
// layer compiles with its classes patched into the module `eranos`, whose
// module-info exports only the layer's package.
val generated = sourceSets.create("generated")

sourceSets.main {
    compileClasspath += generated.output
    runtimeClasspath += generated.output
}

sourceSets.test {
    compileClasspath += generated.output
    runtimeClasspath += generated.output
}

// The examples in Java: each package ports the Rust example of its name and
// prints the same report, which `cargo xtask gates` compares byte for byte,
// run on the host library whose identifiers count up.
val examples =
    sourceSets.create("examples") {
        compileClasspath += sourceSets.main.get().output + generated.output
        runtimeClasspath += sourceSets.main.get().output + generated.output
    }

val googleJavaFormat = configurations.create("googleJavaFormat")

dependencies {
    errorprone(libs.errorprone.core)
    googleJavaFormat(libs.google.java.format)
    testImplementation(platform(libs.junit.bom))
    testImplementation(libs.junit.jupiter)
    testRuntimeOnly(libs.junit.launcher)
}

tasks.named<JavaCompile>("compileGeneratedJava") {
    options.release = 25
    options.errorprone.enabled.set(false)
}

// The layer compiles for JDK 25 with every lint as an error and Error Prone,
// which runs inside a forked javac that the plugin gives the exports of the
// compiler's internals it reads. The generated classes are patched into the
// module `eranos`, so `-Xlint:exports` refuses a public signature that names
// one of their types.
tasks.compileJava {
    options.release = 25
    options.isFork = true
    options.compilerArgs.addAll(
        listOf(
            "-Xlint:all",
            "-Werror",
            "--patch-module",
            "eranos=${generated.output.classesDirs.asPath}",
        ),
    )
}

tasks.compileTestJava {
    options.release = 25
    options.isFork = true
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

tasks.named<JavaCompile>("compileExamplesJava") {
    options.release = 25
    options.isFork = true
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

// One JAR holds the module: the layer and the generated classes.
tasks.jar {
    from(generated.output)
}

val examplesJar =
    tasks.register<Jar>("examplesJar") {
        description = "Assembles a JAR of the ports of the examples."
        group = "build"
        archiveClassifier = "examples"
        from(examples.output)
    }

// The ports and the module they run on, in one directory, from which the
// gates start each port with `java`, the host library on its library path.
tasks.register<Sync>("ports") {
    description = "Gathers the ports and the module they run on."
    group = "build"
    from(tasks.jar, examplesJar)
    into(layout.buildDirectory.dir("ports"))
}

// The tests load the host library that `cargo build --package eranos-uniffi`
// writes: from the workspace's `target/debug`, or from the directory the
// property `eranos.library` names, which the gates set to `debug` under
// cargo's target directory. Loading it is a restricted method of the JDK:
// the tests grant it, and deny it to any code the grant does not name, so a
// missing grant fails the tests instead of printing a warning.
tasks.test {
    useJUnitPlatform()
    val library = providers.gradleProperty("eranos.library").orElse(rootDir.resolve("../../target/debug").path)
    systemProperty("java.library.path", library.get())
    jvmArgs("--enable-native-access=ALL-UNNAMED", "--illegal-native-access=deny")
}

jacoco {
    toolVersion = libs.versions.jacoco.get()
}

// The XML report is what a script reads for the lines a test missed.
tasks.jacocoTestReport {
    reports {
        xml.required = true
    }
}

// Every instruction, line, and method of the layer runs in its tests
// (docs/TESTING.md § 5).
tasks.jacocoTestCoverageVerification {
    violationRules {
        rule {
            for (counter in listOf("INSTRUCTION", "LINE", "METHOD")) {
                limit {
                    this.counter = counter
                    minimum = "1.0".toBigDecimal()
                }
            }
        }
    }
}

// The documentation of the module documents its exported package alone, and
// fails on anything doclint reports.
tasks.javadoc {
    val options = options as StandardJavadocDocletOptions
    options.addStringOption("-patch-module", "eranos=${generated.output.classesDirs.asPath}")
    options.addBooleanOption("Xdoclint:all", true)
    options.addBooleanOption("Werror", true)
}

// google-java-format's AOSP style: four spaces, and its fixed 100 columns.
// It reads the sources with javac's parser, which it reaches through these
// exports on every JDK since 16.
val googleJavaFormatExports =
    listOf("api", "code", "file", "parser", "tree", "util").map {
        "--add-exports=jdk.compiler/com.sun.tools.javac.$it=ALL-UNNAMED"
    }
val javaSources = fileTree("src") { include("main/java/**/*.java", "test/java/**/*.java", "examples/java/**/*.java") }

tasks.register<JavaExec>("formatCheck") {
    description = "Checks that google-java-format leaves the layer, its tests, and the examples as they are."
    group = "verification"
    inputs.files(javaSources)
    classpath = googleJavaFormat
    mainClass.set("com.google.googlejavaformat.java.Main")
    jvmArgs(googleJavaFormatExports)
    argumentProviders.add { listOf("--aosp", "--dry-run", "--set-exit-if-changed") + javaSources.map { it.path } }
}

tasks.register<JavaExec>("format") {
    description = "Formats the layer, its tests, and the examples with google-java-format."
    group = "formatting"
    classpath = googleJavaFormat
    mainClass.set("com.google.googlejavaformat.java.Main")
    jvmArgs(googleJavaFormatExports)
    argumentProviders.add { listOf("--aosp", "--replace") + javaSources.map { it.path } }
}
