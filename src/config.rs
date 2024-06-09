use serde::Deserialize;
use std::sync::Arc;
use teloxide_core::prelude::ChatId;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub telegram_token: String,
    pub pixiv_refresh: String,
    pub telegram_target: ChatId,
    pub telegram_host: Option<String>,
    pub vk_bot_token: String,
    pub vk_target: i64,
    pub discord_webhook: Option<String>,
}

pub fn get_config() -> Arc<Config> {
    Arc::new(
        envy::prefixed("NS_")
            .from_env::<Config>()
            .expect("Error on load config"),
    )
}
