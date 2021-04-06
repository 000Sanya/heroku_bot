use crate::config::Config;
use crate::request::{Image, ImageRequest, ImageRequestBody, ImageSender};
use act_zero::{Actor, ActorError, ActorResult, Addr, Produces};
use std::borrow::Cow;
use std::sync::Arc;
use teloxide_core::prelude::{Request, Requester};
use crate::utils::ResultExtension;

pub struct TelegramSenderActor {
    bot: teloxide_core::Bot,
    config: Arc<Config>,
}

#[async_trait::async_trait]
impl Actor for TelegramSenderActor {
    async fn started(&mut self, _addr: Addr<Self>) -> ActorResult<()>
    where
        Self: Sized,
    {
        log::info!("Start Telegram actor");
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        log::error!("{}", error);
        false
    }
}

impl TelegramSenderActor {
    pub fn new(config: Arc<Config>) -> Self {
        let client = reqwest::Client::builder()
            .connection_verbose(true)
            .build()
            .expect("Erron on build client");

        let bot = teloxide_core::Bot::with_client(&config.telegram_token, client)
            .set_api_url(reqwest::Url::parse(config.telegram_host.as_str()).expect("WTF"));

        Self { config, bot }
    }

    async fn upload_images(&self, album: &[Image], request: &str) -> ActorResult<()> {
        let media: Vec<_> = album
            .iter()
            .map(image_as_teloxide_file)
            .map(|file| teloxide_core::types::InputMediaPhoto::new(file))
            .map(|photo| teloxide_core::types::InputMedia::Photo(photo))
            .collect();

        self.bot
            .send_media_group(self.config.telegram_target, media)
            .send()
            .await?;

        log::info!("Sended {} image from {}", album.len(), request);

        Produces::ok(())
    }
}

#[async_trait::async_trait]
impl ImageSender for TelegramSenderActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        log::info!("Handle request from {}", request.source);

        self.bot
            .send_message(self.config.telegram_target, &request.source)
            .send()
            .await
            .log_on_error("Error on send message");

        match &request.body {
            ImageRequestBody::SingleImage { image } => {
                let file = image_as_teloxide_doc_file(image);
                let image_file = image_as_teloxide_file(image);

                self.bot
                    .send_photo(self.config.telegram_target, image_file)
                    .send()
                    .await
                    .log_on_error("Error on upload as image");

                self.bot
                    .send_document(self.config.telegram_target, file)
                    .send()
                    .await
                    .log_on_error("Error on upload as document");

                log::info!("Sended one image from {}", request.source);
            }
            ImageRequestBody::Album { images } => {
                for album in images.chunks(10) {

                    self.upload_images(album, request.source.as_str()).await
                        .log_on_error("Error on upload as image");

                    for image in album {
                        let file = image_as_teloxide_doc_file(image);

                        self.bot
                            .send_document(self.config.telegram_target, file)
                            .send()
                            .await
                            .log_on_error("Error on upload as document");
                    }
                }
            }
        }

        Produces::ok(())
    }
}

fn image_as_teloxide_file(image: &Image) -> teloxide_core::types::InputFile {
    let file_name = image.filename.clone();
    let image = image::load_from_memory(image.data.as_ref()).expect("Error on load image");
    let mut buffer = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 90);
    encoder.encode_image(&image)
        .expect("Error on encode image");

    teloxide_core::types::InputFile::Memory {
        file_name,
        data: Cow::from(buffer),
    }
}

fn image_as_teloxide_doc_file(image: &Image) -> teloxide_core::types::InputFile {
    teloxide_core::types::InputFile::Memory {
        file_name: image.filename.clone(),
        data: Cow::from(image.data.to_vec()),
    }
}
