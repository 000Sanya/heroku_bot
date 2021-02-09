use crate::request::{ImageRequest, ImageSender};
use act_zero::{send, Actor, ActorResult, Addr, Produces};
use std::sync::Arc;

pub struct RequestProcessorActor {
    targets: Vec<Addr<dyn ImageSender + 'static>>,
}

impl Actor for RequestProcessorActor {}

impl RequestProcessorActor {
    pub fn new(targets: Vec<Addr<dyn ImageSender + 'static>>) -> Self {
        Self { targets }
    }

    pub async fn handle_request(&mut self, request: ImageRequest) -> ActorResult<()> {
        log::info!("Start process request from {}", &request.source);

        let request = Arc::new(request);

        for target in &self.targets {
            send!(target.handle_request(request.clone()));
        }

        Produces::ok(())
    }
}
