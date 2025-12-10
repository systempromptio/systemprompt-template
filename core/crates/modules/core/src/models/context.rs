use crate::models::modules::ModuleApiRegistry;
use crate::services::AnalyticsService;
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::{Database, DbPool};
use systemprompt_core_logging::{CliService, LogService};
use systemprompt_models::{Config, ContentConfig, RouteClassifier, SystemPaths};
use systemprompt_traits::{AppContext as AppContextTrait, ConfigProvider, DatabaseHandle};

pub type GeoIpReader = Arc<maxminddb::Reader<Vec<u8>>>;

#[derive(Clone)]
pub struct AppContext {
    config: Arc<Config>,
    database: DbPool,
    api_registry: Arc<ModuleApiRegistry>,
    pub log: LogService,
    geoip_reader: Option<GeoIpReader>,
    content_config: Option<Arc<ContentConfig>>,
    #[allow(dead_code)]
    route_classifier: Arc<RouteClassifier>,
    analytics_service: Arc<AnalyticsService>,
}

impl std::fmt::Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContext")
            .field("config", &"Config")
            .field("database", &"DbPool")
            .field("api_registry", &"ModuleApiRegistry")
            .field("log", &"LogService")
            .field("geoip_reader", &self.geoip_reader.is_some())
            .field("content_config", &self.content_config.is_some())
            .field("route_classifier", &"RouteClassifier")
            .field("analytics_service", &"AnalyticsService")
            .finish()
    }
}

impl AppContext {
    pub async fn new() -> Result<Self> {
        dotenvy::dotenv().ok();
        let config = Arc::new(Config::from_env()?);
        let database =
            Arc::new(Database::from_config(&config.database_type, &config.database_url).await?);

        let api_registry = Arc::new(ModuleApiRegistry::new());
        let log = LogService::system(database.clone());

        let geoip_reader = Self::load_geoip_database();
        let content_config = Self::load_content_config(&config);

        let route_classifier = Arc::new(RouteClassifier::new(content_config.clone()));

        let analytics_service = Arc::new(AnalyticsService::new(
            database.clone(),
            geoip_reader.clone(),
            content_config.clone(),
        ));

        Ok(Self {
            config,
            database,
            api_registry,
            log,
            geoip_reader,
            content_config,
            route_classifier,
            analytics_service,
        })
    }

    fn load_geoip_database() -> Option<GeoIpReader> {
        let Ok(geoip_path) = std::env::var("GEOIP_DATABASE_PATH") else {
            CliService::warning(
                "GEOIP_DATABASE_PATH not set - geographic data will not be available",
            );
            return None;
        };

        match maxminddb::Reader::open_readfile(&geoip_path) {
            Ok(reader) => Some(Arc::new(reader)),
            Err(e) => {
                CliService::warning(&format!(
                    "Could not load GeoIP database from {geoip_path}: {e}"
                ));
                CliService::info("  Geographic data (country/region/city) will not be available.");
                None
            },
        }
    }

    fn load_content_config(config: &Config) -> Option<Arc<ContentConfig>> {
        let content_config_path = SystemPaths::content_config(config);

        match ContentConfig::load_from_file(&content_config_path) {
            Ok(content_cfg) => Some(Arc::new(content_cfg)),
            Err(e) => {
                CliService::warning(&format!(
                    "Could not load content config from {}: {}",
                    content_config_path.display(),
                    e
                ));
                CliService::info("  Landing page detection will not be available.");
                None
            },
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn content_config(&self) -> Option<&ContentConfig> {
        self.content_config.as_ref().map(AsRef::as_ref)
    }

    pub const fn db_pool(&self) -> &DbPool {
        &self.database
    }

    pub const fn database(&self) -> &DbPool {
        &self.database
    }

    pub fn api_registry(&self) -> &ModuleApiRegistry {
        &self.api_registry
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }

    pub fn jwt_secret(&self) -> &str {
        &self.config.jwt_secret
    }

    pub fn get_provided_audiences() -> Vec<String> {
        vec!["a2a".to_string(), "api".to_string(), "mcp".to_string()]
    }

    pub fn get_valid_audiences(_module_name: &str) -> Vec<String> {
        Self::get_provided_audiences()
    }

    pub fn get_server_audiences(_server_name: &str, _port: u16) -> Vec<String> {
        Self::get_provided_audiences()
    }

    pub const fn geoip_reader(&self) -> Option<&GeoIpReader> {
        self.geoip_reader.as_ref()
    }

    pub const fn analytics_service(&self) -> &Arc<AnalyticsService> {
        &self.analytics_service
    }

    pub const fn route_classifier(&self) -> &Arc<RouteClassifier> {
        &self.route_classifier
    }
}

impl AppContextTrait for AppContext {
    fn config(&self) -> Arc<dyn ConfigProvider> {
        self.config.clone()
    }

    fn database_handle(&self) -> Arc<dyn DatabaseHandle> {
        self.database.clone()
    }
}
