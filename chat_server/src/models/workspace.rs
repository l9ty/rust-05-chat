use sqlx::PgPool;

use crate::error::AppError;

use super::{RowID, User, Workspace};

#[allow(unused)]
impl Workspace {
    pub async fn create(
        pool: &PgPool,
        name: &str,
        user_id: RowID,
    ) -> Result<Option<Workspace>, AppError> {
        let ret = sqlx::query("SELECT id FROM workspaces WHERE name = $1 LIMIT 1")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        match ret {
            Some(_) => Ok(None),
            None => {
                let ws = sqlx::query_as(
                    "INSERT INTO workspaces (name, owner_id) VALUES ($1, $2) RETURNING *",
                )
                .bind(name)
                .bind(user_id)
                .fetch_one(pool)
                .await?;
                Ok(Some(ws))
            }
        }
    }

    pub async fn update_owner(pool: &PgPool, ws_id: RowID, user_id: RowID) -> Result<(), AppError> {
        sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
            .bind(user_id)
            .bind(ws_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as("SELECT * FROM workspaces WHERE name = $1 LIMIT 1")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        Ok(ws)
    }

    pub async fn find_by_id(pool: &PgPool, id: RowID) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as("SELECT * FROM workspaces WHERE id = $1 LIMIT 1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(ws)
    }

    pub async fn list_all_users(pool: &PgPool, ws_id: RowID) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as("SELECT * FROM users WHERE ws_id = $1")
            .bind(ws_id)
            .fetch_all(pool)
            .await?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::MIGRATOR;

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn create(pool: PgPool) {
        // find by id should return none
        let ret = Workspace::find_by_id(&pool, 1).await.unwrap();
        assert!(ret.is_none());

        // find by name should return none
        let ret = Workspace::find_by_name(&pool, "test").await.unwrap();
        assert!(ret.is_none());

        // create should work
        let ws = Workspace::create(&pool, "test", 1).await.unwrap();
        assert!(ws.is_some());

        // create again should return none
        let ret = Workspace::create(&pool, "test", 1).await.unwrap();
        assert!(ret.is_none());

        // find by name should work
        let ret = Workspace::find_by_name(&pool, "test").await.unwrap();
        assert!(ret.is_some());

        // find by id should work
        let ret = Workspace::find_by_id(&pool, 1).await.unwrap();
        assert!(ret.is_some());

        // update owner should work
        let ret = Workspace::update_owner(&pool, 1, 2).await;
        assert!(ret.is_ok());

        // list all users should work
        let ret = Workspace::list_all_users(&pool, 1).await;
        assert!(ret.is_ok());
    }
}
