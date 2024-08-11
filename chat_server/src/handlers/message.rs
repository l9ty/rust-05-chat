use std::path::Path as StdPath;

use anyhow::Context;
use axum::{
    extract::{Multipart, Path, Query, State},
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;

use crate::{
    error::{AppError, AppResult},
    models::{
        message::{CreateMessage, ListMessage},
        Message, RowID,
    },
    utils::UserCliams,
    AppState,
};

pub async fn list_message_handler(
    State(state): State<AppState>,
    Query(input): Query<ListMessage>,
    Path(id): Path<RowID>,
) -> AppResult<Json<Vec<Message>>> {
    let messages = Message::list(&state.db, id, input).await?;
    Ok(Json(messages))
}

pub async fn send_message_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<RowID>,
    Extension(user): Extension<UserCliams>,
    Json(mut input): Json<CreateMessage>,
) -> AppResult<Json<Message>> {
    let base = StdPath::new(&state.config.base_dir);
    input.chat_id = chat_id;
    let msg = Message::create(&state.db, input, user.uid, base).await?;
    Ok(Json(msg))
}

pub async fn upload_file_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserCliams>,
    mut multipart: Multipart,
) -> AppResult<Json<Vec<String>>> {
    let base = StdPath::new(&state.config.base_dir);
    let mut files = Vec::with_capacity(5);
    while let Some(field) = multipart.next_field().await? {
        let Some(filename) = field.file_name() else {
            continue;
        };
        let filename = filename.to_string();
        let data = field.bytes().await?;

        if filename.is_empty() || filename.contains('/') {
            continue;
        }

        let path = base.join(user.ws_id.to_string()).join(&filename);
        if !path.exists() {
            let Some(dir) = path.parent() else {
                continue;
            };
            fs::create_dir_all(dir)
                .await
                .with_context(|| format!("create dir: {:?}", dir))?;
            fs::write(&path, data)
                .await
                .with_context(|| format!("write file: {:?}", &path))?;

            files.push(filename);
        }
    }

    Ok(Json(files))
}

pub async fn download_file_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserCliams>,
    Path(file): Path<String>,
) -> AppResult<impl IntoResponse> {
    let base = StdPath::new(&state.config.base_dir);
    let path = base.join(user.ws_id.to_string()).join(&file);
    if !path.exists() {
        return Err(AppError::NotFound(format!("file not found: {}", &file)));
    }

    let data = fs::read(&path)
        .await
        .with_context(|| format!("read file: {:?}", &path))?;

    Ok(data)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../fixtures/test.sql")
    )]
    async fn t_list_message(pool: PgPool) {
        let state = AppState::new_for_test(pool);
        let res = list_message_handler(
            State(state),
            Query(ListMessage {
                last_id: None,
                limit: 10,
            }),
            Path(1),
        )
        .await
        .unwrap();

        assert_eq!(res.0.len(), 2);
    }

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../fixtures/test.sql")
    )]
    async fn t_send_message(pool: PgPool) {
        let state = AppState::new_for_test(pool);
        let res = send_message_handler(
            State(state),
            Path(1),
            Extension(Default::default()),
            Json(CreateMessage {
                content: "test".to_string(),
                chat_id: 1,
                files: vec![],
            }),
        )
        .await;
        assert!(res.is_ok());
    }
}
