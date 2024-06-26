use crate::config::Config;
use crate::request::{Image, ImageRequest, ImageRequestBody, ImageSender};
use crate::utils::ResultExtension;
use act_zero::{Actor, ActorError, ActorResult, Addr, Produces};
use std::sync::Arc;
use tap::Pipe;
use teloxide_core::adaptors::Throttle;
use teloxide_core::prelude::{Request, Requester};
use teloxide_core::prelude::RequesterExt;

pub struct TelegramSenderActor {
    bot: Throttle<teloxide_core::Bot>,
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
            .build()
            .expect("Error on build client");

        let bot = teloxide_core::Bot::with_client(&config.telegram_token, client)
            .pipe(|bot| match config.telegram_host.as_ref() {
                None => bot,
                Some(host) => bot.set_api_url(reqwest::Url::parse(host).expect("WTF"))
            })
            .throttle(Default::default());


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

        log::info!("Sent {} image from {}", album.len(), request);

        Produces::ok(())
    }
}

#[async_trait::async_trait]
impl ImageSender for TelegramSenderActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        log::info!("Handle request from {}", request.source);

        let _ = self
            .bot
            .send_message(self.config.telegram_target, &request.source)
            .send()
            .await
            .on_error(|_| log::error!("Error on send message"));

        match &request.body {
            ImageRequestBody::SingleImage { image } => {
                let file = image_as_teloxide_doc_file(image);
                let image_file = image_as_teloxide_file(image);

                let _ = self
                    .bot
                    .send_photo(self.config.telegram_target, image_file)
                    .send()
                    .await
                    .on_error(|_| log::error!("Error on upload as image"));

                let _ = self
                    .bot
                    .send_document(self.config.telegram_target, file)
                    .send()
                    .await
                    .on_error(|_| log::error!("Error on upload as document"));

                log::info!("Sent one image from {}", request.source);
            }
            ImageRequestBody::Album { images } => {
                for album in images.chunks(10) {
                    let _ = self
                        .upload_images(album, request.source.as_str())
                        .await
                        .on_error(|_| log::error!("Error on upload as image"));

                    for image in album {
                        let file = image_as_teloxide_doc_file(image);

                        let _ = self
                            .bot
                            .send_document(self.config.telegram_target, file)
                            .send()
                            .await
                            .on_error(|_| log::error!("Error on upload as document"));
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
    encoder.encode_image(&image).expect("Error on encode image");

    teloxide_core::types::InputFile::memory(
        buffer
    )
        .file_name(file_name)
}

fn image_as_teloxide_doc_file(image: &Image) -> teloxide_core::types::InputFile {
    teloxide_core::types::InputFile::memory(
        image.data.to_vec()
    )
        .file_name(image.filename.clone())
}
