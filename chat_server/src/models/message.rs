use std::path::Path;

use chat_core::{Message, RowID};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
pub struct CreateMessage {
    #[serde(default)]
    pub chat_id: RowID,
    pub content: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Deserialize)]
pub struct ListMessage {
    pub last_id: Option<RowID>,
    pub limit: u32,
}

pub async fn list(pool: &PgPool, chat_id: RowID, input: ListMessage) -> AppResult<Vec<Message>> {
    let messages = sqlx::query_as(
        r#"
            SELECT id, chat_id, sender_id, content, files, created_at
            FROM messages
            WHERE chat_id = $1 AND id < $2
            ORDER BY created_at DESC
            LIMIT $3
        "#,
    )
    .bind(chat_id)
    .bind(input.last_id.unwrap_or(RowID::MAX))
    .bind(input.limit as i64)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn create(
    pool: &PgPool,
    input: CreateMessage,
    uid: RowID,
    base_dir: &Path,
) -> AppResult<Message> {
    input.verify(base_dir)?;

    let message = sqlx::query_as(
        r#"
            INSERT INTO messages (chat_id, sender_id, content, files)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#,
    )
    .bind(input.chat_id)
    .bind(uid)
    .bind(input.content)
    .bind(input.files)
    .fetch_one(pool)
    .await?;

    Ok(message)
}

impl CreateMessage {
    pub fn verify(&self, base_dir: &Path) -> AppResult<()> {
        // content should not be empty
        if self.content.is_empty() {
            return Err(AppError::invalid_input("empty content"));
        }

        // files should be exist
        for file in &self.files {
            let file_path = base_dir.join(file);
            if !file_path.exists() {
                return Err(AppError::invalid_input("some files not exist"));
            }
        }
        Ok(())
    }
}
