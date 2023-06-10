use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use teloxide::{
    prelude::*,
    types::{ParseMode, Recipient},
};

#[derive(Deserialize)]
struct MessagePayload {
    #[serde(rename = "_message")]
    message: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting notification bot...");

    let bot = Arc::new(Bot::from_env());
    let server_bot = bot.clone();

    tokio::spawn(async move {
        run_server(server_bot).await;
    });

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_message(
            msg.chat.id,
            format!("This bot is for sending custom notifications. If you found this randomly, it is probably not for you.\n\nYour personal ID is `{}`", msg.chat.id),
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
    let user = Recipient::Id(chat_id);

    log::debug!("Sending message '{}'", user);

    bot.send_message(user, payload.message)
        .await
        .map(|_| ().into())
        .map_err(|e| e.to_string())
}
