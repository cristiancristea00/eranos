//! The binding core's records, enums, and error, written for `uniffi` from
//! the mirrors the core's items carry: each mirror imports its item here,
//! its copy repeats the item field for field, and `uniffi` derives the
//! item's conversions from the copy while the core keeps the type.

eranos_bindings::mirrors!(::eranos_glue_macros::uniffi_mirror);
