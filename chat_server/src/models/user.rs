use std::mem;

use anyhow::{anyhow, Context};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chat_core::User;
use serde::Deserialize;

use crate::error::AppError;

pub async fn create(pool: &sqlx::PgPool, input: &CreateUser) -> Result<User, AppError> {
    let user: Option<User> = sqlx::query_as(
        r#"
        SELECT id, fullname, email, password_hash, ws_id, created_at
        FROM users WHERE email = $1
        "#,
    )
    .bind(&input.email)
    .fetch_optional(pool)
    .await
    .context("find user failed")?;

    if user.is_some() {
        return Err(AppError::already_exist("user"));
    }

    let password_hash = hash_password(&input.password)?;

    let user = sqlx::query_as(
        r#"
        INSERT INTO users (fullname, email, password_hash, ws_id) VALUES ($1, $2, $3, 0)
        RETURNING id, fullname, email, password_hash, ws_id, created_at"#,
    )
    .bind(&input.fullname)
    .bind(&input.email)
    .bind(&password_hash)
    .fetch_one(pool)
    .await
    .context("create user failed")?;

    Ok(user)
}

// Return None if user not found or password is wrong
pub async fn signin(pool: &sqlx::PgPool, input: &SigninInput) -> Result<Option<User>, AppError> {
    let user: Option<User> = sqlx::query_as(
        r#"
        SELECT id, fullname, email, password_hash, ws_id, created_at FROM users WHERE email = $1
        "#,
    )
    .bind(&input.email)
    .fetch_optional(pool)
    .await?;

    let Some(mut user) = user else {
        return Ok(None);
    };

    // clear password_hash
    let hash = mem::take(&mut user.password_hash);
    match verify_password(&input.password, &hash)? {
        true => Ok(Some(user)),
        false => Ok(None),
    }
}

#[derive(Deserialize)]
pub struct SigninInput {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub password: String,
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("password hash error: {}", e))
        .map(|hash| hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let hash = PasswordHash::new(hash).map_err(|e| anyhow!("password hash error: {}", e))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
}

#[cfg(test)]
mod tests {
    use crate::models;

    use super::*;
    use sqlx::PgPool;

    #[test]
    fn t_argon2_hash() {
        let password = "123456laskjdlasjl";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[sqlx::test(migrator = "crate::tests::MIGRATOR")]
    async fn t_create(pool: PgPool) {
        let input = CreateUser {
            fullname: "test".to_string(),
            email: "b@a.com".to_string(),
            password: "123456".to_string(),
        };

        let user = models::user::create(&pool, &input).await.unwrap();
        assert_eq!(user.fullname, input.fullname);
        assert_eq!(user.email, input.email);
        assert!(verify_password(&input.password, &user.password_hash).unwrap());
    }
}
