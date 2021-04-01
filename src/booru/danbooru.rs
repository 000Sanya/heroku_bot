use crate::booru::{ImageBoard, Post};
use itertools::Itertools;

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DanbooruPost {
    pub id: i64,
    pub score: i64,
    pub source: String,
    pub md5: String,
    pub rating: String,
    pub image_width: i64,
    pub image_height: i64,
    pub tag_string: String,
    pub file_ext: String,
    pub parent_id: Option<i64>,
    pub has_children: bool,
    pub tag_count_general: i64,
    pub tag_count_artist: i64,
    pub tag_count_character: i64,
    pub tag_count_copyright: i64,
    pub file_size: i64,
    pub pool_string: String,
    pub tag_count: i64,
    pub updated_at: String,
    pub pixiv_id: Option<i64>,
    pub tag_count_meta: i64,
    pub tag_string_general: String,
    pub tag_string_character: String,
    pub tag_string_copyright: String,
    pub tag_string_artist: String,
    pub tag_string_meta: String,
    pub file_url: String,
    pub large_file_url: String,
    pub preview_file_url: String,
}

impl From<DanbooruPost> for Post {
    fn from(post: DanbooruPost) -> Self {
        let DanbooruPost {
            id,
            source,
            file_url,
            preview_file_url,
            tag_string,
            ..
        } = post;

        Post {
            id: id,
            source: source.into(),
            preview: preview_file_url,
            full: file_url,
            tags: tag_string.split_terminator(" ").map(|s| s.to_owned()).collect(),
        }
    }
}

pub struct DanbooruApi {
    client: reqwest::Client,
}

impl DanbooruApi {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub fn with_proxy(proxy: impl AsRef<str>) -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .proxy(reqwest::Proxy::all(proxy.as_ref())?)
            .build()?;

        Ok(Self::new(client))
    }
}

#[async_trait::async_trait]
impl ImageBoard for DanbooruApi {
    async fn fetch_by_tags<'a>(&self, mut tags: impl Iterator<Item = &str> + Send + 'a, limit: u8) -> Vec<Post> {
        let tags = tags.join(" ");

        self.client
            .get("https://danbooru.donmai.us/posts.json")
            .query(&[("tags", &tags)])
            .send()
            .await
            .unwrap()
            .json::<Vec<DanbooruPost>>()
            .await
            .unwrap()
            .into_iter()
            .map(|post| post.into())
            .collect()
    }
}
