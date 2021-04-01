use crate::booru::Post;

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MoebooruPost {
    pub id: i64,
    pub tags: String,
    pub source: String,
    pub score: i64,
    pub md5: String,
    pub file_size: i64,
    pub file_ext: String,
    pub file_url: String,
    pub is_shown_in_index: bool,
    pub preview_url: String,
    pub preview_width: i64,
    pub preview_height: i64,
    pub actual_preview_width: i64,
    pub actual_preview_height: i64,
    pub sample_url: String,
    pub sample_width: i64,
    pub sample_height: i64,
    pub sample_file_size: i64,
    pub jpeg_url: String,
    pub jpeg_width: i64,
    pub jpeg_height: i64,
    pub jpeg_file_size: i64,
    pub rating: String,
    pub is_rating_locked: bool,
    pub has_children: bool,
    pub status: String,
    pub width: i64,
    pub height: i64,
    pub frames_pending_string: String,
    pub frames_pending: Vec<::serde_json::Value>,
    pub frames_string: String,
    pub frames: Vec<::serde_json::Value>,
}

impl From<MoebooruPost> for Post {
    fn from(post: MoebooruPost) -> Self {
        let MoebooruPost {
            id,
            tags,
            source,
            preview_url,
            file_url,
            ..
        } = post;

        Post {
            id,
            source: source.into(),
            tags: tags.split_ascii_whitespace().map(|s| s.to_owned()).collect(),
            preview: preview_url,
            full: file_url,
        }
    }
}
