use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_traits::RepositoryError;

use crate::models::a2a::Part;

pub async fn get_message_parts(
    pool: &Arc<PgPool>,
    message_id: &str,
) -> Result<Vec<Part>, RepositoryError> {
    let part_rows: Vec<crate::models::MessagePart> = sqlx::query_as!(
        crate::models::MessagePart,
        r#"SELECT
            id as "id!",
            message_id as "message_id!",
            task_id as "task_id!",
            part_kind as "part_kind!",
            sequence_number as "sequence_number!",
            text_content,
            file_name,
            file_mime_type,
            file_uri,
            file_bytes,
            data_content,
            metadata
        FROM message_parts WHERE message_id = $1 ORDER BY sequence_number ASC"#,
        message_id
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let mut parts = Vec::new();

    for row in part_rows {
        let part = match row.part_kind.as_str() {
            "text" => {
                let text = row
                    .text_content
                    .ok_or_else(|| RepositoryError::InvalidData("Missing text_content".into()))?;
                Part::Text(crate::models::a2a::TextPart { text })
            },
            "file" => {
                let bytes = row
                    .file_bytes
                    .ok_or_else(|| RepositoryError::InvalidData("Missing file_bytes".into()))?;
                Part::File(crate::models::a2a::FilePart {
                    file: crate::models::a2a::FileWithBytes {
                        name: row.file_name,
                        mime_type: row.file_mime_type,
                        bytes,
                    },
                })
            },
            "data" => {
                let data_value = row
                    .data_content
                    .ok_or_else(|| RepositoryError::InvalidData("Missing data_content".into()))?;
                let data = if let serde_json::Value::Object(map) = data_value {
                    map
                } else {
                    return Err(RepositoryError::InvalidData(
                        "Data content must be a JSON object".into(),
                    ));
                };
                Part::Data(crate::models::a2a::DataPart { data })
            },
            _ => {
                return Err(RepositoryError::InvalidData(format!(
                    "Unknown part kind: {}",
                    row.part_kind
                )));
            },
        };

        parts.push(part);
    }

    Ok(parts)
}

pub async fn persist_part_sqlx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    part: &Part,
    message_id: &str,
    task_id: &str,
    sequence_number: i32,
) -> Result<(), RepositoryError> {
    match part {
        Part::Text(text_part) => {
            sqlx::query!(
                r#"INSERT INTO message_parts (message_id, task_id, part_kind, sequence_number, text_content)
                VALUES ($1, $2, 'text', $3, $4)"#,
                message_id,
                task_id,
                sequence_number,
                text_part.text
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        },
        Part::File(file_part) => {
            let file_uri: Option<&str> = None;
            sqlx::query!(
                r#"INSERT INTO message_parts (message_id, task_id, part_kind, sequence_number, file_name, file_mime_type, file_uri, file_bytes)
                VALUES ($1, $2, 'file', $3, $4, $5, $6, $7)"#,
                message_id,
                task_id,
                sequence_number,
                file_part.file.name,
                file_part.file.mime_type,
                file_uri,
                file_part.file.bytes
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        },
        Part::Data(data_part) => {
            let data_json = serde_json::to_value(&data_part.data)
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            sqlx::query!(
                r#"INSERT INTO message_parts (message_id, task_id, part_kind, sequence_number, data_content)
                VALUES ($1, $2, 'data', $3, $4)"#,
                message_id,
                task_id,
                sequence_number,
                data_json
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        },
    }

    Ok(())
}

pub async fn persist_part_with_tx(
    tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
    part: &Part,
    message_id: &str,
    task_id: &str,
    sequence_number: i32,
) -> Result<(), RepositoryError> {
    match part {
        Part::Text(text_part) => {
            let query: &str = "INSERT INTO message_parts (message_id, task_id, part_kind, \
                               sequence_number, text_content) VALUES ($1, $2, 'text', $3, $4)";
            tx.execute(
                &query,
                &[&message_id, &task_id, &sequence_number, &text_part.text],
            )
            .await?;
        },
        Part::File(file_part) => {
            let uri_opt: Option<&str> = None;
            let query: &str = "INSERT INTO message_parts (message_id, task_id, part_kind, \
                               sequence_number, file_name, file_mime_type, file_uri, file_bytes) \
                               VALUES ($1, $2, 'file', $3, $4, $5, $6, $7)";
            tx.execute(
                &query,
                &[
                    &message_id,
                    &task_id,
                    &sequence_number,
                    &file_part.file.name,
                    &file_part.file.mime_type,
                    &uri_opt,
                    &file_part.file.bytes,
                ],
            )
            .await?;
        },
        Part::Data(data_part) => {
            let data_json = serde_json::to_string(&data_part.data)?;
            let query: &str = "INSERT INTO message_parts (message_id, task_id, part_kind, \
                               sequence_number, data_content) VALUES ($1, $2, 'data', $3, $4)";
            tx.execute(
                &query,
                &[&message_id, &task_id, &sequence_number, &data_json],
            )
            .await?;
        },
    }

    Ok(())
}
