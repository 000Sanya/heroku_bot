use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Illust {
    pub illust_id: Option<String>,
    pub illust_type: Option<i64>,
    pub create_date: Option<String>,
    pub upload_date: Option<String>,
    pub urls: Urls,
    pub width: i64,
    pub height: i64,
    pub page_count: i64,
    pub pages: Vec<Page>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Urls {
    pub mini: String,
    pub thumb: String,
    pub small: String,
    pub regular: String,
    pub original: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub urls: Urls2,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Urls2 {
    #[serde(rename = "thumb_mini")]
    pub thumb_mini: String,
    pub small: String,
    pub regular: String,
    pub original: String,
}

#[derive(Debug, Error)]
pub enum PixivJsError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("json error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub struct PixivJsApi {
    client: reqwest::Client,
}

impl PixivJsApi {
    pub fn new() -> Result<Self, PixivJsError> {
        let client = reqwest::ClientBuilder::new()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.150 Safari/537.36")
            .build()?;

        Ok(Self { client })
    }

    pub async fn pages(&self, illust_id: i64) -> Result<Vec<Page>, PixivJsError> {
        let illusts = self
            .client
            .get(&format!("https://pixiv.js.org/api/illust/{}", illust_id))
            .send()
            .await?;

        let illusts = illusts.json::<Illust>().await?;

        Ok(illusts.pages)
    }

    pub async fn download(&self, url: &str) -> Result<(String, Vec<u8>), PixivJsError> {
        let url = url
            .replace("/-/", "https://i.pximg.net/")
            .replace("/~/", "https://s.pximg.net/");

        let res = self
            .client
            .get(&url)
            .header("Referer", "https://app-api.pixiv.net")
            .send()
            .await?;
        let filename = res.url().path().rsplit("/").next().expect("Need file name");
        Ok((filename.to_owned(), res.bytes().await?.to_vec()))
    }
}
