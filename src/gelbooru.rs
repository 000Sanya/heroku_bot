use std::sync::Arc;
use act_zero::{Actor, ActorResult, Addr, Produces};
use crate::config::Config;
use crate::processor::RequestProcessorActor;
use crate::request::{Image, ImageRequest, ImageRequestBody};
use crate::utils::ResultExtension;

pub struct GelbooruReceiveActor {
    client: gelbooru_api::Client,
    processor: Addr<RequestProcessorActor>,
}

impl Actor for GelbooruReceiveActor {}

impl GelbooruReceiveActor {
    pub fn new(_config: Arc<Config>, processor: Addr<RequestProcessorActor>) -> Self {
        let client = gelbooru_api::Client::public();

        Self { client, processor }
    }

    pub async fn receive_id(&mut self, id: u64, url: String) -> ActorResult<()> {
        log::info!("Start gelbooru process {}", id);

        let result = gelbooru_api::posts()
            .tag(format!("id:{}", id))
            .limit(1)
            .send(&self.client)
            .await
            .on_error(|x| log::error!("Error on request posts"))
            .ok();

        if let Some(result) = result {
            if result.posts.len() >= 1 {
                let post = result.posts.into_iter().next().unwrap();

                let image = reqwest::get(&post.file_url)
                    .await
                    .on_error(|x| log::error!("Error on download image"))
                    .ok();
                let image = match image {
                    None => None,
                    Some(image) => image.bytes().await.ok()
                };

                if let Some(image) = image {
                    let request = ImageRequest {
                        source: url,
                        body: ImageRequestBody::SingleImage {
                            image: Image {
                                filename: post.file_url.split_once("/").unwrap().1.to_string(),
                                data: image.as_ref().to_owned().into()
                            }
                        }
                    };

                    act_zero::send!(self.processor.handle_request(request))
                }
            }
        }

        Produces::ok(())
    }
}
