//! The bindings of eranos for the web, through `uniffi` and
//! uniffi-bindgen-react-native's wasm2 flavour.
//!
//! The crate compiles the sources of `eranos-uniffi` against `uniffi`
//! 0.31.2, which the TypeScript generator of uniffi-bindgen-react-native
//! 0.31.0-5 reads, so the mirrors, objects, and value helpers are those of
//! the other bindings. On wasm32 it links `uniffi-runtime-wasm`, which gives
//! the generated code's player the allocator and the panic hook it calls.
#![forbid(unsafe_code)]

uniffi::setup_scaffolding!("eranos");

// Linked for the exports it defines, which the player calls by name.
#[cfg(target_family = "wasm")]
extern crate uniffi_runtime_wasm as _;

#[path = "../../eranos-uniffi/src/mirrors.rs"]
mod mirrors;
#[path = "../../eranos-uniffi/src/objects.rs"]
mod objects;
#[path = "../../eranos-uniffi/src/values.rs"]
mod values;
