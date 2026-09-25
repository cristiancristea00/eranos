//! The Java generator, built on the `uniffi` release that builds the
//! library.

use std::process::ExitCode;

fn main() -> ExitCode {
    match uniffi_bindgen_java::run_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:?}");
            ExitCode::FAILURE
        }
    }
}
