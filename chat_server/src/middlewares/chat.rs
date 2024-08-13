// Only chat member can send read/write message

use axum::{
    extract::{Path, Request, State},
    response::Response,
    Extension,
};

use crate::{
    error::{AppError, AppResult},
    models::{Chat, RowID},
    utils::UserCliams,
    AppState,
};

// /chat/:id/
pub async fn ensure_chat_member(
    State(state): State<AppState>,
    Extension(user): Extension<UserCliams>,
    Path(chat_id): Path<RowID>,
    req: Request,
    next: axum::middleware::Next,
) -> AppResult<Response> {
    let is_member = Chat::is_member(&state.db, chat_id, user.uid).await?;
    if !is_member {
        return Err(AppError::forbidden(
            "user is not the chat member or chat not exist",
        ));
    }
    Ok(next.run(req).await)
}

#[cfg(test)]
mod test {
    use axum::{body::Body, middleware::from_fn_with_state, routing::get, Router};
    use http::StatusCode;

    use sqlx::PgPool;
    use tower::ServiceExt as _;

    use crate::middlewares::verify_token;

    use super::*;

    async fn index_handler() -> &'static str {
        "Hello, World!"
    }

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../fixtures/test.sql")
    )]
    async fn test_ensure_chat_member(pool: PgPool) {
        let state = AppState::new_for_test(pool);
        let router = Router::new()
            .route("/chat/:id", get(index_handler))
            .layer(from_fn_with_state(state.clone(), ensure_chat_member))
            .layer(from_fn_with_state(state.clone(), verify_token));

        // user 1 can access chat 1
        let token = state
            .ek
            .sign(&UserCliams {
                uid: 1,
                ..Default::default()
            })
            .unwrap();

        let req = Request::builder()
            .uri("/chat/1")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty());

        let resp = router.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let token = state
            .ek
            .sign(&UserCliams {
                uid: 3,
                ..Default::default()
            })
            .unwrap();

        // user 3 cannot access chat 1
        let req = Request::builder()
            .uri("/chat/1")
            .header("Authorization", format!("Bearer {token}"));
        let resp = router
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
