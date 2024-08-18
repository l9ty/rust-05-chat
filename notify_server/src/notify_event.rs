use std::collections::HashSet;

use anyhow::{anyhow, bail, Context};
use chat_core::{Chat, Message, RowID};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgListener, PgNotification};
use tracing::{error, info, warn};

use crate::NotifyState;

#[derive(Debug, Clone, Serialize)]
pub enum NotifyEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

pub struct AppNotification {
    users: HashSet<RowID>,
    event: NotifyEvent,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]

struct ChatUpdatedNotification {
    // delete/update/insert
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

#[derive(Debug, Deserialize)]
struct NewMessageNotification {
    message: Message,
    members: Vec<RowID>,
}

pub async fn setup_pg_listener(state: NotifyState) -> anyhow::Result<()> {
    let mut ls = PgListener::connect(&state.config.server.db_url)
        .await
        .context("failed to create pg listener")?;
    ls.listen("chat_updated")
        .await
        .context("listen chat_updated")?;
    ls.listen("chat_message_created")
        .await
        .context("listen chat_message_created")?;

    let mut stream = ls.into_stream();

    tokio::spawn(async move {
        while let Some(nf) = stream.next().await {
            let nf = match nf {
                Err(e) => {
                    error!("failed to receive pg notification: {}", e);
                    break;
                }
                Ok(nf) => nf,
            };
            info!("Receive notification: {:?}", nf);

            let nf: AppNotification = match nf.try_into() {
                Ok(nf) => nf,
                Err(e) => {
                    error!("failed to parse pg notification: {:#}", e);
                    continue;
                }
            };

            nf.users
                .iter()
                .filter_map(|uid| state.users.get(uid))
                .for_each(|kv| {
                    if let Err(e) = kv.send(nf.event.clone()) {
                        error!("failed to send notification to {}: {}", kv.key(), e);
                    }
                });
        }

        warn!("pg listener exit");
    });

    Ok(())
}

impl TryFrom<PgNotification> for AppNotification {
    type Error = anyhow::Error;

    fn try_from(nf: PgNotification) -> Result<Self, Self::Error> {
        let payload = nf.payload();
        let channel = nf.channel();
        match channel {
            "chat_updated" => {
                let payload = serde_json::from_str::<ChatUpdatedNotification>(payload)
                    .with_context(|| format!("invalid chat_updated payload: {}", payload))?;
                payload.try_into()
            }
            "chat_message_created" => {
                let payload = serde_json::from_str::<NewMessageNotification>(payload)
                    .with_context(|| {
                        format!("invalid chat_message_created payload: {}", payload)
                    })?;
                payload.try_into()
            }
            _ => Err(anyhow!("invalid pg notification channel: {}", channel)),
        }
    }
}

impl TryFrom<NewMessageNotification> for AppNotification {
    type Error = anyhow::Error;
    fn try_from(value: NewMessageNotification) -> Result<Self, Self::Error> {
        let users = value.members.iter().copied().collect();
        Ok(AppNotification {
            users,
            event: NotifyEvent::NewMessage(value.message),
        })
    }
}

impl TryFrom<ChatUpdatedNotification> for AppNotification {
    type Error = anyhow::Error;

    fn try_from(value: ChatUpdatedNotification) -> Result<Self, Self::Error> {
        match (value.old, value.new) {
            // update
            (Some(old), Some(new)) => {
                let old_users: HashSet<_> = old.members.iter().copied().collect();
                let new_users: HashSet<_> = new.members.iter().copied().collect();
                if old_users == new_users {
                    Ok(AppNotification {
                        users: HashSet::default(),
                        event: NotifyEvent::AddToChat(new),
                    })
                } else {
                    let users = old_users.union(&new_users).copied().collect();

                    Ok(AppNotification {
                        users,
                        event: NotifyEvent::AddToChat(new),
                    })
                }
            }
            // insert
            (None, Some(new)) => {
                let users = new.members.iter().copied().collect();
                Ok(AppNotification {
                    users,
                    event: NotifyEvent::NewChat(new),
                })
            }
            (Some(old), None) => {
                let users = old.members.iter().copied().collect();
                Ok(AppNotification {
                    users,
                    event: NotifyEvent::RemoveFromChat(old),
                })
            }
            // no change
            _ => bail!("chat_updated: (None, None)"),
        }
    }
}
