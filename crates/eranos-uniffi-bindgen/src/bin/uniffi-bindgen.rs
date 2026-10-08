//! The `uniffi` generator for Swift, Kotlin, and Python, of the release that builds
//! the library, so the generated code reads the library's metadata as
//! written.
#![forbid(unsafe_code)]

fn main() {
    uniffi::uniffi_bindgen_main();
}
