use act_zero::{Actor, ActorResult};
use std::sync::Arc;

pub struct ImageRequest {
    pub source: String,
    pub body: ImageRequestBody,
}

pub enum ImageRequestBody {
    SingleImage { image: Image },
    Album { images: Vec<Image> },
}

#[derive(Clone)]
pub struct Image {
    pub filename: String,
    pub data: Arc<[u8]>,
}

#[async_trait::async_trait]
pub trait ImageSender: Send {
    async fn handle_request(&mut self, request: Arc<ImageRequest>) -> ActorResult<()>;
}

impl Actor for dyn ImageSender {}
