// The Kotlin Multiplatform package of eranos: the code gobley generates from
// eranos-kmp, and the hand-written layer of the Kotlin package, compiled from
// its sources for the JVM, Android, and the Apple platforms. The layer
// depends on the generated code as an implementation detail, so the
// generated classes stay off the compile classpath of an app.

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

rootProject.name = "eranos-kmp"

include(":generated", ":eranos")
