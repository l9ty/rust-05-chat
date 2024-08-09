pub mod user;
pub mod workspace;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

pub type RowID = i64;

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: RowID,
    pub fullname: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub ws_id: RowID,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Workspace {
    pub id: RowID,
    pub name: String,
    pub owner_id: RowID,
    pub created_at: DateTime<Utc>,
}
