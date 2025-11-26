use crate::models::analytics::{ClientAnalytics, ClientErrorAnalytics};
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_identifiers::ClientId;

#[derive(Debug)]
pub struct AnalyticsRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl AnalyticsRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn get_client_analytics(&self) -> Result<Vec<ClientAnalytics>> {
        let rows = self
            .db
            .fetch_all(
                &DatabaseQueryEnum::GetClientAnalytics.get(self.db.as_ref()),
                &[],
            )
            .await?;

        rows.iter()
            .map(ClientAnalytics::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_client_analytics_by_id(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<ClientAnalytics>> {
        let row = self
            .db
            .fetch_optional(
                &DatabaseQueryEnum::GetClientAnalyticsById.get(self.db.as_ref()),
                &[&client_id.as_str()],
            )
            .await?;

        row.map(|r| ClientAnalytics::from_json_row(&r)).transpose()
    }

    pub async fn get_client_errors(&self) -> Result<Vec<ClientErrorAnalytics>> {
        let rows = self
            .db
            .fetch_all(
                &DatabaseQueryEnum::GetClientErrors.get(self.db.as_ref()),
                &[],
            )
            .await?;

        rows.iter()
            .map(ClientErrorAnalytics::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_client_errors_by_id(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<ClientErrorAnalytics>> {
        let row = self
            .db
            .fetch_optional(
                &DatabaseQueryEnum::GetClientErrorsById.get(self.db.as_ref()),
                &[&client_id.as_str()],
            )
            .await?;

        row.map(|r| ClientErrorAnalytics::from_json_row(&r))
            .transpose()
    }
}
