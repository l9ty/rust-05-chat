use std::{io, path::Path as StdPath};

use anyhow::Context;
use axum::{
    extract::{Multipart, Path, Query, State},
    Extension, Json,
};
use chat_core::{utils::UserCliams, Message, RowID};
use tokio::fs;

use crate::{
    error::{AppError, AppResult},
    models::{
        self,
        file::ChatFile,
        message::{self, CreateMessage, ListMessage},
    },
    AppState,
};

pub async fn list_message_handler(
    State(state): State<AppState>,
    Query(input): Query<ListMessage>,
    Path(id): Path<RowID>,
) -> AppResult<Json<Vec<Message>>> {
    let messages = models::message::list(&state.db, id, input).await?;
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
    let msg = message::create(&state.db, input, user.uid, base).await?;
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
        if filename.is_empty() || filename.contains('/') {
            continue;
        }
        let filename = filename.to_owned();
        let data = field.bytes().await?;
        let chat_file = ChatFile::new(user.ws_id, &filename, &data);
        let path = chat_file.path(base);

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap_or(base))
                .await
                .with_context(|| format!("create dir: {:?}", &path))?;
            fs::write(&path, data)
                .await
                .with_context(|| format!("write file: {:?}", &path))?;
            files.push(chat_file.url());
        }
    }

    Ok(Json(files))
}

pub async fn download_file_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserCliams>,
    Path(file_url): Path<String>,
) -> AppResult<Vec<u8>> {
    let base = StdPath::new(&state.config.base_dir);
    let file = ChatFile::from_url(&file_url)?;
    if file.ws_id != user.ws_id {
        return Err(AppError::not_found("file not found"));
    }
    let fs_path = file.path(base);
    let data = fs::read(&fs_path).await.map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => AppError::not_found("file not found"),
        _ => AppError::any(e),
    })?;
    Ok(data)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(
        migrator = "crate::tests::MIGRATOR",
        fixtures("../../../fixtures/test.sql")
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
        fixtures("../../../fixtures/test.sql")
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
