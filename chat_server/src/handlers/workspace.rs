use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chat_core::utils::UserCliams;

use crate::{error::AppError, models, AppState};

pub async fn list_ws_users_handler(
    Extension(user): Extension<UserCliams>,
    State(state): State<AppState>,
) -> Result<Response<Body>, AppError> {
    let users = models::workspace::list_all_users(&state.db, user.ws_id).await?;
    Ok(Json(users).into_response())
}

#[cfg(test)]
mod tests {
    use chat_core::User;
    use http_body_util::BodyExt as _;

    use super::*;

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../../fixtures/test.sql")
    )]
    async fn t_list_ws_users(pool: sqlx::PgPool) {
        let state = AppState::new_for_test(pool);
        let res = list_ws_users_handler(Extension(Default::default()), State(state))
            .await
            .unwrap();
        assert!(res.status().is_success());
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let users = serde_json::from_slice::<Vec<User>>(&body).unwrap();
        assert_eq!(users.len(), 2);
    }
}
