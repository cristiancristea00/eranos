//! The `uniffi` generator for Swift and Kotlin, of the release that builds
//! the library, so the generated code reads the library's metadata as
//! written.

fn main() {
    uniffi::uniffi_bindgen_main();
}
