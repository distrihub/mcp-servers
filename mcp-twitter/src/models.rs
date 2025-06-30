use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TwitterUser {
    pub id: String,
    pub name: String,
    pub username: String,
    pub created_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub pinned_tweet_id: Option<String>,
    pub profile_image_url: Option<String>,
    pub protected: Option<bool>,
    pub public_metrics: Option<UserPublicMetrics>,
    pub url: Option<String>,
    pub verified: Option<bool>,
    pub verified_type: Option<String>,
    pub withheld: Option<UserWithheld>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPublicMetrics {
    pub followers_count: u64,
    pub following_count: u64,
    pub tweet_count: u64,
    pub listed_count: u64,
    pub like_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWithheld {
    pub country_codes: Vec<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub author_id: Option<String>,
    pub conversation_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub edit_history_tweet_ids: Option<Vec<String>>,
    pub entities: Option<TweetEntities>,
    pub geo: Option<TweetGeo>,
    pub in_reply_to_user_id: Option<String>,
    pub lang: Option<String>,
    pub non_public_metrics: Option<TweetNonPublicMetrics>,
    pub organic_metrics: Option<TweetOrganicMetrics>,
    pub possibly_sensitive: Option<bool>,
    pub promoted_metrics: Option<TweetPromotedMetrics>,
    pub public_metrics: Option<TweetPublicMetrics>,
    pub referenced_tweets: Option<Vec<ReferencedTweet>>,
    pub reply_settings: Option<String>,
    pub source: Option<String>,
    pub withheld: Option<TweetWithheld>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetEntities {
    pub annotations: Option<Vec<Annotation>>,
    pub cashtags: Option<Vec<CashtagEntity>>,
    pub hashtags: Option<Vec<HashtagEntity>>,
    pub mentions: Option<Vec<MentionEntity>>,
    pub urls: Option<Vec<UrlEntity>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub start: u32,
    pub end: u32,
    pub probability: f64,
    pub r#type: String,
    pub normalized_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashtagEntity {
    pub start: u32,
    pub end: u32,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashtagEntity {
    pub start: u32,
    pub end: u32,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentionEntity {
    pub start: u32,
    pub end: u32,
    pub username: String,
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlEntity {
    pub start: u32,
    pub end: u32,
    pub url: String,
    pub expanded_url: Option<String>,
    pub display_url: Option<String>,
    pub unwound_url: Option<String>,
    pub status: Option<u16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub images: Option<Vec<UrlImage>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlImage {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetGeo {
    pub coordinates: Option<GeoCoordinates>,
    pub place_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeoCoordinates {
    pub r#type: String,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetPublicMetrics {
    pub retweet_count: u64,
    pub reply_count: u64,
    pub like_count: u64,
    pub quote_count: u64,
    pub bookmark_count: Option<u64>,
    pub impression_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetNonPublicMetrics {
    pub impression_count: u64,
    pub url_link_clicks: Option<u64>,
    pub user_profile_clicks: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetOrganicMetrics {
    pub impression_count: u64,
    pub like_count: u64,
    pub reply_count: u64,
    pub retweet_count: u64,
    pub url_link_clicks: Option<u64>,
    pub user_profile_clicks: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetPromotedMetrics {
    pub impression_count: u64,
    pub like_count: u64,
    pub reply_count: u64,
    pub retweet_count: u64,
    pub url_link_clicks: Option<u64>,
    pub user_profile_clicks: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferencedTweet {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetWithheld {
    pub copyright: Option<bool>,
    pub country_codes: Vec<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: Option<Vec<Tweet>>,
    pub includes: Option<SearchIncludes>,
    pub meta: Option<SearchMeta>,
    pub errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchIncludes {
    pub users: Option<Vec<TwitterUser>>,
    pub tweets: Option<Vec<Tweet>>,
    pub places: Option<Vec<Place>>,
    pub media: Option<Vec<Media>>,
    pub polls: Option<Vec<Poll>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMeta {
    pub newest_id: Option<String>,
    pub oldest_id: Option<String>,
    pub result_count: Option<u32>,
    pub next_token: Option<String>,
    pub previous_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub full_name: String,
    pub name: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub geo: Option<PlaceGeo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceGeo {
    pub r#type: String,
    pub bbox: Option<Vec<f64>>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub media_key: String,
    pub r#type: String,
    pub url: Option<String>,
    pub duration_ms: Option<u32>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub preview_image_url: Option<String>,
    pub public_metrics: Option<MediaPublicMetrics>,
    pub non_public_metrics: Option<MediaNonPublicMetrics>,
    pub organic_metrics: Option<MediaOrganicMetrics>,
    pub promoted_metrics: Option<MediaPromotedMetrics>,
    pub alt_text: Option<String>,
    pub variants: Option<Vec<MediaVariant>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaPublicMetrics {
    pub view_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaNonPublicMetrics {
    pub playback_0_count: Option<u64>,
    pub playback_25_count: Option<u64>,
    pub playback_50_count: Option<u64>,
    pub playback_75_count: Option<u64>,
    pub playback_100_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaOrganicMetrics {
    pub playback_0_count: Option<u64>,
    pub playback_25_count: Option<u64>,
    pub playback_50_count: Option<u64>,
    pub playback_75_count: Option<u64>,
    pub playback_100_count: Option<u64>,
    pub view_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaPromotedMetrics {
    pub playback_0_count: Option<u64>,
    pub playback_25_count: Option<u64>,
    pub playback_50_count: Option<u64>,
    pub playback_75_count: Option<u64>,
    pub playback_100_count: Option<u64>,
    pub view_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaVariant {
    pub bit_rate: Option<u32>,
    pub content_type: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    pub id: String,
    pub options: Vec<PollOption>,
    pub duration_minutes: Option<u32>,
    pub end_datetime: Option<DateTime<Utc>>,
    pub voting_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollOption {
    pub position: u32,
    pub label: String,
    pub votes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub detail: String,
    pub title: String,
    pub r#type: String,
    pub resource_type: Option<String>,
    pub parameter: Option<String>,
    pub resource_id: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostTweetResponse {
    pub data: PostTweetData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostTweetData {
    pub id: String,
    pub text: String,
    pub edit_history_tweet_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendLocation {
    pub name: String,
    pub woeid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trend {
    pub name: String,
    pub url: String,
    pub promoted_content: Option<String>,
    pub query: String,
    pub tweet_volume: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendsResponse {
    pub trends: Vec<Trend>,
    pub as_of: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub locations: Vec<TrendLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Analytics {
    pub data: Vec<AnalyticsData>,
    pub meta: Option<AnalyticsMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub metric: String,
    pub value: u64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsMeta {
    pub total_tweet_count: Option<u64>,
    pub result_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineResponse {
    pub data: Option<Vec<Tweet>>,
    pub includes: Option<SearchIncludes>,
    pub meta: Option<SearchMeta>,
    pub errors: Option<Vec<ApiError>>,
}