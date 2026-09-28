//! Throwaway databases whose test never dropped them.
//!
//! `TempDb::cleanup` is an async call at the end of a test, so a test that
//! panics first never reaches it, and a killed or timed-out run cannot run any
//! destructor at all. Every failing test leaked its database; 216 had piled up
//! on the local server.
//!
//! So each throwaway database is named with the PID of the process that owns
//! it, and every new one first drops those whose owner is no longer running.
//! Templates (`<prefix>_tpl_<hash>`) never match the owned-name pattern.

use std::path::Path;

use sqlx::{AssertSqlSafe, PgPool};

const OWNER_MARK: &str = "_p";
const SUFFIX_HEX: usize = 12;
const POSTGRES_NAME_LIMIT: usize = 63;

#[must_use]
pub fn database_name(prefix: &str) -> String {
    let uuid = uuid::Uuid::new_v4().simple().to_string();
    let tail = format!("{OWNER_MARK}{}_{}", std::process::id(), &uuid[..SUFFIX_HEX]);
    let head: String = prefix
        .chars()
        .take(POSTGRES_NAME_LIMIT.saturating_sub(tail.len()))
        .collect();
    format!("{head}{tail}")
}

pub async fn sweep(admin: &PgPool) {
    let pattern = format!("{OWNER_MARK}[0-9]+_[0-9a-f]{{{SUFFIX_HEX}}}$");
    let names =
        match sqlx::query_scalar::<_, String>("SELECT datname FROM pg_database WHERE datname ~ $1")
            .bind(&pattern)
            .fetch_all(admin)
            .await
        {
            Ok(names) => names,
            Err(e) => {
                eprintln!("could not list throwaway databases to sweep: {e}");
                return; // skip-ok: an orphan that cannot be listed is collected next run
            },
        };
    for name in names {
        let Some(owner) = owner_of(&name) else {
            continue;
        };
        if !owner_is_gone(owner) {
            continue;
        }
        if let Err(e) = sqlx::query(AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"
        )))
        .execute(admin)
        .await
        {
            eprintln!("could not drop orphaned throwaway database {name}: {e}");
        }
    }
}

fn owner_of(name: &str) -> Option<u32> {
    let (rest, _suffix) = name.rsplit_once('_')?;
    let (_, pid) = rest.rsplit_once(OWNER_MARK)?;
    pid.parse().ok()
}

// Why: only Linux answers "is this PID running" from the filesystem, and the
// suites run there, locally and in CI. Anywhere else nothing counts as gone,
// so the sweep is a no-op rather than a guess that drops a live run's data.
fn owner_is_gone(pid: u32) -> bool {
    cfg!(target_os = "linux")
        && pid != std::process::id()
        && !Path::new(&format!("/proc/{pid}")).exists()
}
