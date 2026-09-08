//! Every admin template must be registrable by the engine that serves it.
//!
//! The admin SSR router is built by `WebExtension`'s `admin_ssr` module, which
//! registers all templates up front and returns `None` if any single one fails.
//! A `None` there mounts the API routes and nothing else, so one unparsable
//! template does not break one page — it turns every `/admin/*` URL into a 404,
//! including the login page, while the admin API keeps answering 200. That
//! failure reaches the log as one line at startup and is otherwise invisible.
//!
//! This registers the same directory with the same library and names the
//! offenders, so the mistake fails a test instead of the console.

use crate::support::repo_root;

#[test]
fn every_admin_template_registers_with_the_engine() {
    let dir = repo_root().join("storage/files/admin/templates");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("gate cannot read {}: {e}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "hbs"))
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no admin templates found in {}",
        dir.display()
    );

    // Why: a fresh registry rather than the admin engine — this asserts the
    // parse, and the engine additionally needs partials and helpers that a
    // template is entitled to reference before they are registered.
    let mut hbs = handlebars::Handlebars::new();
    let mut broken = Vec::new();
    for path in paths {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if let Err(err) = hbs.register_template_file(&name, &path) {
            let reason = err.to_string();
            let first = reason.lines().next().unwrap_or("unknown parse failure");
            broken.push(format!("{name}: {first}"));
        }
    }

    assert!(
        broken.is_empty(),
        "admin template(s) the engine cannot register — every /admin/* URL 404s while any one of \
         these is on disk:\n{}",
        broken.join("\n")
    );
}
