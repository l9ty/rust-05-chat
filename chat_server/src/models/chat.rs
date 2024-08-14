use chat_core::{Chat, ChatType, RowID};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<RowID>,
    pub public: bool,
}

pub async fn create(pool: &PgPool, ws_id: RowID, input: CreateChat) -> AppResult<Chat> {
    if input.members.len() < 2 {
        return Err(AppError::invalid_input("chat must have at least 2 members"));
    }
    if input.members.len() > 5 && input.name.is_some() {
        return Err(AppError::invalid_input(
            "group chat with more than 5 members must have a name",
        ));
    }

    // all members must exist
    let rows = sqlx::query("SELECT id FROM users WHERE id = ANY($1)")
        .bind(&input.members)
        .fetch_all(pool)
        .await?;
    if rows.len() != input.members.len() {
        return Err(AppError::invalid_input("some members don't exist"));
    }

    let typ = match (&input.name, input.members.len()) {
        (None, 2) => ChatType::Single,
        (None, _) => ChatType::Group,
        (Some(_), _) => {
            if input.public {
                ChatType::PublicChannel
            } else {
                ChatType::PrivateChannel
            }
        }
    };

    let chat = sqlx::query_as(
        r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
    )
    .bind(ws_id)
    .bind(input.name)
    .bind(typ)
    .bind(input.members)
    .fetch_one(pool)
    .await?;

    Ok(chat)
}

pub async fn list(pool: &PgPool, ws_id: RowID, uid: RowID) -> AppResult<Vec<Chat>> {
    let chats = sqlx::query_as(
        r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1 AND $2 = ANY(members)
        "#,
    )
    .bind(ws_id)
    .bind(uid)
    .fetch_all(pool)
    .await?;
    Ok(chats)
}

pub async fn get(pool: &PgPool, id: RowID) -> AppResult<Chat> {
    let chat = sqlx::query_as("SELECT * FROM chats WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(chat)
}

pub async fn is_member(pool: &PgPool, chat_id: RowID, uid: RowID) -> AppResult<bool> {
    let ret = sqlx::query("SELECT 1 FROM chats WHERE id = $1 AND $2 = ANY(members)")
        .bind(chat_id)
        .bind(uid)
        .fetch_optional(pool)
        .await?;
    Ok(ret.is_some())
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../../fixtures/test.sql")
    )]
    async fn t_create_single(pool: PgPool) {
        let input = CreateChat {
            members: vec![1, 2],
            name: None,
            public: false,
        };
        let chat = create(&pool, 1, input).await.unwrap();
        assert_eq!(chat.members, vec![1, 2]);
        matches!(chat.typ, ChatType::Single);
        assert!(chat.name.is_none());
    }

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../../fixtures/test.sql")
    )]
    async fn t_create_named_public(pool: PgPool) {
        let input = CreateChat {
            members: vec![1, 2],
            name: Some("test".to_string()),
            public: true,
        };
        let chat = create(&pool, 1, input).await.unwrap();
        assert_eq!(chat.members, vec![1, 2]);
        matches!(chat.typ, ChatType::PublicChannel);
        assert!(chat.name.is_some());
    }

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../../fixtures/test.sql")
    )]
    async fn t_list(pool: PgPool) {
        // user-1 has one chat
        let chats = list(&pool, 1, 1).await.unwrap();
        assert_eq!(chats.len(), 1);
    }
}
