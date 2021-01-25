use crate::config::Config;
use crate::request::{Image, ImageSender, ImageRequest, ImageRequestBody};
use act_zero::{Actor, Produces, ActorResult};
use std::io::Cursor;
use std::sync::Arc;
use tgbot::methods::{SendMediaGroup, SendMessage, SendPhoto, SendDocument};
use tgbot::types::{InputFile, InputFileInfo, InputFileReader, InputMediaPhoto, MediaGroup, InputMediaDocument};
use crate::utils::ResultExtension;

pub struct TelegramSenderActor {
    bot: tgbot::Api,
    config: Arc<Config>,
}

impl Actor for TelegramSenderActor {}

impl TelegramSenderActor {
    pub fn new(config: Arc<Config>) -> Self {
        let bot = tgbot::Api::new(
            tgbot::Config::new(config.telegram_token.clone())
                .host(config.telegram_host.clone())
        ).expect("Error on build api");

        Self { config, bot }
    }
}

#[async_trait::async_trait]
impl ImageSender for TelegramSenderActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        log::info!("Handle request from {}", request.source);
        match &request.body {
            ImageRequestBody::SingleImage { image } => {
                let file = image_as_input_file(image);
                let file2 = image_as_input_file(image);
                let method = SendPhoto::new(self.config.telegram_target, file);
                let method2 = SendDocument::new(self.config.telegram_target, file2);
                self.bot.execute(method).await
                    .log_on_error("error on execute load")?;
                self.bot.execute(method2).await
                    .log_on_error("error on execute load")?;

                log::info!("Sended one image from {}", request.source);
            }
            ImageRequestBody::Album { images } => {
                for album in images.chunks(10) {
                    let media = album
                        .iter()
                        .map(|i| image_as_input_file(i))
                        .fold(MediaGroup::default(), |media, file| {
                            media.add_item(file, InputMediaPhoto::default())
                        });

                    let method = SendMediaGroup::new(self.config.telegram_target, media)?;

                    self.bot.execute(method).await?;

                    log::info!("Sended {} image from {}", album.len(), request.source);
                }

                for album in images.chunks(10) {
                    let docs = album
                        .iter()
                        .map(|i| image_as_input_file(i))
                        .fold(MediaGroup::default(), |media, file| {
                            media.add_item(file, InputMediaDocument::default())
                        });
                    let method2 = SendMediaGroup::new(self.config.telegram_target, docs)?;

                    self.bot.execute(method2).await?;

                    log::info!("Sended {} docs from {}", album.len(), request.source);
                }
            }
        }

        self.bot.execute(SendMessage::new(self.config.telegram_target, &request.source))
            .await?;

        Produces::ok(())
    }
}

fn image_as_input_file(image: &Image) -> InputFile {
    InputFile::reader(
        InputFileReader::new(Cursor::new(image.data.clone())).info(image.filename.as_ref()),
    )
}

pub struct TelegramReceiverActor {}
