//! The bindings of eranos for Swift, Kotlin, Java, Python, and Kotlin
//! Multiplatform, through `uniffi`.
//!
//! The crate mirrors the binding core's records, enums, and error for
//! `uniffi`, wraps its group handle, read handle, and cancellation in
//! objects, and exports its value helpers; every method and function
//! forwards to the core, converting nothing and deciding nothing. The
//! generators of `eranos-uniffi-bindgen` turn the library's metadata into
//! each language's code: `uniffi-bindgen` for Swift, Kotlin, and Python,
//! and `uniffi-bindgen-java` for Java; the Kotlin Multiplatform package's
//! plugin reads the library itself.
//!
//! The web and KMP crates compile `mirrors.rs`, `objects.rs`, and
//! `values.rs` from here against their own `uniffi` releases, so these keep
//! to what `uniffi` 0.29 accepts: records, enums with data, one error,
//! objects with synchronous and asynchronous methods, and no methods on
//! records. The copies of the core's items and the objects' calls are
//! written from the core's mirrors and table by `eranos-glue-macros`, whose
//! code names `::uniffi`, the release of the crate that compiles it. The
//! documentation of the objects and functions reaches the generated code of
//! those languages, where a Rust link would read as raw markdown, so it
//! names types in plain code spans.
#![forbid(unsafe_code)]

uniffi::setup_scaffolding!("eranos");

mod mirrors;
mod objects;
mod values;
