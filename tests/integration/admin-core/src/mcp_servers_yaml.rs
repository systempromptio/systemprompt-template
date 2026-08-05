//! `repositories::mcp::mcp_servers` — the reader that turns the
//! `services/mcp/*.yaml` tree into `McpServerDetail` rows.
//!
//! This repository touches the filesystem rather than Postgres, so the tests
//! build a throwaway services directory instead of a throwaway database. The
//! reader is deliberately lenient — an unparseable file, a file with no
//! `mcp_servers:` key, and a non-YAML extension are all skipped rather than
//! failing the page — so each of those paths is pinned here.

use std::fs;
use std::path::{Path, PathBuf};

use systemprompt_web_admin::repositories::mcp::mcp_servers::list_mcp_servers;

use crate::fixtures::unique;

/// A services directory that removes itself when the test ends.
struct ServicesDir {
    root: PathBuf,
}

impl ServicesDir {
    fn new() -> Self {
        let root = std::env::temp_dir().join(unique("admin-mcp"));
        fs::create_dir_all(root.join("mcp")).expect("create services/mcp");
        Self { root }
    }

    /// A services directory with no `mcp/` subdirectory at all.
    fn bare() -> Self {
        let root = std::env::temp_dir().join(unique("admin-mcp"));
        fs::create_dir_all(&root).expect("create services root");
        Self { root }
    }

    fn write(&self, file_name: &str, body: &str) {
        fs::write(self.root.join("mcp").join(file_name), body).expect("write mcp yaml");
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for ServicesDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn list_mcp_servers_is_empty_when_the_mcp_directory_is_absent() {
    let dir = ServicesDir::bare();

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert!(servers.is_empty());
}

#[test]
fn list_mcp_servers_is_empty_when_the_directory_holds_no_yaml() {
    let dir = ServicesDir::new();
    dir.write("README.md", "not a config");
    dir.write("notes.txt", "mcp_servers: {}");

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert!(servers.is_empty(), "only .yaml and .yml are read");
}

#[test]
fn list_mcp_servers_skips_a_file_that_is_not_valid_yaml() {
    let dir = ServicesDir::new();
    dir.write("broken.yaml", "mcp_servers: [unclosed\n  - :::");
    dir.write(
        "good.yaml",
        "mcp_servers:\n  salesforce:\n    binary: sf-mcp\n",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].id.as_str(), "salesforce");
}

#[test]
fn list_mcp_servers_skips_a_file_with_no_mcp_servers_key() {
    let dir = ServicesDir::new();
    dir.write("other.yaml", "agents:\n  reviewer:\n    id: reviewer\n");

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert!(servers.is_empty());
}

#[test]
fn list_mcp_servers_defaults_a_binary_backed_server_to_internal_on_port_5000() {
    let dir = ServicesDir::new();
    dir.write(
        "internal.yaml",
        "mcp_servers:\n  systemprompt:\n    binary: systemprompt-mcp\n",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    let server = &servers[0];
    assert_eq!(server.server_type, "internal");
    assert_eq!(server.binary, "systemprompt-mcp");
    assert_eq!(server.port, 5000);
    assert!(server.enabled, "absent `enabled` means enabled");
    assert!(server.removable);
    assert_eq!(server.package_name, "");
    assert_eq!(server.endpoint, "");
}

#[test]
fn list_mcp_servers_defaults_a_server_with_no_binary_to_external() {
    let dir = ServicesDir::new();
    dir.write(
        "external.yaml",
        "mcp_servers:\n  hosted:\n    endpoint: https://example.test/mcp\n",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert_eq!(servers[0].server_type, "external");
    assert_eq!(servers[0].endpoint, "https://example.test/mcp");
}

#[test]
fn list_mcp_servers_reads_every_declared_field() {
    let dir = ServicesDir::new();
    dir.write(
        "full.yaml",
        r"mcp_servers:
  salesforce:
    type: remote
    binary: sf-mcp
    package: '@astound/sf-mcp'
    port: 7100
    endpoint: https://sf.example.test/mcp
    description: Salesforce accessor
    enabled: false
    oauth:
      required: true
      scopes:
        - mcp_api
        - refresh_token
      audience: https://sf.example.test
",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    let server = &servers[0];
    assert_eq!(server.server_type, "remote", "an explicit type wins");
    assert_eq!(server.package_name, "@astound/sf-mcp");
    assert_eq!(server.port, 7100);
    assert_eq!(server.description, "Salesforce accessor");
    assert!(!server.enabled);
    assert!(server.oauth_required);
    assert_eq!(server.oauth_scopes, ["mcp_api", "refresh_token"]);
    assert_eq!(server.oauth_audience, "https://sf.example.test");
}

#[test]
fn list_mcp_servers_falls_back_to_the_default_port_when_the_value_does_not_fit() {
    let dir = ServicesDir::new();
    dir.write(
        "badport.yaml",
        "mcp_servers:\n  wide:\n    binary: b\n    port: 999999\n",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert_eq!(servers[0].port, 5000, "a port past u16 is not truncated");
}

#[test]
fn list_mcp_servers_records_the_source_path_relative_to_the_services_root() {
    let dir = ServicesDir::new();
    dir.write(
        "salesforce.yaml",
        "mcp_servers:\n  salesforce:\n    binary: b\n",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert_eq!(servers[0].source_path, "services/mcp/salesforce.yaml");
}

#[test]
fn list_mcp_servers_sorts_across_files_by_server_id() {
    let dir = ServicesDir::new();
    dir.write("z.yaml", "mcp_servers:\n  alpha:\n    binary: a\n");
    dir.write("a.yml", "mcp_servers:\n  zulu:\n    binary: z\n");
    dir.write("m.yaml", "mcp_servers:\n  mike:\n    binary: m\n");

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    let ids: Vec<&str> = servers.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["alpha", "mike", "zulu"]);
}

#[test]
fn list_mcp_servers_reads_every_entry_in_one_file() {
    let dir = ServicesDir::new();
    dir.write(
        "many.yaml",
        "mcp_servers:\n  one:\n    binary: a\n  two:\n    binary: b\n",
    );

    let servers = list_mcp_servers(dir.path()).expect("list servers");

    assert_eq!(servers.len(), 2);
}
