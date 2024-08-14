use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::AppError,
    models::{
        self,
        user::{CreateUser, SigninInput},
    },
    AppState,
};

pub async fn singup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<Response<Body>, AppError> {
    let user = models::user::create(&state.db, &input).await?;
    let token = state.ek.sign(&user.into())?;
    let body = SigninOutput { token };
    Ok((StatusCode::CREATED, Json(body)).into_response())
}

pub async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninInput>,
) -> Result<Response<Body>, AppError> {
    let user = models::user::signin(&state.db, &input).await?;
    let Some(user) = user else {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "invalid email or password"
            })),
        )
            .into_response());
    };
    let token = state.ek.sign(&user.into())?;
    Ok(Json(SigninOutput { token }).into_response())
}

#[derive(Serialize, Deserialize)]
struct SigninOutput {
    token: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::MIGRATOR;

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn t_signup_signin(pool: sqlx::PgPool) {
        let state = AppState::new_for_test(pool);

        // sigin with non exist email
        let input = SigninInput {
            email: "a@a.com".to_string(),
            password: "123".to_string(),
        };
        let res = signin_handler(State(state.clone()), Json(input)).await;
        assert_eq!(res.unwrap().status(), StatusCode::FORBIDDEN);

        // create user
        let input = CreateUser {
            fullname: "test".to_string(),
            email: "b@a.com".to_string(),
            password: "123".to_string(),
        };
        let res = singup_handler(State(state.clone()), Json(input)).await;
        assert_eq!(res.unwrap().status(), StatusCode::CREATED);

        // create exist user
        let input = CreateUser {
            fullname: "test".to_string(),
            email: "b@a.com".to_string(),
            password: "123".to_string(),
        };
        let res = singup_handler(State(state.clone()), Json(input)).await;
        matches!(res, Err(AppError::AlreadyExist(_)));

        // sigin with exist email
        let input = SigninInput {
            email: "b@a.com".to_string(),
            password: "123".to_string(),
        };
        let res = signin_handler(State(state), Json(input)).await;
        assert_eq!(res.unwrap().status(), StatusCode::OK);
    }
}
