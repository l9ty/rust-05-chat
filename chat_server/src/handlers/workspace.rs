use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};

use crate::{error::AppError, models::Workspace, utils::UserCliams, AppState};

pub async fn list_users(
    Extension(user): Extension<UserCliams>,
    State(state): State<AppState>,
) -> Result<Response<Body>, AppError> {
    let users = Workspace::list_all_users(&state.db, user.ws_id).await?;
    Ok(Json(users).into_response())
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt as _;

    use super::*;
    use crate::{models::User, tests::MIGRATOR};

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn list_users_should_work(pool: sqlx::PgPool) {
        // TODO insert more users
        let state = AppState::new_for_test(pool);
        let res = list_users(Extension(Default::default()), State(state)).await;
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.status().is_success());
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let users = serde_json::from_slice::<Vec<User>>(&body).unwrap();
        assert_eq!(users.len(), 1);
    }
}
