//! Built-in governance policies. Each submodule registers itself with the
//! `policy` registry via `inventory::submit!`. Adding a new policy means
//! creating a new file here and listing it below.

mod rate_limit;
mod scope_check;
mod secret_scan;
mod tool_blocklist;
