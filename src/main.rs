use axum::{
    extract::{Path, Query, State},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::{env, net::SocketAddr, process::exit, sync::Arc};
use teloxide::{
    prelude::*,
    types::{ParseMode, Recipient},
};
use tokio::signal::unix::SignalKind;

#[derive(Deserialize)]
struct MessagePayload {
    #[serde(rename = "_message")]
    message: String,

    #[serde(flatten)]
    settings: SettingsPayload,
}

#[derive(Deserialize)]
struct SettingsPayload {
    #[serde(default)]
    silent: bool,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting notification bot...");

    let bot = Arc::new(Bot::from_env());
    let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())
        .expect("failed to acquire SIGINT listener");

    tokio::select! {
        _ = run_server(bot.clone()) => {},
        _ = bot_repl(bot) => {},
        _ = interrupt.recv() => {
            println!("Received interrupt, exiting ...");
            exit(0);
        }
    }
}

async fn bot_repl(bot: Arc<Bot>) {
    teloxide::repl(bot, |bot: Arc<Bot>, msg: Message| async move {
        bot.send_message(
            msg.chat.id,
            format!("This bot is for sending custom notifications\\. If you found this randomly, it is probably not for you\\.\n\nYour personal ID is `{}`", msg.chat.id),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        Ok(())
    })
    .await;
}

async fn run_server(bot: Arc<Bot>) {
    let app = Router::new()
        .route("/chat/:id", post(handle_hook))
        .route("/notify", post(handle_default_hook))
        .with_state(bot);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    log::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_hook(
    Path(chat_id): Path<ChatId>,
    State(bot): State<Arc<Bot>>,
    Json(payload): Json<MessagePayload>,
) -> Result<Json<()>, String> {
    send_message(chat_id, bot, payload.message, payload.settings.silent).await
}

async fn handle_default_hook(
    State(bot): State<Arc<Bot>>,
    Query(settings): Query<SettingsPayload>,
    payload: String,
) -> Result<Json<()>, String> {
    let chat_id = ChatId(
        env::var("DEFAULT_CHAT_ID")
            .expect("`DEFAULT_CHAT_ID` env var is required but not set")
            .parse::<i64>()
            .unwrap(),
    );

    send_message(chat_id, bot, payload, settings.silent).await
}

async fn send_message(
    chat_id: ChatId,
    bot: Arc<Bot>,
    payload: String,
    silent: bool,
) -> Result<Json<()>, String> {
    let user = Recipient::Id(chat_id);
    log::debug!("Sending message '{}'", user);
    bot.send_message(user, payload)
        .disable_notification(silent)
        .await
        .map(|_| ().into())
        .map_err(|e| e.to_string())
}
