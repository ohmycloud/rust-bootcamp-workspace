use std::str::FromStr;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde::{ Serialize, Deserialize};
use crate::{AppError, AppState};
use crate::models::ChatFile;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

impl AppState {
    pub async fn create_message(&self, input: CreateMessage, chat_id: i64, user_id: i64) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        // verify content - not empty
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError("Content cannot be empty".to_owned()));
        }
        // verify files
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(
                    format!("File {} does not exist", s)));
            }
        }
        // create message
        let message: Message = sqlx::query_as(
            r#"
        INSERT INTO messages(chat_id, sender_id, content,files)
        VALUES ($1, $2, $3, $4)
        RETURNING id, chat_id, sender_id, content, files, created_at"#,
        )
            .bind(chat_id)
            .bind(user_id)
            .bind(input.content)
            .bind(&input.files)
            .fetch_one(&self.pool)
            .await?;

        Ok(message)
    }
}