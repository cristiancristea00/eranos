// The plugins the modules apply, at the versions gradle/libs.versions.toml
// pins, declared once so that every module resolves the same release and
// shares the Kotlin plugin's build services.
plugins {
    alias(libs.plugins.android.multiplatform.library) apply false
    alias(libs.plugins.kotlin.multiplatform) apply false
    alias(libs.plugins.uniffi) apply false
}
