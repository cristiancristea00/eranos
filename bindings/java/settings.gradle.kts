// The Java package of eranos: one JPMS module, `eranos`, holding the
// hand-written layer apps use and, in `eranos.internal`, which it does not
// export, the code UniFFI generates.

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
}

rootProject.name = "eranos"
