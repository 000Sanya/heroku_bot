use crate::config::Config;
use crate::request::{Image, ImageSender, ImageRequest, ImageRequestBody};
use act_zero::{Actor, Produces, ActorResult, ActorError, Addr};
use std::io::Cursor;
use std::sync::Arc;
use tgbot::methods::{SendMediaGroup, SendMessage, SendPhoto, SendDocument};
use tgbot::types::{InputFile, InputFileInfo, InputFileReader, InputMediaPhoto, MediaGroup, InputMediaDocument};
use crate::utils::ResultExtension;
use tokio::time::Duration;

pub struct TelegramSenderActor {
    bot: tgbot::Api,
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
        let bot = tgbot::Api::new(
            tgbot::Config::new(config.telegram_token.clone())
                .host(config.telegram_host.clone())
        ).expect("Error on build api");

        Self { config, bot }
    }

    #[inline(never)]
    async fn upload_images(&self, album: &[Image], request: &str) -> ActorResult<()> {
        let media = album
            .iter()
            .map(|i| image_as_input_file(i))
            .fold(MediaGroup::default(), |media, file| {
                media.add_item(file, InputMediaPhoto::default())
            });

        let method = SendMediaGroup::new(self.config.telegram_target, media)
            .log_on_error("Error on compile media group")?;

        self.bot.execute(method).await
            .log_on_error("Error on send media group")?;

        log::info!("Sended {} image from {}", album.len(), request);

        Produces::ok(())
    }

    #[inline(never)]
    async fn upload_docs(&self, album: &[Image], request: &str) -> ActorResult<()> {
        let docs = album
            .iter()
            .map(|i| image_as_input_file(i))
            .fold(MediaGroup::default(), |media, file| {
                media.add_item(file, InputMediaDocument::default())
            });
        let method2 = SendMediaGroup::new(self.config.telegram_target, docs)
            .log_on_error("Error on compile docs group")?;

        self.bot.execute(method2).await
            .log_on_error("Error on send docs group")?;


        log::info!("Sended {} docs from {}", album.len(), request);

        Produces::ok(())
    }
}

#[async_trait::async_trait]
impl ImageSender for TelegramSenderActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        log::info!("Handle request from {}", request.source);

        self.bot.execute(SendMessage::new(self.config.telegram_target, &request.source))
            .await?;

        match &request.body {
            ImageRequestBody::SingleImage { image } => {
                let file = image_as_input_file(image);
                let file2 = image_as_input_file(image);
                let method = SendPhoto::new(self.config.telegram_target, file);
                let method2 = SendDocument::new(self.config.telegram_target, file2);
                self.bot.execute(method).await
                    .log_on_error("error on send image")?;
                self.bot.execute(method2).await
                    .log_on_error("error on send docs")?;

                log::info!("Sended one image from {}", request.source);
            }
            ImageRequestBody::Album { images } => {
                for album in images.chunks(10) {
                    self.upload_images(album, request.source.as_str()).await?;

                    self.upload_docs(album, request.source.as_str()).await?;
                }
            }
        }

        Produces::ok(())
    }
}

fn image_as_input_file(image: &Image) -> InputFile {
    InputFile::reader(
        InputFileReader::new(Cursor::new(image.data.clone())).info(image.filename.as_ref()),
    )
}

pub struct TelegramReceiverActor {}
