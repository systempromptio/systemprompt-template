//! The binding generator, built from the same `uniffi` release the library
//! links, so the generated Kotlin and Swift always match its scaffolding.
//! A host tool: `cargo run --features cli --bin uniffi-bindgen`.

fn main() {
    uniffi::uniffi_bindgen_main();
}
