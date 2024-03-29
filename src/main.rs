use crate::pixiv::PixivReceiveActor;
use crate::processor::RequestProcessorActor;
use crate::telegram::TelegramSenderActor;
use crate::vk::VkSenderActor;
use act_zero::runtimes::tokio::spawn_actor;
use act_zero::{send, upcast, Addr};
use std::env;

use warp::Filter;
use crate::discord::DiscordWebhookActor;

mod config;
mod pixiv;
mod pixiv_api;
mod processor;
mod request;
mod telegram;
mod utils;
mod vk;
mod discord;

fn pixiv_handler(
    pixiv_receiver: Addr<PixivReceiveActor>,
    body: impl warp::Buf,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    if body.has_remaining() {
        let body = String::from_utf8(body.chunk().to_vec())?;
        if body.contains("pixiv.net") {
            let regex = regex::Regex::new(r"https?://(www\.)?pixiv\.net/en/artworks/(?P<id>\d+)")
                .expect("Error on compile regex");
            let id = regex
                .captures(&body)
                .and_then(|c| c.name("id"))
                .and_then(|m| m.as_str().parse::<i64>().ok());

            if let Some(id) = id {
                send!(pixiv_receiver.receive_illust(id));
            }
        }
    }

    Ok("Ok")
}

#[tokio::main]
async fn main() {
    let config = config::get_config();
    env::set_var("RUST_LOG", "heroku_bot=trace,atc_zero=warn");
    env_logger::init();
    log::trace!("TEST");

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

    let server = warp::post()
        .map(move || pixiv_receiver.clone())
        .and(warp::path("post"))
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::aggregate())
        .map(pixiv_handler)
        .map(|r: Result<&'static str, Box<dyn std::error::Error>>| r.unwrap_or("Error"));

    let port = env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse()
        .expect("not number");

    warp::serve(server).run(([0, 0, 0, 0], port)).await
}
