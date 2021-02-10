use reqwest::header::{HeaderValue, COOKIE};
use serde::Deserialize;
use std::fmt::Debug;
use thiserror::Error;

pub struct PixivAjaxClient {
    phpssesid: String,
    client: reqwest::Client,
}

impl PixivAjaxClient {
    pub fn new(phpssesid: String) -> Result<Self, PixivAjaxError> {
        let client = reqwest::ClientBuilder::new()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.150 Safari/537.36")
            .build()?;

        Ok(Self { phpssesid, client })
    }

    pub async fn pages(&self, illust_id: i64) -> Result<Vec<Page>, PixivAjaxError> {
        self.client
            .get(&format!(
                "https://www.pixiv.net/ajax/illust/{}/pages?lang=en",
                illust_id
            ))
            .header(
                COOKIE,
                HeaderValue::from_str(&format!("PHPSESSID={}", self.phpssesid.as_str())).unwrap(),
            )
            .send()
            .await?
            .json::<Response<Page>>()
            .await?
            .into_result()
    }

    pub async fn download(&self, url: &str) -> Result<(String, Vec<u8>), PixivAjaxError> {
        let res = self
            .client
            .get(url)
            .header("Referer", "https://app-api.pixiv.net")
            .send()
            .await?;
        let filename = res.url().path().rsplit("/").next().expect("Need file name");
        Ok((filename.to_owned(), res.bytes().await?.to_vec()))
    }
}

#[derive(Debug, Error)]
pub enum PixivAjaxError {
    #[error("Pixiv api error: {0}")]
    ApiError(String),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("json error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Deserialize)]
struct Response<T: Debug> {
    error: bool,
    message: String,
    body: Vec<T>,
}

impl<T: Debug> Response<T> {
    fn into_result(self) -> Result<Vec<T>, PixivAjaxError> {
        log::info!("{:?}", self);
        match self.error {
            false => Ok(self.body),
            true => Err(PixivAjaxError::ApiError(self.message)),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Page {
    pub urls: Urls,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Urls {
    pub thumb_mini: String,
    pub small: String,
    pub regular: String,
    pub original: String,
}
