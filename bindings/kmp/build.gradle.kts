// The plugins the modules apply, at the versions gradle/libs.versions.toml
// pins, declared once so that every module resolves the same release and
// shares the Kotlin plugin's build services.
plugins {
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.gobley.cargo) apply false
    alias(libs.plugins.gobley.uniffi) apply false
    alias(libs.plugins.kotlin.atomicfu) apply false
    alias(libs.plugins.kotlin.multiplatform) apply false
}
