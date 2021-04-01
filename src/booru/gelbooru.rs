use crate::booru::Post;

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GelbooruPost {
    pub source: String,
    pub directory: String,
    pub hash: String,
    pub height: i64,
    pub id: i64,
    pub image: String,
    pub change: i64,
    pub owner: String,
    pub parent_id: Option<i64>,
    pub rating: String,
    pub sample: i64,
    pub preview_height: i64,
    pub preview_width: i64,
    pub sample_height: i64,
    pub sample_width: i64,
    pub score: i64,
    pub tags: String,
    pub title: String,
    pub width: i64,
    pub file_url: String,
    pub created_at: String,
}

impl From<GelbooruPost> for Post {
    fn from(post: GelbooruPost) -> Self {
        let GelbooruPost {
            id,
            source,
            file_url,
            tags,
            directory,
            image,
            ..
        } = post;

        let preview = format!(
            "https://img3.gelbooru.com/thumbnails/{}/thumbnail_{}",
            directory, image
        );

        Post {
            id,
            source: source.into(),
            tags: tags.split_ascii_whitespace().map(|s| s.to_owned()).collect(),
            full: file_url,
            preview,
        }
    }
}
