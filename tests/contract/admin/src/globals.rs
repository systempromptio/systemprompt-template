//! The fixture services tree the router is built over.
//!
//! The fixture ships its own services tree rather than the repository's: the
//! catalog and ACL contracts assert against a controlled two-route gateway,
//! and reading the shipped catalog would turn them into assertions about
//! whatever this deployment happens to expose. Each test binary works on its
//! own copy, because the gateway editor routes write the gateway file back.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use systemprompt::manifest::services::ServicesConfig;

pub use template_test_common::paths::repo_root;

const GATEWAY: &str = include_str!("../fixtures/services/ai/gateway.yaml");

pub fn init() -> bool {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::ERROR)
        .with_test_writer()
        .try_init();
    true
}

pub fn services() -> ServicesConfig {
    template_test_common::env::services_config(GATEWAY)
}

pub fn services_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let dir = repo_root().join(format!(
            "tests/target/contract-services-{}",
            std::process::id()
        ));
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/services"),
            &dir,
        );
        dir
    })
}

fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create fixture services directory");
    for entry in std::fs::read_dir(source).expect("read fixture services directory") {
        let entry = entry.expect("fixture services entry");
        let target = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture services file type")
            .is_dir()
        {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy fixture services file");
        }
    }
}
