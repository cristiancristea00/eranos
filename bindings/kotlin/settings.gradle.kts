// The Kotlin package of eranos: the code UniFFI generates and the hand-written
// layer apps depend on, each built for the JVM and for Android, and the
// examples' ports on the layer. The layer depends on the generated code as
// an implementation detail, so the generated classes stay off the compile
// classpath of an app.

pluginManagement {
    repositories {
        gradlePluginPortal()
        google()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "eranos-kotlin"

include(":generated", ":eranos", ":generated-android", ":eranos-android", ":examples")
