pub mod middlewares;
pub mod utils;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

pub type RowID = i64;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: RowID,
    pub fullname: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub ws_id: RowID,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Workspace {
    pub id: RowID,
    pub name: String,
    pub owner_id: RowID,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Chat {
    pub id: RowID,
    pub ws_id: RowID,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub typ: ChatType,
    pub members: Vec<RowID>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: RowID,
    pub chat_id: RowID,
    pub sender_id: RowID,
    pub content: String,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
}
