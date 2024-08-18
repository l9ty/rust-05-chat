use core::panic;

use chat_core::{Chat, ChatType, Message};
use chat_server::AppState;
use futures::StreamExt;
use notify_server::{NotifyConfig, NotifyState};
use reqwest_eventsource::{Event, EventSource};
use serde_json::json;
use tokio::net::TcpListener;

const PUBLIC_KEY: &str = include_str!("../../fixtures/public.pem");
const CHAT_SERVER_PORT: u16 = 8080;
const NOTIFY_SERVER_PORT: u16 = 8081;

struct ChatServer {
    token: String,
    http_client: reqwest::Client,
}

#[allow(unused)]
struct NotifyServer;

#[sqlx::test(
    migrator = "chat_server::tests::MIGRATOR",
    fixtures("../../fixtures/test.sql")
)]
async fn t_chat_server(pool: sqlx::PgPool) {
    let state = chat_server::AppState::new_for_test(pool);
    let mut chat = ChatServer::try_new(state).await.unwrap();
    chat.signin().await.unwrap();

    let notify_config = NotifyServer::new_config();
    let notify_state = NotifyState::try_new(notify_config).unwrap();
    let _notify = NotifyServer::try_new(notify_state, &chat.token)
        .await
        .unwrap();

    chat.create_chat().await;
    chat.create_message().await;
}

impl ChatServer {
    async fn try_new(state: AppState) -> anyhow::Result<Self> {
        let router = chat_server::get_router(state)
            .await
            .expect("get router failed");

        let addr = format!("0.0.0.0:{}", CHAT_SERVER_PORT);
        let ls = TcpListener::bind(addr)
            .await
            .expect("bind tcp listener failed");

        let server = axum::serve(ls, router);
        tokio::spawn(async move { server.await.expect("chat server exist with error") });

        let http_client = reqwest::Client::new();

        let chat = Self {
            token: Default::default(),
            http_client,
        };

        Ok(chat)
    }

    async fn signin(&mut self) -> anyhow::Result<()> {
        let url = format!("http://localhost:{CHAT_SERVER_PORT}/signin");
        let body = json!(
            {
                "email": "user-1@a.com",
                "password": "123456"
            }
        );
        let resp = self.http_client.post(url).json(&body).send().await?;
        assert!(resp.status().is_success());
        let body = resp.json::<serde_json::Value>().await.unwrap();
        let token = body
            .as_object()
            .unwrap()
            .get("token")
            .unwrap()
            .as_str()
            .unwrap();
        self.token = token.to_string();
        Ok(())
    }

    async fn create_chat(&self) {
        let url = format!("http://localhost:{CHAT_SERVER_PORT}/api/chat");
        let body = json!(
            {
                "name": "ig-chat",
                "members": [1, 2, 3],
                "public": true,
            }
        );
        let resp = self
            .http_client
            .post(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
    }

    async fn create_message(&self) {
        let url = format!("http://localhost:{CHAT_SERVER_PORT}/api/chat/1/message");
        let body = json!(
            {
                "content": "hello"
            }
        );
        let resp = self
            .http_client
            .put(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
    }
}

impl NotifyServer {
    async fn try_new(state: NotifyState, token: &str) -> anyhow::Result<Self> {
        let router = notify_server::get_router(state)
            .await
            .expect("get router failed");

        let addr = format!("0.0.0.0:{NOTIFY_SERVER_PORT}");
        let ls = TcpListener::bind(addr)
            .await
            .expect("bind tcp listener failed");

        let server = axum::serve(ls, router);
        tokio::spawn(async move { server.await.expect("notify server exist with error") });

        let token = token.to_string();
        tokio::spawn(async move {
            let mut es = EventSource::get(format!(
                "http://localhost:{NOTIFY_SERVER_PORT}/events?access_token={token}"
            ));

            let client = reqwest::Client::new();

            let resp = client
                .get(format!(
                    "http://localhost:{NOTIFY_SERVER_PORT}/events?access_token={token}"
                ))
                .send()
                .await
                .unwrap();
            assert!(resp.status().is_success());

            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("event source connection open!"),
                    Ok(Event::Message(msg)) => match msg.event.as_str() {
                        "NewChat" => {
                            let chat = serde_json::from_str::<Chat>(&msg.data).unwrap();
                            assert_eq!(chat.name.as_ref().unwrap(), "ig-test");
                            assert_eq!(chat.members, vec![1, 2, 3]);
                            matches!(chat.typ, ChatType::PublicChannel);
                        }
                        "NewMessage" => {
                            let message = serde_json::from_str::<Message>(&msg.data).unwrap();
                            assert_eq!(message.content.as_str(), "hello");
                        }
                        _ => {
                            panic!("unknown event: {}", msg.event.as_str());
                        }
                    },
                    Err(e) => {
                        es.close();
                        println!("event source error: {}", e);
                    }
                }
            }
        });

        Ok(Self)
    }

    fn new_config() -> NotifyConfig {
        NotifyConfig {
            server: notify_server::config::ServerConfig {
                db_url: Default::default(),
                host: "0.0.0.0".to_string(),
                port: NOTIFY_SERVER_PORT,
            },
            auth: notify_server::config::AuthConfig {
                pk: PUBLIC_KEY.to_string(),
            },
        }
    }
}
