use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{
        user::{CreateUser, SigninInput},
        User,
    },
    AppState,
};

pub async fn singup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<Response<Body>, AppError> {
    let user = User::create(&state.db, &input).await?;
    let token = state.ek.sign(user.id)?;
    let body = SigninOutput { token };
    Ok((StatusCode::CREATED, Json(body)).into_response())
}

pub async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninInput>,
) -> Result<Response<Body>, AppError> {
    let user = User::signin(&state.db, &input).await?;
    let Some(user) = user else {
        return Ok((StatusCode::FORBIDDEN, "email or password is wrong").into_response());
    };
    let token = state.ek.sign(user.id)?;
    Ok(Json(SigninOutput { token }).into_response())
}

#[derive(Serialize, Deserialize)]
struct SigninOutput {
    token: String,
}
