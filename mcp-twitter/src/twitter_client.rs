use anyhow::{anyhow, Result};
use reqwest::{Client, RequestBuilder};
use serde_json::Value;
use crate::auth::TwitterAuth;
use crate::models::*;

pub struct TwitterClient {
    client: Client,
    auth: TwitterAuth,
    base_url: String,
}

impl TwitterClient {
    pub fn new(auth: TwitterAuth) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            auth,
            base_url: "https://api.twitter.com/2".to_string(),
        })
    }

    pub async fn post_tweet(
        &self,
        text: &str,
        reply_to: Option<&str>,
        media_ids: Option<&[String]>,
    ) -> Result<PostTweetData> {
        let mut payload = serde_json::json!({
            "text": text
        });

        if let Some(reply_id) = reply_to {
            payload["reply"] = serde_json::json!({
                "in_reply_to_tweet_id": reply_id
            });
        }

        if let Some(media) = media_ids {
            payload["media"] = serde_json::json!({
                "media_ids": media
            });
        }

        let response = self
            .authenticated_request("POST", "/tweets")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to post tweet: {}", error_text));
        }

        let response_data: PostTweetResponse = response.json().await?;
        Ok(response_data.data)
    }

    pub async fn search_tweets(
        &self,
        query: &str,
        max_results: Option<u32>,
        tweet_fields: Option<&[String]>,
    ) -> Result<SearchResponse> {
        let mut url = format!("{}/tweets/search/recent", self.base_url);
        let mut params = vec![
            ("query", query.to_string()),
            ("max_results", max_results.unwrap_or(10).to_string()),
        ];

        if let Some(fields) = tweet_fields {
            params.push(("tweet.fields", fields.join(",")));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(self.auth.bearer_token.as_ref().ok_or_else(|| {
                anyhow!("Bearer token required for search")
            })?)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to search tweets: {}", error_text));
        }

        let search_response: SearchResponse = response.json().await?;
        Ok(search_response)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<TwitterUser> {
        let url = format!("{}/users/by/username/{}", self.base_url, username);
        let response = self
            .client
            .get(&url)
            .query(&[("user.fields", "created_at,description,location,pinned_tweet_id,profile_image_url,protected,public_metrics,url,verified,verified_type")])
            .bearer_auth(self.auth.bearer_token.as_ref().ok_or_else(|| {
                anyhow!("Bearer token required for user lookup")
            })?)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get user: {}", error_text));
        }

        let response_data: Value = response.json().await?;
        let user: TwitterUser = serde_json::from_value(response_data["data"].clone())?;
        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<TwitterUser> {
        let url = format!("{}/users/{}", self.base_url, user_id);
        let response = self
            .client
            .get(&url)
            .query(&[("user.fields", "created_at,description,location,pinned_tweet_id,profile_image_url,protected,public_metrics,url,verified,verified_type")])
            .bearer_auth(self.auth.bearer_token.as_ref().ok_or_else(|| {
                anyhow!("Bearer token required for user lookup")
            })?)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get user: {}", error_text));
        }

        let response_data: Value = response.json().await?;
        let user: TwitterUser = serde_json::from_value(response_data["data"].clone())?;
        Ok(user)
    }

    pub async fn get_user_timeline(
        &self,
        user_id: &str,
        max_results: u32,
        exclude_replies: bool,
        exclude_retweets: bool,
    ) -> Result<TimelineResponse> {
        let url = format!("{}/users/{}/tweets", self.base_url, user_id);
        let mut params = vec![
            ("max_results", max_results.to_string()),
            ("tweet.fields", "created_at,author_id,public_metrics,context_annotations".to_string()),
        ];

        if exclude_replies {
            params.push(("exclude", "replies".to_string()));
        }

        if exclude_retweets {
            let exclude_value = if exclude_replies {
                "replies,retweets".to_string()
            } else {
                "retweets".to_string()
            };
            params.retain(|(key, _)| key != &"exclude");
            params.push(("exclude", exclude_value));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(self.auth.bearer_token.as_ref().ok_or_else(|| {
                anyhow!("Bearer token required for timeline")
            })?)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get timeline: {}", error_text));
        }

        let timeline_response: TimelineResponse = response.json().await?;
        Ok(timeline_response)
    }

    pub async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet> {
        let url = format!("{}/tweets/{}", self.base_url, tweet_id);
        let response = self
            .client
            .get(&url)
            .query(&[("tweet.fields", "created_at,author_id,public_metrics,context_annotations,entities,geo,lang,possibly_sensitive,referenced_tweets,reply_settings,source")])
            .bearer_auth(self.auth.bearer_token.as_ref().ok_or_else(|| {
                anyhow!("Bearer token required for tweet lookup")
            })?)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get tweet: {}", error_text));
        }

        let response_data: Value = response.json().await?;
        let tweet: Tweet = serde_json::from_value(response_data["data"].clone())?;
        Ok(tweet)
    }

    pub async fn get_trends(&self, location: &str) -> Result<TrendsResponse> {
        // Note: Twitter API v2 doesn't have trends endpoint like v1.1
        // This is a placeholder implementation
        Err(anyhow!("Trends endpoint not available in Twitter API v2"))
    }

    pub async fn get_analytics(
        &self,
        user_id: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        metrics: Option<&[String]>,
    ) -> Result<Analytics> {
        // Note: Analytics require special permissions and are not available in basic API
        Err(anyhow!("Analytics require Twitter API v2 Academic Research or Enterprise access"))
    }

    fn authenticated_request(&self, method: &str, endpoint: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, endpoint);
        let request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => self.client.get(&url),
        };

        // Use OAuth 1.0a for write operations if available
        if let (Some(access_token), Some(access_token_secret)) = 
            (&self.auth.access_token, &self.auth.access_token_secret) {
            // For now, we'll use bearer token as OAuth 1.0a is more complex
            if let Some(bearer_token) = &self.auth.bearer_token {
                request.bearer_auth(bearer_token)
            } else {
                request
            }
        } else if let Some(bearer_token) = &self.auth.bearer_token {
            request.bearer_auth(bearer_token)
        } else {
            request
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::TwitterAuth;

    #[test]
    fn test_client_creation() {
        let auth = TwitterAuth::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            Some("test_token".to_string()),
            Some("test_token_secret".to_string()),
            Some("test_bearer".to_string()),
        );
        
        let client = TwitterClient::new(auth);
        assert!(client.is_ok());
    }
}