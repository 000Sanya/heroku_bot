use chrono::prelude::*;
use md5::{Digest, Md5};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivClientData {
    access_token: String,
    expires_in: Number,
    token_type: String,
    scope: String,
    refresh_token: String,
    user: PixivClientUser,
    device_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProfileImage {
    px_16x16: String,
    px_50x50: String,
    px_170x170: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivClientUser {
    profile_image_urls: ProfileImage,
    id: String,
    name: String,
    account: String,
    mail_address: String,
    is_premium: bool,
    x_restrict: Number,
    is_mail_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivRequestData {
    client_id: String,
    client_secret: String,
    get_secure_url: String,
    grant_type: String,
    refresh_token: String,
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Restrict {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    day,
    week,
    month,
    day_male,
    day_female,
    week_original,
    week_rookie,
    day_r18,
    day_male_r18,
    day_female_r18,
    week_r18,
    week_r18g,
    day_manga,
    week_manga,
    month_manga,
    week_rookie_manga,
    day_r18_manga,
    week_r18_manga,
    week_r18g_manga,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchTarget {
    partial_match_for_tags,
    exact_match_for_tags,
    title_and_caption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Sort {
    date_desc,
    date_asc,
    popular_desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivParams {
    user_id: Option<Number>,
    #[serde(rename = "type")]
    r#type: Option<String>,
    filter: Option<String>,
    restrict: Option<Restrict>,
    illust_id: Option<Number>,
    content_type: Option<String>,
    include_total_comments: Option<bool>,
    include_ranking_label: Option<bool>,
    include_ranking_illusts: Option<bool>,
    include_ranking_novels: Option<bool>,
    mode: Option<Mode>,
    word: Option<String>,
    search_target: Option<SearchTarget>,
    sort: Option<Sort>,
    start_date: Option<String>,
    end_date: Option<String>,
    offset: Option<Number>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivFetchOptions {
    data: Option<PixivParams>,
    method: Option<String>,
    headers: HashMap<String, String>,
    params: Option<PixivParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProfileImageUrl {
    medium: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivUser {
    id: Number,
    name: String,
    account: String,
    profile_image_urls: ProfileImageUrl,
    comment: Option<String>,
    is_followed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Profile {
    webpage: String,
    gender: String,
    birth: String,
    birth_day: String,
    birth_year: Number,
    region: String,
    address_id: Number,
    country_code: String,
    job: String,
    job_id: Number,
    total_follow_users: Number,
    total_mypixiv_users: Number,
    total_illusts: Number,
    total_manga: Number,
    total_novels: Number,
    total_illust_bookmarks_public: Number,
    total_illust_series: Number,
    background_image_url: String,
    twitter_account: String,
    twitter_url: String,
    pawoo_url: String,
    is_premium: bool,
    is_using_custom_profile_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProfilePublicity {
    gender: String,
    region: String,
    birth_day: String,
    birth_year: String,
    job: String,
    pawoo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Workspace {
    pc: String,
    monitor: String,
    tool: String,
    scanner: String,
    tablet: String,
    mouse: String,
    printer: String,
    desktop: String,
    music: String,
    desk: String,
    chair: String,
    comment: String,
    workspace_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivUserDetail {
    user: PixivUser,
    profile: Profile,
    profile_publicity: ProfilePublicity,
    workspace: Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivIllustDetail {
    illust: PixivIllust,
}

impl PixivIllustDetail {
    pub fn links(&self) -> impl Iterator<Item = &str> {
        self.illust
            .meta_pages
            .iter()
            .map(|x| x.image_urls.large.as_str())
            .chain(
                std::iter::once(self.illust.meta_single_page.original_image_url.as_ref())
                    .filter_map(|x| x.map(|x| x.as_str())),
            )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivIllustSearch {
    illusts: Vec<PixivIllust>,
    next_url: Option<String>,
    search_span_limit: Option<Number>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserPreviews {
    user: PixivUser,
    illusts: Vec<PixivIllust>,
    novels: Vec<PixivNovel>,
    is_muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivUserSearch {
    user_previews: Vec<UserPreviews>,
    next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivCommentSearch {
    total_comments: Number,
    comments: Vec<PixivComment>,
    next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivNovelSearch {
    novels: Vec<PixivNovel>,
    next_url: Option<String>,
    privacy_policy: Option<Value>,
    search_span_limit: Option<Number>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivBookmarkSearch {
    bookmark_tags: Vec<PixivTag>,
    next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivMangaSearch {
    illusts: Vec<PixivManga>,
    ranking_illusts: Vec<Value>,
    privacy_policy: Value,
    next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImageUrls {
    square_medium: String,
    medium: String,
    large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MetaSinglePage {
    original_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivIllust {
    id: Number,
    title: String,
    #[serde(rename = "type")]
    r#type: String,
    image_urls: ImageUrls,
    caption: String,
    restrict: Number,
    x_restrict: Number,
    user: PixivUser,
    tags: Vec<PixivTag>,
    tools: Vec<String>,
    create_date: String,
    page_count: Number,
    width: Number,
    height: Number,
    sanity_level: Number,
    meta_single_page: MetaSinglePage,
    meta_pages: Vec<PixivMetaPage>,
    total_view: Number,
    total_bookmarks: Number,
    is_bookmarked: bool,
    visible: bool,
    is_muted: bool,
    total_comments: Number,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivTag {
    name: String,
    translated_name: Option<String>,
    added_by_uploaded_user: Option<bool>,
    illust: Option<PixivIllust>,
    is_registered: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MetaImageUrls {
    square_medium: String,
    medium: String,
    large: String,
    original: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivMetaPage {
    image_urls: MetaImageUrls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivComment {
    id: Number,
    comment: String,
    date: String,
    user: PixivUser,
    parent_comment: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivNovel {
    id: Number,
    title: String,
    caption: String,
    restrict: Number,
    x_restrict: Number,
    image_urls: ImageUrls,
    create_date: String,
    tags: Vec<PixivTag>,
    page_count: Number,
    text_length: Number,
    user: PixivUser,
    series: Value,
    is_bookmarked: bool,
    total_bookmarks: Number,
    total_view: Number,
    visible: bool,
    total_comments: Number,
    is_muted: bool,
    is_mypixiv_only: bool,
    is_xrestricted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivManga {
    id: Number,
    title: String,
    #[serde(rename = "type")]
    r#type: String,
    image_urls: ImageUrls,
    caption: String,
    restrict: Number,
    user: PixivUser,
    tags: Vec<PixivTag>,
    tools: Vec<String>,
    create_date: String,
    page_count: String,
    width: Number,
    height: Number,
    sanity_level: Number,
    x_restrict: Number,
    series: Option<Value>,
    meta_single_page: Value,
    meta_pages: Vec<PixivMetaPage>,
    total_view: Number,
    total_bookmarks: Number,
    is_bookmarked: bool,
    visible: bool,
    is_muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivTrendTags {
    trend_tags: Vec<PixivTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivAutoComplete {
    search_auto_complete_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixivBookmarkDetail {
    is_bookmarked: bool,
    tags: Vec<PixivTag>,
    restrict: String,
}

const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";
const HASH_SECRET: &str = "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c";

pub struct PixivClient {
    client: reqwest::Client,
    client_data: Option<PixivClientData>,
}

impl PixivClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("PixivAndroidApp/5.0.64 (Android 6.0)")
                .default_headers({
                    let mut headers = header::HeaderMap::new();
                    headers.insert(
                        header::ACCEPT_LANGUAGE,
                        header::HeaderValue::from_static("android"),
                    );
                    headers
                })
                .build()
                .expect("Error"),
            client_data: None,
        }
    }

    pub async fn auth(&mut self, login: &str, password: &str) -> reqwest::Result<PixivClientData> {
        let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, false);
        let mut md5 = Md5::new();
        md5.update(format!("{}{}", &datetime, HASH_SECRET).as_bytes());
        let hash = md5.finalize();
        let hash = format!("{:x}", &hash);

        let v: PixivClientData = self
            .client
            .post("https://oauth.secure.pixiv.net/auth/token")
            .header("X-Client-Time", &datetime)
            .header("X-Client-Hash", &hash)
            .form(&PixivRequestData {
                client_id: CLIENT_ID.to_owned(),
                client_secret: CLIENT_SECRET.to_owned(),
                get_secure_url: "1".to_string(),
                grant_type: "password".to_string(),
                refresh_token: "".to_string(),
                username: login.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?
            .json()
            .await?;

        self.client_data.replace(v.clone());
        Ok(v)
    }

    pub async fn get_illust_detail(&self, illust_id: i64) -> reqwest::Result<PixivIllustDetail> {
        let request = self
            .client
            .get("https://app-api.pixiv.net/v1/illust/detail")
            .query(&[("illust_id", illust_id.to_string())])
            .bearer_auth(
                self.client_data
                    .as_ref()
                    .expect("Need auth")
                    .access_token
                    .as_str(),
            )
            .build()?;

        request.url();

        self.client.execute(request).await?.json().await
    }

    pub async fn download(&self, url: &str) -> reqwest::Result<(String, Vec<u8>)> {
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
