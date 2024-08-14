use axum::{
    extract::{Path, State},
    Extension, Json,
};
use chat_core::{utils::UserCliams, Chat, RowID};

use crate::{
    error::AppResult,
    models::{self, chat::CreateChat},
    AppState,
};

pub async fn create_chat_handler(
    Extension(user): Extension<UserCliams>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> AppResult<Json<Chat>> {
    let chat = models::chat::create(&state.db, user.ws_id, input).await?;
    Ok(Json(chat))
}

pub async fn list_chat_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserCliams>,
) -> AppResult<Json<Vec<Chat>>> {
    let chats = models::chat::list(&state.db, user.ws_id, user.uid).await?;
    Ok(Json(chats))
}

pub async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<RowID>,
) -> AppResult<Json<Chat>> {
    let chat = models::chat::get(&state.db, id).await?;
    Ok(Json(chat))
}
