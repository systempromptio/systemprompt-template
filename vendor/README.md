# Dependency compatibility patch

`proc-macro-error2` 2.0.1 is copied from the crates.io source, retaining its MIT/Apache licenses. The only source change makes `extern crate proc_macro` public, as required by its existing public re-export (Rust E0365, rust-lang/rust#127909). This fixes the compiler warning without suppressing it. Upstream 2.0.1 is the latest published release and is unmaintained. Remove this patch when tabled no longer depends on the affected crate or a fixed release is available. Both workspaces use this same in-repository source; core remains on crates.io.
