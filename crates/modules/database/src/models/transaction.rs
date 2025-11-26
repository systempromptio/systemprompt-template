use crate::models::query_selector::QuerySelector;
use crate::models::types::{JsonRow, ToDbValue};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DatabaseTransaction: Send {
    async fn execute(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<u64>;

    async fn fetch_all(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Vec<JsonRow>>;

    async fn fetch_one(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<JsonRow>;

    async fn fetch_optional(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<JsonRow>>;

    async fn commit(self: Box<Self>) -> Result<()>;

    async fn rollback(self: Box<Self>) -> Result<()>;
}
