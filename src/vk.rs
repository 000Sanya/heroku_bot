use crate::config::Config;
use crate::request::{Image, ImageRequest, ImageRequestBody, ImageSender};
use crate::utils::ResultExtension;
use act_zero::{Actor, ActorResult, Produces};
use itertools::Itertools;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct VkSenderActor {
    config: Arc<Config>,
    api: rvk::APIClient,
}

impl VkSenderActor {
    pub fn new(config: Arc<Config>) -> Self {
        let api = rvk_methods::supported_api_client(config.vk_bot_token.clone());

        Self { config, api }
    }
}

#[async_trait::async_trait]
impl Actor for VkSenderActor {
    async fn error(&mut self, error: act_zero::ActorError) -> bool {
        log::error!("{}", error);
        false
    }
}

#[async_trait::async_trait]
impl ImageSender for VkSenderActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        log::info!("Start handle {} with vk target", request.source);

        let mut images = vec![];
        match &request.body {
            ImageRequestBody::SingleImage { image } => images.push(image),
            ImageRequestBody::Album { images: i } => images.extend(i.iter()),
        };

        let url = get_upload_server(&self.api, self.config.vk_target).await?;
        let images: Vec<_> =
            futures::future::join_all(images.into_iter().map(|i| upload_photo(&self.api, &url, i)))
                .await
                .into_iter()
                .filter_map(|i| i.log_on_error("Error on photo upload").ok())
                .collect();

        for imgs in images.chunks(10) {
            let attachment = imgs
                .iter()
                .map(|img| format!("photo{}_{}", img[0].owner_id, img[0].id))
                .join(",");

            let params = maplit::hashmap! {
                "peer_id".into() => self.config.vk_target.to_string(),
                "attachment".into() => attachment,
                "random_id".into() => rand::thread_rng().next_u64().to_string()
            };

            rvk_methods::messages::send::<serde_json::Value>(&self.api, params).await?;
        }

        Produces::ok(())
    }
}

async fn get_upload_server(client: &rvk::APIClient, peer_id: i64) -> anyhow::Result<String> {
    let params = maplit::hashmap! {
        "peer_id".into() => peer_id.to_string()
    };

    let result =
        rvk_methods::photos::get_messages_upload_server::<GetUploadServer>(client, params).await?;

    Ok(result.upload_url)
}

async fn upload_photo(
    api: &rvk::APIClient,
    upload_url: &str,
    image: &Image,
) -> anyhow::Result<Vec<Photo>> {
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(image.data.as_ref().to_owned())
            .file_name(image.filename.clone()),
    );
    let result: UploadResult = client
        .post(upload_url)
        .multipart(form)
        .send()
        .await
        .log_on_error("Erron on upload photo")?
        .json()
        .await?;

    Ok(
        rvk_methods::photos::save_messages_photo::<serde_json::Value>(
            api,
            maplit::hashmap! {
                "hash".into() => result.hash,
                "server".into() => result.server.to_string(),
                "photo".into() => result.photo,
            },
        )
        .await
        .and_then(|photo| Ok(serde_json::from_value::<Vec<Photo>>(photo)?))
        .log_on_error("Error on save photo")?,
    )
}

#[derive(Deserialize, Debug)]
struct GetUploadServer {
    upload_url: String,
    album_id: i64,
    user_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadResult {
    server: i64,
    photo: String,
    hash: String,
}

#[derive(Deserialize, Debug)]
struct Photo {
    id: i64,
    owner_id: i64,
}
