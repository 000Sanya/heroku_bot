use crate::config::Config;
use crate::request::{Image, ImageRequest, ImageRequestBody, ImageSender};
use crate::utils::ResultExtension;
use act_zero::{Actor, ActorError, ActorResult, Addr, Produces};
use std::borrow::Cow;
use std::io::Cursor;
use std::sync::Arc;
use teloxide_core::prelude::{Request, Requester};
use teloxide_core::types::{MediaKind, MessageCommon, MessageKind};
use tokio::time::Duration;
use std::io::{self, Write};
use tempfile::NamedTempFile;

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

    #[inline(never)]
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

    #[inline(never)]
    async fn upload_images_as_file(&self, album: &[NamedTempFile], request: &str) -> ActorResult<()> {
        let media: Vec<_> = album
            .iter()
            .map(|tempfile| {
                let path = tempfile.path().to_owned();
                teloxide_core::types::InputFile::file(path)
            })
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

    #[inline(never)]
    async fn upload_docs(&self, album: &[Image], request: &str) -> ActorResult<Vec<String>> {
        let media: Vec<_> = album
            .iter()
            .map(image_as_teloxide_file_doc)
            .map(teloxide_core::types::InputMediaDocument::new)
            .map(|doc| teloxide_core::types::InputMedia::Document(doc))
            .collect();

        let file_ids: Vec<_> = self
            .bot
            .send_media_group(self.config.telegram_target, media)
            .send()
            .await?
            .into_iter()
            .filter_map(|mes| match mes.kind {
                MessageKind::Common(data) => match data.media_kind {
                    MediaKind::Document(doc) => Some(doc.document.file_id),
                    _ => None,
                },
                _ => None,
            })
            .collect();

        log::info!("Sended {} docs from {}", album.len(), request);

        Produces::ok(file_ids)
    }

    #[inline(never)]
    async fn upload_docs_as_file(&self, album: &[NamedTempFile], request: &str) -> ActorResult<Vec<String>> {
        let media: Vec<_> = album
            .iter()
            .map(|tempfile| {
                let path = tempfile.path().to_owned();
                teloxide_core::types::InputFile::file(path)
            })
            .map(teloxide_core::types::InputMediaDocument::new)
            .map(|doc| teloxide_core::types::InputMedia::Document(doc))
            .collect();

        let file_ids: Vec<_> = self
            .bot
            .send_media_group(self.config.telegram_target, media)
            .send()
            .await?
            .into_iter()
            .filter_map(|mes| match mes.kind {
                MessageKind::Common(data) => match data.media_kind {
                    MediaKind::Document(doc) => Some(doc.document.file_id),
                    _ => None,
                },
                _ => None,
            })
            .collect();

        log::info!("Sended {} docs from {}", album.len(), request);

        Produces::ok(file_ids)
    }
}

#[async_trait::async_trait]
impl ImageSender for TelegramSenderActor {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()> {
        log::info!("Handle request from {}", request.source);

        self.bot
            .send_message(self.config.telegram_target, &request.source)
            .send()
            .await?;

        match &request.body {
            ImageRequestBody::SingleImage { image } => {
                let mut tempfile = tempfile::NamedTempFile::new()?;
                tempfile.write_all(image.data.as_ref())?;
                let path = tempfile.path().to_owned();
                let input_file = teloxide_core::types::InputFile::file(path);

                self.bot
                    .send_photo(self.config.telegram_target, input_file.clone())
                    .send()
                    .await?;

                self.bot
                    .send_document(self.config.telegram_target, input_file)
                    .send()
                    .await?;

                log::info!("Sended one image from {}", request.source);
            }
            ImageRequestBody::Album { images } => {
                let files: Vec<_> = images
                    .iter()
                    .map(|image| {
                        let mut tempfile = tempfile::NamedTempFile::new().unwrap();
                        tempfile.write_all(image.data.as_ref()).unwrap();
                        tempfile
                    })
                    .collect();
                for album in files.chunks(10) {
                    self.upload_images_as_file(album, request.source.as_str()).await?;
                    self.upload_docs_as_file(album, request.source.as_str()).await?;
                }
            }
        }

        Produces::ok(())
    }
}

fn image_as_teloxide_file_doc(image: &Image) -> teloxide_core::types::InputFile {
    teloxide_core::types::InputFile::Memory {
        file_name: format!("doc_{}", &image.filename),
        data: Cow::from(image.data.to_vec()),
    }
}

fn image_as_teloxide_file(image: &Image) -> teloxide_core::types::InputFile {
    teloxide_core::types::InputFile::Memory {
        file_name: image.filename.clone(),
        data: Cow::from(image.data.to_vec()),
    }
}

pub struct TelegramReceiverActor {}
