use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub telegram_token: String,
    pub pixiv_refresh: String,
    pub telegram_target: i64,
    pub telegram_host: String,
    pub vk_bot_token: String,
    pub vk_target: i64,
    pub discord_webhook: String,
}

pub fn get_config() -> Arc<Config> {
    Arc::new(
        envy::prefixed("NS_")
            .from_env::<Config>()
            .expect("Error on load config"),
    )
}
