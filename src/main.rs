use crate::pixiv::PixivReceiveActor;
use crate::processor::RequestProcessorActor;
use crate::telegram::TelegramSenderActor;
use crate::vk::VkSenderActor;
use act_zero::runtimes::tokio::spawn_actor;
use act_zero::{send, upcast, Addr};
use std::env;
use std::sync::Arc;
use axum::extract::State;
use axum::Router;
use axum::routing::post;

use crate::discord::DiscordWebhookActor;
use crate::gelbooru::GelbooruReceiveActor;

mod config;
mod pixiv;
mod pixiv_api;
mod processor;
mod request;
mod telegram;
mod utils;
mod vk;
mod discord;
mod gelbooru;

async fn pixiv_handler(
    State(app_state): State<Arc<AppState>>,
    body: String,
) -> &'static str {
    if body.contains("pixiv.net") {
        let regex = regex::Regex::new(r"https?://(www\.)?pixiv\.net/en/artworks/(?P<id>\d+)")
            .expect("Error on compile regex");
        let id = regex
            .captures(&body)
            .and_then(|c| c.name("id"))
            .and_then(|m| m.as_str().parse::<i64>().ok());

        if let Some(id) = id {
            send!(app_state.pixiv_receiver.receive_illust(id));
        }
    }

    if body.contains("gelbooru.com") {
        let regex = regex::Regex::new(r"id=(?P<id>\d+)")
            .expect("Error on compile regex");
        let id = regex
            .captures(&body)
            .and_then(|c| c.name("id"))
            .and_then(|m| m.as_str().parse::<u64>().ok());
        if let Some(id) = id {
            send!(app_state.gelbooru_receiver.receive_id(id, body));
        }
    }

    "Ok"
}

pub struct AppState {
    pixiv_receiver: Addr<PixivReceiveActor>,
    gelbooru_receiver: Addr<GelbooruReceiveActor>,
}

#[tokio::main]
async fn main() {
    let config = config::get_config();
    env::set_var("RUST_LOG", "heroku_bot=trace,atc_zero=warn");
    env_logger::init();

    log::info!("start bot");

    let mut targets = vec![];

    let telegram_target = spawn_actor(TelegramSenderActor::new(config.clone()));
    targets.push(upcast!(telegram_target));

    let vk_target = spawn_actor(VkSenderActor::new(config.clone()));
    targets.push(upcast!(vk_target));

    if config.discord_webhook.is_some() {
        let discord_target = spawn_actor(DiscordWebhookActor::new(config.clone()));
        targets.push(upcast!(discord_target));
    }

    let processor = spawn_actor(RequestProcessorActor::new(targets));

    let pixiv_receiver =
        spawn_actor(PixivReceiveActor::new(config.clone(), processor.clone()).await);

    let gelbooru_receiver = spawn_actor(GelbooruReceiveActor::new(config.clone(), processor.clone()));

    let port = env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse()
        .expect("not number");

    let app = Router::new()
        .route("/post", post(pixiv_handler))
        .with_state(Arc::new(AppState { pixiv_receiver, gelbooru_receiver }));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
