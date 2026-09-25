// The plugins the modules apply, at the versions gradle/libs.versions.toml
// pins, declared once so that every module resolves the same release.
plugins {
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.binary.compatibility.validator) apply false
    alias(libs.plugins.dokka) apply false
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.kover) apply false
}
