pub mod danbooru;
pub mod gelbooru;
pub mod moebooru;

use once_cell::sync::Lazy;
use regex::Regex;
use std::iter::once_with;

static PXIMG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"https://i\.pximg\.net/img-original/img/\d+/\d+/\d+/\d+/\d+/\d+/(?P<id>\d+)_p\d+\..+"#,
    )
    .expect("Error on compile regex")
});
static PIXIV_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://(www\.)?pixiv\.net/en/artworks/(?P<id>\d+)")
        .expect("Error on compile regex")
});

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Post {
    pub id: i64,
    pub source: Source,
    pub tags: Vec<String>,
    pub preview: String, // URL
    pub full: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Source {
    Twitter { url: String },
    Pixiv { id: i64 },
    Url { url: String },
}

impl From<String> for Source {
    fn from(url: String) -> Self {
        if url.as_str().contains("www.pixiv.net") {
            let id = PIXIV_REGEX
                .captures(url.as_str())
                .and_then(|c| c.name("id"))
                .and_then(|m| m.as_str().parse::<i64>().ok());

            id.map_or(Source::Url { url }, |id| Source::Pixiv { id })
        } else if url.as_str().contains("pximg.net") {
            let id = PXIMG_REGEX
                .captures(url.as_str())
                .and_then(|c| c.name("id"))
                .and_then(|m| m.as_str().parse::<i64>().ok());

            id.map_or(Source::Url { url }, |id| Source::Pixiv { id })
        } else if url.as_str().contains("twitter") {
            Source::Twitter { url }
        } else {
            Source::Url { url }
        }
    }
}

#[async_trait::async_trait]
pub trait ImageBoard {
    async fn fetch_by_tags<'a>(&self, tags: impl Iterator<Item = &str> + Send + 'a, limit: u8) -> Vec<Post>;

    async fn fetch_by_id(&self, id: i64) -> Option<Post> {
        let id = format!("id:{}", id);
        self.fetch_by_tags(once_with(|| id.as_str()), 1)
            .await
            .into_iter()
            .next()
    }
}
