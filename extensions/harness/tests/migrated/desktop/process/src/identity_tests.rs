//! Each build channel's identity, and the installer inputs that keep the
//! channels apart on one machine.

use std::collections::HashSet;
use std::path::PathBuf;

use systemprompt_harness_brand::{Channel, Identity};

use crate::TestResult;

fn builds() -> Vec<&'static Identity> {
    Identity::ALL
        .iter()
        .map(|(brand, channel)| Identity::of(*brand, *channel))
        .collect()
}

fn repository_file(relative: &str) -> TestResult<String> {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let found = start
        .ancestors()
        .map(|dir| dir.join(relative))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("{relative} not found above the crate"))?;
    Ok(std::fs::read_to_string(found)?)
}

#[test]
fn each_channel_has_its_own_name_and_app_id() {
    let builds = builds();
    let names: HashSet<_> = builds.iter().map(|build| build.display_name()).collect();
    let ids: HashSet<_> = builds.iter().map(|build| build.app_id()).collect();
    assert_eq!(names.len(), builds.len());
    assert_eq!(ids.len(), builds.len());
    for build in builds {
        let suffix = match build.channel() {
            Channel::Internal => None,
            Channel::Canary => Some(("Canary", ".canary")),
            Channel::Dev => Some(("Dev", ".dev")),
        };
        match suffix {
            None => assert_eq!(build.channel_label(), None),
            Some((label, id)) => {
                assert!(build.display_name().ends_with(label));
                assert!(build.app_id().ends_with(id));
            },
        }
    }
}

#[test]
fn each_channel_installs_into_its_own_folder_and_registry_keys() -> TestResult {
    let script = repository_file("packaging/harness/windows/harness.nsi")?;
    let packer = repository_file("packaging/harness/windows/package.ps1")?;
    assert!(script.contains(r#"InstallDir "$LOCALAPPDATA\Programs\${INSTALL_NAME}""#));
    assert!(script.contains(r"CurrentVersion\Uninstall\${APP_ID}"));
    assert!(script.contains(r"AppUserModelId\${APP_ID}"));
    assert!(packer.contains("/DINSTALL_NAME=$product"));
    let folders: HashSet<_> = builds().iter().map(|build| build.display_name()).collect();
    assert_eq!(folders.len(), Identity::ALL.len());
    Ok(())
}
