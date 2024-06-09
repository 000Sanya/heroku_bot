use crate::config::Config;
use crate::processor::RequestProcessorActor;
use crate::request::{Image, ImageRequest, ImageRequestBody};
use crate::utils::ResultExtension;
use act_zero::{Actor, ActorResult, Addr, Produces};
use futures::future::join_all;
use std::sync::Arc;
use crate::pixiv_api::PixivClient;

pub struct PixivReceiveActor {
    client: PixivClient,
    processor: Addr<RequestProcessorActor>,
}

impl Actor for PixivReceiveActor {}

impl PixivReceiveActor {
    pub async fn new(_config: Arc<Config>, processor: Addr<RequestProcessorActor>) -> Self {
        let mut client = PixivClient::new();
        let _ = client
            .auth(&_config.pixiv_refresh)
            .await
            .unwrap();

        Self { client, processor }
    }

    pub async fn receive_illust(&mut self, id: i64) -> ActorResult<()> {
        log::info!("Start process {}", id);
        let illust = self
            .client
            .get_illust_detail(id)
            .await
            .on_error(|_| log::error!("Error on get illust"))?;
        log::info!("{:?}", &illust);
        let images: Vec<_> = join_all(
            illust
                .links()
                .map(|s| self.client.download(s))
                .collect::<Vec<_>>(),
        )
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .map(|(filename, data)| Image {
            filename,
            data: data.into(),
        })
        .collect();

        log::info!("Downloaded {} images", images.len());

        let req = ImageRequest {
            source: format!("https://www.pixiv.net/en/artworks/{}", id),
            body: if images.len() > 1 {
                ImageRequestBody::Album { images }
            } else if images.len() == 1 {
                ImageRequestBody::SingleImage {
                    image: images.into_iter().next().unwrap(),
                }
            } else {
                Err("Downloaded incorrect images")?
            },
        };

        act_zero::send!(self.processor.handle_request(req));

        Produces::ok(())
    }
}
