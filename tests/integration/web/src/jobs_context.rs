//! The jobs whose entry point is `Job::execute(&JobContext)`.
//!
//! `jobs_db` covers the one job with a pool-only entry point. Everything else
//! reads its pool and its `AppPaths` back out of a `JobContext`, and the two
//! that write a file also read the process-wide `Config` for the deployment's
//! external URL. All three are constructible in a test process — `AppPaths`
//! from a `PathsConfig` pointing at a tempdir, `JobContext` from its public
//! constructor, and `Config` through `Config::install` — so the job bodies run
//! here against a throwaway database and a throwaway filesystem, rather than
//! only their pure helpers.

use std::path::Path;
use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::extension::{AssetDefinition, AssetType, ExtensionRegistry};
use systemprompt::identifiers::{Actor, UserId};
use systemprompt::models::config::RateLimitConfig;
use systemprompt::models::profile::{ContentNegotiationConfig, PathsConfig, SecurityHeadersConfig};
use systemprompt::models::{AppPaths, Config, PathResolution};
use systemprompt::traits::{Job, JobContext};
use systemprompt_web_jobs::{
    BundleAdminCssJob, ContentIngestionJob, ContentPrerenderJob, CopyExtensionAssetsJob,
    LlmsTxtGenerationJob, PublishPipelineJob, RobotsTxtGenerationJob, SecretMigrationJob,
    SitemapGenerationJob,
};
use tempfile::TempDir;

use crate::tempdb::TempDb;

// Stage count of `PublishPipelineJob`: ingestion, CSS bundle, asset copy,
// content prerender, page prerender, sitemap, llms.txt, robots.txt, feed, the
// unconditional success the pipeline records between them, and asset
// organisation. Asserting the total pins that no stage is silently dropped.
const PIPELINE_STAGES: u64 = 11;

// A 32-byte key, hex-encoded, so `load_master_key` accepts it. Under nextest
// each test is its own process and sets this before spawning anything, so
// there is no concurrent reader of the environment.
fn set_master_key() {
    unsafe {
        std::env::set_var(
            "ENCRYPTION_MASTER_KEY",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
    }
}

// Every job that reads the global `Config` reads exactly one field from it,
// and `Config` is a process-wide `OnceLock` — so all tests in this binary must
// agree on the value, or the first installer would silently decide it for the
// rest. Fixing it here makes the assertions below independent of test order.
const BASE_URL: &str = "https://astound.test";

const DOCUMENTATION_SOURCE_ENABLED: &str = "\
content_sources:\n\
\x20 documentation:\n\
\x20   path: content/documentation\n\
\x20   source_id: documentation\n\
\x20   category_id: documentation\n\
\x20   enabled: true\n\
\x20   sitemap:\n\
\x20     enabled: true\n\
\x20     url_pattern: /documentation/{slug}\n\
\x20     priority: 0.7\n\
\x20     changefreq: weekly\n\
\x20     fetch_from: database\n";

const DOCUMENTATION_SOURCE_DISABLED: &str = "\
content_sources:\n\
\x20 documentation:\n\
\x20   path: content/documentation\n\
\x20   source_id: documentation\n\
\x20   category_id: documentation\n\
\x20   enabled: false\n";

pub(crate) fn install_config() {
    if Config::is_initialized() {
        return;
    }
    let _ = Config::install(Config {
        instance_id: "jobs-context-tests".to_owned(),
        max_concurrent_streams: 16,
        sitename: "astound-test".to_owned(),
        database_type: "postgres".to_owned(),
        database_url: "postgres://unused".to_owned(),
        database_write_url: None,
        github_link: String::new(),
        github_token: None,
        system_path: "/tmp".to_owned(),
        services_path: "/tmp".to_owned(),
        bin_path: "/tmp".to_owned(),
        skills_path: "/tmp".to_owned(),
        settings_path: "/tmp".to_owned(),
        content_config_path: "/tmp".to_owned(),
        geoip_database_path: None,
        web_path: "/tmp".to_owned(),
        web_config_path: "/tmp".to_owned(),
        web_metadata_path: "/tmp".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        api_server_url: BASE_URL.to_owned(),
        api_internal_url: BASE_URL.to_owned(),
        api_external_url: BASE_URL.to_owned(),
        jwt_issuer: "https://issuer.test".to_owned(),
        jwt_access_token_expiration: 3_600,
        jwt_refresh_token_expiration: 86_400,
        jwt_audiences: vec![],
        allowed_resource_audiences: vec![],
        trusted_issuers: vec![],
        id_jag_ttl_secs: 300,
        signing_key_path: std::path::PathBuf::from("signing_key.pem"),
        use_https: true,
        rate_limits: RateLimitConfig::default(),
        cors_allowed_origins: vec![],
        trusted_proxies: vec![],
        is_cloud: false,
        content_negotiation: ContentNegotiationConfig::default(),
        security_headers: SecurityHeadersConfig::default(),
        allow_registration: false,
        login_page_url: None,
        system_admin_username: "admin".to_owned(),
        system_admin_email: None,
    });
}

// The jobs write into `paths.web().dist()` and read `paths.system()`, so the
// tree has to exist before a job runs — `AppPaths::from_profile` canonicalises,
// which a missing directory fails.
fn app_paths(root: &Path) -> Arc<AppPaths> {
    let root = root.to_string_lossy().to_string();
    Arc::new(
        AppPaths::from_profile(
            &PathsConfig {
                system: root.clone(),
                services: root.clone(),
                bin: root.clone(),
                web_path: Some(root.clone()),
                storage: Some(root),
                geoip_database: None,
            },
            PathResolution::Canonicalize,
        )
        .expect("build AppPaths over the temporary tree"),
    )
}

struct Harness {
    db: TempDb,
    tmp: TempDir,
    paths: Arc<AppPaths>,
}

impl Harness {
    async fn create() -> Option<Self> {
        install_config();
        let db = TempDb::create().await?;
        let tmp = TempDir::new().expect("temporary tree");
        let paths = app_paths(tmp.path());
        std::fs::create_dir_all(paths.web().dist()).expect("create the dist directory");
        Some(Self { db, tmp, paths })
    }

    fn context(&self) -> JobContext {
        let database = Arc::new(Database::from_pools(
            Arc::clone(&self.db.pool),
            Some(Arc::clone(&self.db.pool)),
        ));
        // The context type-erases each slot to `Arc<dyn Any>` and jobs downcast
        // it back to `DbPool` (itself an `Arc<Database>`) and `Arc<AppPaths>` —
        // so each value goes in wrapped in a second `Arc`, or the downcast
        // misses and the job reports the slot as absent.
        JobContext::new(
            Actor::user(UserId::new("jobs-context-test")),
            Arc::new(database),
            Arc::new(()),
            Arc::new(Arc::clone(&self.paths)),
        )
    }

    // A context whose pool and paths are the unit type: both downcasts miss,
    // which is the shape a job sees when the scheduler was wired wrong.
    fn empty_context() -> JobContext {
        JobContext::new(
            Actor::user(UserId::new("jobs-context-test")),
            Arc::new(()),
            Arc::new(()),
            Arc::new(()),
        )
    }

    // A context carrying the database but no `AppPaths`: the shape a job sees
    // when the scheduler was wired with half its dependencies.
    fn context_without_paths(&self) -> JobContext {
        let database = Arc::new(Database::from_pools(
            Arc::clone(&self.db.pool),
            Some(Arc::clone(&self.db.pool)),
        ));
        JobContext::new(
            Actor::user(UserId::new("jobs-context-test")),
            Arc::new(database),
            Arc::new(()),
            Arc::new(()),
        )
    }

    fn dist(&self) -> &Path {
        self.paths.web().dist()
    }

    fn css_dir(&self) -> &Path {
        self.paths.storage().css()
    }

    fn write_admin_css(&self, name: &str, contents: &str) {
        let dir = self.css_dir().join("admin");
        std::fs::create_dir_all(&dir).expect("create the admin CSS directory");
        std::fs::write(dir.join(name), contents).expect("write the stylesheet");
    }

    // The ingestion job resolves its config from this context's own paths, so a
    // per-test tree needs nothing process-global -- the file just has to sit
    // where those paths say.
    fn point_blog_config_at(&self, yaml: &str) {
        let dir = self.paths.system().services().join("config");
        std::fs::create_dir_all(&dir).expect("create the services config directory");
        std::fs::write(dir.join("blog.yaml"), yaml).expect("write the blog config");
    }

    fn context_with(&self, parameters: &[(&str, &str)]) -> JobContext {
        self.context().with_parameters(
            parameters
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        )
    }

    async fn seed_plaintext_secret(&self, id: &str, user_id: &str, name: &str, value: &str) {
        sqlx::query(
            "INSERT INTO plugin_env_vars (id, user_id, plugin_id, var_name, var_value, is_secret) \
             VALUES ($1, $2, 'test-plugin', $3, $4, true)",
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(value)
        .execute(&*self.db.pool)
        .await
        .expect("seed a plaintext secret");
    }

    fn write_content_config(&self, yaml: &str) {
        let path = self.paths.system().content_config();
        std::fs::create_dir_all(path.parent().expect("content config has a parent"))
            .expect("create the content config directory");
        std::fs::write(path, yaml).expect("write the content config");
    }

    async fn cleanup(self) {
        drop(self.tmp);
        self.db.cleanup().await;
    }
}

#[tokio::test]
async fn robots_txt_is_written_into_dist_against_the_configured_base_url() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let result = RobotsTxtGenerationJob
        .execute(&h.context())
        .await
        .expect("generate robots.txt");

    assert!(result.success);
    let written = std::fs::read_to_string(h.dist().join("robots.txt")).expect("robots.txt exists");
    assert!(
        written.contains(&format!("Sitemap: {BASE_URL}/sitemap.xml")),
        "the sitemap line points at the deployment's external URL: {written}"
    );
    assert!(
        written.contains("Disallow: /api/"),
        "the API surface stays out of the crawl: {written}"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn robots_txt_refuses_a_context_with_no_app_paths() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = RobotsTxtGenerationJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("without AppPaths there is nowhere to write");

    assert!(
        error.to_string().contains("AppPaths"),
        "the error names the missing context entry: {error}"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn llms_txt_lists_the_documentation_source_it_was_pointed_at() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_content_config(DOCUMENTATION_SOURCE_ENABLED);

    let result = LlmsTxtGenerationJob
        .execute(&h.context())
        .await
        .expect("generate llms.txt");

    assert!(result.success);
    let written = std::fs::read_to_string(h.dist().join("llms.txt")).expect("llms.txt exists");
    assert!(written.contains("## Documentation"));
    assert!(
        written.contains(&format!("- Homepage: {BASE_URL}")),
        "the header links back to the deployment: {written}"
    );
    assert!(
        written.contains(&format!("[Sitemap]({BASE_URL}/sitemap.xml)")),
        "the resources section links the sitemap: {written}"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn llms_txt_still_writes_a_file_when_the_documentation_source_is_disabled() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_content_config(DOCUMENTATION_SOURCE_DISABLED);

    LlmsTxtGenerationJob
        .execute(&h.context())
        .await
        .expect("generate llms.txt");

    let written = std::fs::read_to_string(h.dist().join("llms.txt")).expect("llms.txt exists");
    assert!(
        written.contains("## Documentation"),
        "the heading is unconditional; only its entries depend on the source"
    );
    assert!(
        !written.contains("### General"),
        "a disabled source contributes no entry sections: {written}"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn llms_txt_fails_when_the_content_config_is_absent() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = LlmsTxtGenerationJob
        .execute(&h.context())
        .await
        .expect_err("the job cannot decide which sources to list without its config");

    assert!(
        !error.to_string().is_empty(),
        "the missing config surfaces as an error rather than an empty manifest"
    );
    assert!(
        !h.dist().join("llms.txt").exists(),
        "nothing is written when the config could not be read"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn llms_txt_fails_on_a_malformed_content_config() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_content_config("content_sources: [this is a list, not a map]\n");

    let error = LlmsTxtGenerationJob
        .execute(&h.context())
        .await
        .expect_err("a content config that does not match the schema is fatal");

    assert!(!error.to_string().is_empty());

    h.cleanup().await;
}

#[tokio::test]
async fn llms_txt_refuses_a_context_with_no_database() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = LlmsTxtGenerationJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("the manifest is built from content rows");

    assert!(
        error.to_string().contains("DbPool"),
        "the error names the missing context entry: {error}"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn sitemap_generation_refuses_a_context_with_no_database() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = SitemapGenerationJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("the sitemap is built from content rows");

    assert!(error.to_string().contains("DbPool"));

    h.cleanup().await;
}

#[tokio::test]
async fn copy_extension_assets_copies_every_registered_required_asset() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let registry = ExtensionRegistry::discover().expect("discover extension registrations");
    let assets = registry.all_required_assets(h.paths.as_ref());
    // Sources live under the temporary tree because `AppPaths` was built over
    // it; creating each one is what makes the copy loop's success arm run.
    for (_, asset) in &assets {
        if let Some(parent) = asset.source().parent() {
            std::fs::create_dir_all(parent).expect("create the asset source directory");
        }
        std::fs::write(asset.source(), b"/* fixture */").expect("write the asset source");
    }

    let result = CopyExtensionAssetsJob::execute_copy(&h.paths)
        .await
        .expect("copy the registered assets");

    assert!(result.success);
    assert_eq!(
        result.items_processed,
        Some(u64::try_from(assets.len()).expect("asset count fits in u64")),
        "every registered asset is accounted for"
    );
    assert_eq!(result.items_failed, Some(0));
    for (_, asset) in &assets {
        assert!(
            h.dist().join(asset.destination()).exists(),
            "asset {} did not reach dist",
            asset.destination()
        );
    }

    h.cleanup().await;
}

#[tokio::test]
async fn copy_extension_assets_fails_when_a_required_source_is_missing() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = CopyExtensionAssetsJob::execute_copy(&h.paths)
        .await
        .expect_err("a required asset with no source file is fatal, not skipped");

    assert!(
        !error.to_string().is_empty(),
        "the failure names the copy that could not be made"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn copy_extension_assets_refuses_a_context_with_no_app_paths() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = CopyExtensionAssetsJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("without AppPaths there is nowhere to copy from or to");

    assert!(error.to_string().contains("AppPaths"));

    h.cleanup().await;
}

#[tokio::test]
async fn every_registered_job_declares_a_name_a_description_and_a_schedule() {
    let jobs = systemprompt_web_jobs::extension_jobs();

    assert!(!jobs.is_empty(), "the extension registers jobs");
    for job in &jobs {
        assert!(!job.name().is_empty());
        assert!(!job.description().is_empty());
        let fields = job.schedule().split_whitespace().count();
        assert!(
            fields == 6 || fields == 0,
            "job {} must declare a six-field cron expression or an empty one (run on demand \
             only), got {:?}",
            job.name(),
            job.schedule()
        );
        assert!(
            job.tags().contains(&systemprompt_web_jobs::JOB_TAG),
            "job {} is not tagged as belonging to this extension",
            job.name()
        );
    }
}

#[tokio::test]
async fn every_registered_job_reports_whether_it_is_enabled_and_schedulable() {
    let jobs = systemprompt_web_jobs::extension_jobs();

    for job in &jobs {
        assert!(
            job.enabled(),
            "job {} is registered but disabled",
            job.name()
        );
        assert!(
            job.schedulable() || job.schedule().is_empty(),
            "job {} declares a schedule but reports itself unschedulable",
            job.name()
        );
    }
}

#[tokio::test]
async fn bundle_admin_css_concatenates_the_admin_stylesheets_in_filename_order() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_admin_css("02-second.css", ".second {}");
    h.write_admin_css("01-first.css", ".first {}");

    let result = BundleAdminCssJob
        .execute(&h.context())
        .await
        .expect("bundle the admin CSS");

    assert!(result.success);
    assert_eq!(result.items_processed, Some(2));
    assert_eq!(result.items_failed, Some(0));
    let bundle = std::fs::read_to_string(h.css_dir().join("admin-bundle.css"))
        .expect("the bundle is written next to the admin directory");
    assert_eq!(bundle, ".first {}\n.second {}");

    h.cleanup().await;
}

#[tokio::test]
async fn bundle_admin_css_ignores_files_that_are_not_stylesheets() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_admin_css("01-only.css", ".only {}");
    h.write_admin_css("notes.txt", "not a stylesheet");

    let result = BundleAdminCssJob
        .execute(&h.context())
        .await
        .expect("bundle the admin CSS");

    assert_eq!(result.items_processed, Some(1));
    let bundle =
        std::fs::read_to_string(h.css_dir().join("admin-bundle.css")).expect("bundle written");
    assert_eq!(bundle, ".only {}");

    h.cleanup().await;
}

#[tokio::test]
async fn bundle_admin_css_fails_when_the_admin_directory_is_absent() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = BundleAdminCssJob
        .execute(&h.context())
        .await
        .expect_err("an unreadable input directory is a failure, not an empty bundle");

    assert!(!h.css_dir().join("admin-bundle.css").exists());
    let _ = error;
    h.cleanup().await;
}

// An entry that ends in `.css` but is a directory is collected and then fails
// to read — the one path that reaches the `failed > 0` guard.
#[tokio::test]
async fn bundle_admin_css_fails_when_a_collected_stylesheet_cannot_be_read() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_admin_css("01-good.css", ".good {}");
    std::fs::create_dir_all(h.css_dir().join("admin").join("02-broken.css"))
        .expect("create the unreadable entry");

    let error = BundleAdminCssJob
        .execute(&h.context())
        .await
        .expect_err("an unreadable stylesheet fails the job rather than silently shrinking it");

    assert!(
        error.to_string().contains("Failed to read 1 CSS file(s)"),
        "unexpected error: {error}"
    );
    h.cleanup().await;
}

#[tokio::test]
async fn bundle_admin_css_refuses_a_context_with_no_app_paths() {
    let error = BundleAdminCssJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("a context with no AppPaths cannot name an input directory");

    assert!(error.to_string().contains("AppPaths"));
}

#[tokio::test]
async fn content_prerender_refuses_a_context_with_no_database() {
    let error = ContentPrerenderJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("prerendering reads content out of the database");

    assert!(error.to_string().contains("DbPool"));
}

#[tokio::test]
async fn content_prerender_refuses_a_context_with_no_app_paths() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = ContentPrerenderJob
        .execute(&h.context_without_paths())
        .await
        .expect_err("prerendering has nowhere to write without AppPaths");

    assert!(error.to_string().contains("AppPaths"));
    h.cleanup().await;
}

#[tokio::test]
async fn secret_migration_does_nothing_when_no_master_key_is_configured() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let result = SecretMigrationJob
        .execute(&h.context())
        .await
        .expect("a missing master key is a skip, not a failure");

    assert!(result.success);
    assert_eq!(result.items_processed, Some(0));
    h.cleanup().await;
}

#[tokio::test]
async fn secret_migration_reports_no_work_when_every_secret_is_already_encrypted() {
    let Some(h) = Harness::create().await else {
        return;
    };
    set_master_key();

    let result = SecretMigrationJob
        .execute(&h.context())
        .await
        .expect("run the migration over an empty table");

    assert!(result.success);
    assert_eq!(result.items_processed, Some(0));
    assert_eq!(result.items_failed, Some(0));
    h.cleanup().await;
}

#[tokio::test]
async fn secret_migration_encrypts_plaintext_rows_and_audits_each_one() {
    let Some(h) = Harness::create().await else {
        return;
    };
    set_master_key();
    h.seed_plaintext_secret("secret-1", "user-a", "API_KEY", "plaintext-value")
        .await;
    h.seed_plaintext_secret("secret-2", "user-a", "OTHER_KEY", "another-value")
        .await;

    let result = SecretMigrationJob
        .execute(&h.context())
        .await
        .expect("migrate the plaintext rows");

    assert_eq!(result.items_processed, Some(2));
    assert_eq!(result.items_failed, Some(0));

    let rows = sqlx::query_as::<_, (String, Option<Vec<u8>>, i32)>(
        "SELECT var_value, encrypted_value, key_version FROM plugin_env_vars ORDER BY id",
    )
    .fetch_all(&*h.db.pool)
    .await
    .expect("read the migrated rows back");
    assert_eq!(rows.len(), 2);
    for (var_value, encrypted, key_version) in &rows {
        assert!(var_value.is_empty(), "the plaintext must be cleared");
        assert!(
            encrypted.as_ref().is_some_and(|e| !e.is_empty()),
            "the ciphertext must replace it"
        );
        assert!(*key_version >= 1);
    }

    let audited: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM secret_audit_log")
        .fetch_one(&*h.db.pool)
        .await
        .expect("count the audit rows");
    assert_eq!(audited, 2, "every migrated secret is audited");

    h.cleanup().await;
}

#[tokio::test]
async fn secret_migration_refuses_a_context_with_no_database() {
    set_master_key();

    let error = SecretMigrationJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("the migration reads and rewrites database rows");

    assert!(error.to_string().contains("Database not available"));
}

#[tokio::test]
async fn publish_pipeline_runs_every_stage_and_reports_each_outcome() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.write_admin_css("01-first.css", ".first {}");

    let result = PublishPipelineJob
        .execute(&h.context())
        .await
        .expect("the pipeline reports sub-job failures in its stats rather than as an error");

    assert!(result.success);
    let succeeded = result.items_processed.unwrap_or(0);
    let failed = result.items_failed.unwrap_or(0);
    assert_eq!(
        succeeded + failed,
        PIPELINE_STAGES,
        "every stage must be accounted for exactly once"
    );
    assert!(
        succeeded > 0,
        "the CSS bundle stage at least runs against the temporary tree"
    );
    assert!(
        h.css_dir().join("admin-bundle.css").exists(),
        "the pipeline ran the bundle stage against AppPaths, not the process working directory"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn publish_pipeline_refuses_a_context_with_no_database() {
    let error = PublishPipelineJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("the pipeline needs a database before it runs a single stage");

    assert!(error.to_string().contains("Database not available"));
}

#[tokio::test]
async fn publish_pipeline_refuses_a_context_with_no_app_paths() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let error = PublishPipelineJob
        .execute(&h.context_without_paths())
        .await
        .expect_err("the pipeline needs somewhere to write before it runs a single stage");

    assert!(error.to_string().contains("AppPaths"));
    h.cleanup().await;
}

#[tokio::test]
async fn content_ingestion_walks_every_enabled_source_the_blog_config_names() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let tree = h.tmp.path().join("guides");
    write_article(&tree, "first.md", "first");
    write_article(&tree, "second.md", "second");
    write_article(&h.tmp.path().join("disabled"), "third.md", "third");
    h.point_blog_config_at(&format!(
        "content_sources:\n  - source_id: guides\n    category_id: guides\n    path: \
         {tree}\n  - source_id: archive\n    category_id: archive\n    path: {disabled}\n    \
         enabled: false\n",
        tree = tree.display(),
        disabled = h.tmp.path().join("disabled").display(),
    ));

    let result = ContentIngestionJob
        .execute(&h.context())
        .await
        .expect("ingest the configured sources");

    assert!(result.success);
    assert_eq!(
        result.items_processed,
        Some(2),
        "the disabled source is not walked"
    );
    assert_eq!(result.items_failed, Some(0));

    let slugs = ingested_slugs(&h).await;
    assert_eq!(slugs, vec!["first".to_owned(), "second".to_owned()]);

    h.cleanup().await;
}

#[tokio::test]
async fn content_ingestion_counts_a_malformed_file_without_failing_the_job() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let tree = h.tmp.path().join("guides");
    write_article(&tree, "good.md", "good");
    std::fs::write(tree.join("bad.md"), "no frontmatter here").expect("write the malformed file");
    h.point_blog_config_at(&single_source_config(&tree));

    let result = ContentIngestionJob
        .execute(&h.context())
        .await
        .expect("a malformed file is counted, not raised");

    assert_eq!(result.items_processed, Some(1));
    assert_eq!(result.items_failed, Some(1));

    h.cleanup().await;
}

#[tokio::test]
async fn content_ingestion_prunes_orphans_only_when_the_environment_asks_for_it() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let tree = h.tmp.path().join("guides");
    write_article(&tree, "kept.md", "kept");
    h.point_blog_config_at(&single_source_config(&tree));
    seed_orphan(&h, "vanished").await;

    ContentIngestionJob
        .execute(&h.context_with(&[("delete_orphans", "true")]))
        .await
        .expect("ingest with pruning on");

    assert_eq!(
        ingested_slugs(&h).await,
        vec!["kept".to_owned()],
        "a row whose file is gone is pruned when the flag is set"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn content_ingestion_is_skipped_when_the_profile_has_no_blog_config() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let result = ContentIngestionJob
        .execute(&h.context())
        .await
        .expect("no blog config is a supported state, not a failure");

    assert!(result.success);
    assert_eq!(result.message.as_deref(), Some("skipped: no blog config"));

    h.cleanup().await;
}

#[tokio::test]
async fn content_ingestion_fails_when_the_blog_config_does_not_validate() {
    let Some(h) = Harness::create().await else {
        return;
    };
    h.point_blog_config_at(&single_source_config(&h.tmp.path().join("never-created")));

    let error = ContentIngestionJob
        .execute(&h.context())
        .await
        .expect_err("an enabled source pointing at nothing is a configuration error");

    assert!(
        error.to_string().contains("Failed to load blog config"),
        "unexpected error: {error}"
    );

    h.cleanup().await;
}

#[tokio::test]
async fn content_ingestion_refuses_a_context_with_no_database() {
    let error = ContentIngestionJob
        .execute(&Harness::empty_context())
        .await
        .expect_err("ingestion writes rows and so needs a write pool");

    assert!(error.to_string().contains("Database not available"));
}

fn single_source_config(tree: &Path) -> String {
    format!(
        "content_sources:\n  - source_id: guides\n    category_id: guides\n    path: {}\n",
        tree.display()
    )
}

fn write_article(dir: &Path, name: &str, slug: &str) {
    std::fs::create_dir_all(dir).expect("create the content tree");
    std::fs::write(
        dir.join(name),
        format!(
            "---\ntitle: Title for {slug}\ndescription: Description for {slug}\nauthor: Test \
             Author\npublished_at: 2026-01-01\nslug: {slug}\nkeywords: alpha\nkind: \
             blog\n---\nBody for {slug}\n"
        ),
    )
    .expect("write the article");
}

async fn ingested_slugs(h: &Harness) -> Vec<String> {
    sqlx::query_scalar::<_, String>("SELECT slug FROM markdown_content ORDER BY slug")
        .fetch_all(&*h.db.pool)
        .await
        .expect("read the ingested slugs back")
}

async fn seed_orphan(h: &Harness, slug: &str) {
    systemprompt_web_content::repository::ContentRepository::new(Arc::clone(&h.db.pool))
        .create(&crate::fixtures::content_params(
            slug,
            &systemprompt::identifiers::SourceId::new("guides".to_owned()),
        ))
        .await
        .expect("seed a row whose file does not exist");
}

// A failing *optional* asset is counted, not fatal — the branch the registered
// assets (all required) never take.
#[tokio::test]
async fn an_optional_asset_that_cannot_be_copied_is_counted_and_not_fatal() {
    let tmp = TempDir::new().expect("temporary tree");
    let assets = vec![(
        "test-extension",
        AssetDefinition::builder(
            tmp.path().join("absent.css"),
            "css/absent.css",
            AssetType::Css,
        )
        .optional()
        .build(),
    )];

    let (copied, failed) = systemprompt_web_jobs::internals::copy_all_assets(tmp.path(), assets)
        .await
        .expect("an optional asset failure is not an error");

    assert_eq!((copied, failed), (0, 1));
}

#[tokio::test]
async fn a_required_asset_that_cannot_be_copied_fails_the_copy() {
    let tmp = TempDir::new().expect("temporary tree");
    let assets = vec![(
        "test-extension",
        AssetDefinition::css(tmp.path().join("absent.css"), "css/absent.css"),
    )];

    let error = systemprompt_web_jobs::internals::copy_all_assets(tmp.path(), assets)
        .await
        .expect_err("a required asset failure stops the copy");

    let _ = error;
}

// `SitemapGenerationJob` and `ContentPrerenderJob` are not driven to
// completion here. Both delegate to a core generator that loads the full
// `WebConfig` from the *global* `Config`'s `web_config_path` — a
// process-wide value fixed by the first test to install a config, which
// cannot point at the per-test temporary tree the job writes into. Their
// context lookups are asserted above, and the publish pipeline drives both
// through their failure paths; rendering a fixture site is what the contract
// suite does.
