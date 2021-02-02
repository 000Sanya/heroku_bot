use act_zero::runtimes::tokio::spawn_actor;
use act_zero::{call, send, Actor, ActorResult, Produces, Addr, upcast};
use std::env;
use std::time::Duration;
use warp::Filter;
use crate::telegram::TelegramSenderActor;
use crate::processor::RequestProcessorActor;
use crate::pixiv::PixivReceiveActor;
use std::ops::Add;
use crate::vk::VkSenderActor;

mod config;
mod pixiv;
mod processor;
mod request;
mod telegram;
mod vk;
mod utils;

fn pixiv_handler( pixiv_receiver: Addr<PixivReceiveActor>, mut body: impl warp::Buf,) -> Result<&'static str, Box<dyn std::error::Error>> {
    if body.has_remaining() {
        let body = String::from_utf8(body.chunk().to_vec())?;
        if body.contains("pixiv.net") {
            let regex = regex::Regex::new(r"https?://(www\.)?pixiv\.net/en/artworks/(?P<id>\d+)")
                .expect("Error on compile regex");
            let id = regex.captures(&body)
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
    env::set_var("RUST_LOG", "heroku_bot=trace,atc_zero=warn,tgbot=trace,reqwest=trace");
    env_logger::init();
    log::trace!("TEST");

    let telegram_target = spawn_actor(TelegramSenderActor::new(
        config.clone()
    ));

    let vk_target = spawn_actor(VkSenderActor::new(
       config.clone()
    ));

    let processor = spawn_actor(RequestProcessorActor::new(vec![
        upcast!(telegram_target),
        upcast!(vk_target),
    ]));

    let pixiv_receiver = spawn_actor(PixivReceiveActor::new(config.clone(), processor.clone()).await);

    let server = warp::post()
        .map(move || pixiv_receiver.clone())
        .and(warp::path("post"))
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::aggregate())
        .map(pixiv_handler)
        .map(|r: Result<&'static str, Box<dyn std::error::Error>>| r.unwrap_or("Error"));

    let port = env::var("PORT").unwrap_or("8080".to_owned()).parse()
        .expect("not number");

    warp::serve(server)
        .run(([0, 0, 0, 0], port)).await
}
