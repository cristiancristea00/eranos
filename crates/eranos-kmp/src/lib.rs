//! The bindings of eranos for Kotlin Multiplatform, through `uniffi` and
//! gobley.
//!
//! The crate compiles the sources of `eranos-uniffi` against `uniffi` 0.29.4,
//! which the generator of gobley 0.3.7 reads, so the mirrors, objects, and
//! value helpers are those of the other bindings.
#![forbid(unsafe_code)]

uniffi::setup_scaffolding!("eranos");

#[path = "../../eranos-uniffi/src/mirrors.rs"]
mod mirrors;
#[path = "../../eranos-uniffi/src/objects.rs"]
mod objects;
#[path = "../../eranos-uniffi/src/values.rs"]
mod values;
