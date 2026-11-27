//! Embeds the Windows icon resource when building for Windows and the icon file
//! is present.

use std::env;
use std::path::PathBuf;

const RESOURCE: &str = "resources/windows/harness.rc";
const ICON: &str = "assets/icons/icon.ico";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={RESOURCE}");
    println!("cargo:rerun-if-changed={ICON}");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifest = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default();
    if !manifest.join(ICON).is_file() {
        println!("cargo:warning={ICON} is missing; building without the window and taskbar icon");
        return;
    }
    if embed_resource::compile(RESOURCE, embed_resource::NONE)
        .manifest_optional()
        .is_err()
    {
        println!("cargo:warning=the Windows icon resource was not embedded");
    }
}
