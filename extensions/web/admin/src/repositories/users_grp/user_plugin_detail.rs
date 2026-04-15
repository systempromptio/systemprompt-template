use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::UserPluginWithAssociations;

pub async fn get_plugin_with_associations(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<Option<UserPluginWithAssociations>, sqlx::Error> {
    crate::repositories::user_plugins::find_plugin_with_associations(pool, user_id, plugin_id).await
}
