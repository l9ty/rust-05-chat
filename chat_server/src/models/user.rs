use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Deserialize;
use sqlx::query_as;

use crate::error::AppError;

use super::User;

impl User {
    pub async fn create(pool: &sqlx::PgPool, input: &CreateUser) -> Result<User, AppError> {
        // check email is exist
        let user = query_as!(
            User,
            "SELECT id, fullname, email, password_hash, create_at FROM users WHERE email = $1",
            input.email
        )
        .fetch_optional(pool)
        .await?;

        if user.is_some() {
            return Err(AppError::UserAlreadyExist);
        }

        let password_hash = hash_password(&input.password)?;

        let user = query_as!(
            User,
            r#"
            INSERT INTO users (fullname, email, password_hash) VALUES ($1, $2, $3)
            RETURNING id, fullname, email, password_hash, create_at
            "#,
            input.fullname,
            input.email,
            password_hash
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    // Return None if user not found or password is wrong
    pub async fn signin(
        pool: &sqlx::PgPool,
        input: &SigninInput,
    ) -> Result<Option<User>, AppError> {
        let user = query_as!(
            User,
            "SELECT id, fullname, email, password_hash, create_at FROM users WHERE email = $1",
            input.email
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::SqlxError)?;

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

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::PasswordHashError(e.to_string()))
        .map(|hash| hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let hash = PasswordHash::new(hash).map_err(|e| AppError::PasswordHashError(e.to_string()))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_password() {
        let password = "123456";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }
}
