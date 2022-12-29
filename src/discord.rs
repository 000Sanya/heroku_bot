use std::sync::Arc;
use act_zero::{Actor, ActorResult, Produces};
use crate::config::Config;
use crate::request::{ImageRequest, ImageRequestBody, ImageSender};

pub struct DiscordWebhookActor {
    config: Arc<Config>,
    http: serenity::http::Http,
}

impl DiscordWebhookActor {
    pub fn new(config: Arc<Config>) -> Self {
        let http = serenity::http::Http::new("");

        Self { config, http }
    }
}

#[async_trait::async_trait]
impl Actor for DiscordWebhookActor {
    async fn error(&mut self, error: act_zero::ActorError) -> bool {
        log::error!("{}", error);
        false
    }
}

#[async_trait::async_trait]
impl ImageSender for DiscordWebhookActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        let webhook = self.config.discord_webhook.as_ref().unwrap();

        log::info!("Uploading image to Discord from {}", request.source);
        let mut images = Vec::new();
        match &request.body {
            ImageRequestBody::SingleImage { image: i } => images.push(i),
            ImageRequestBody::Album { images: i } => images.extend(i),
        };

        let webhook = self.http.get_webhook_from_url(webhook).await?;

        for image in images {
            let file_name = image.filename.clone();
            let image = image::load_from_memory(image.data.as_ref()).expect("Error on load image");
            let mut buffer = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 90);
            encoder.encode_image(&image)?;

            webhook.execute(&self.http, false, |w| {
                w.add_file((buffer.as_slice(), format!("{file_name}.jpg").as_str()))
            })
                .await?;
        }

        log::info!("Uploaded image to Discord from {}", request.source);
        Produces::ok(())
    }
}
