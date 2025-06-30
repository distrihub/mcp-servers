use anyhow::{anyhow, Result};
use async_mcp::{
    Content, PromptMessage, Resource, Server, Tool, ToolCall, ToolResult, ClientCapabilities, McpError
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use reqwest::Client;
use chrono::{DateTime, Utc};

pub mod twitter_client;
pub mod auth;
pub mod models;

use twitter_client::TwitterClient;
use auth::TwitterAuth;
use models::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostTweetRequest {
    pub text: String,
    pub reply_to: Option<String>,
    pub media_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTweetsRequest {
    pub query: String,
    pub max_results: Option<u32>,
    pub tweet_fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserRequest {
    pub username: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsRequest {
    pub user_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub metrics: Option<Vec<String>>,
}

pub struct McpTwitterServer {
    client: TwitterClient,
    auth: TwitterAuth,
}

impl McpTwitterServer {
    pub fn new(
        api_key: String,
        api_secret: String,
        access_token: Option<String>,
        access_token_secret: Option<String>,
        bearer_token: Option<String>,
    ) -> Result<Self> {
        let auth = TwitterAuth::new(
            api_key,
            api_secret,
            access_token,
            access_token_secret,
            bearer_token,
        );
        let client = TwitterClient::new(auth.clone())?;

        Ok(Self { client, auth })
    }

    pub async fn serve(&self) -> Result<()> {
        let server = Server::new();

        // Register tools
        server.add_tool(Tool::new(
            "post_tweet",
            "Post a tweet to Twitter/X",
            json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text content of the tweet (max 280 characters)",
                        "maxLength": 280
                    },
                    "reply_to": {
                        "type": "string",
                        "description": "Tweet ID to reply to (optional)"
                    },
                    "media_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Media IDs to attach to the tweet"
                    }
                },
                "required": ["text"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "search_tweets",
            "Search for tweets using Twitter's search API",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (supports Twitter search operators)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results (10-100)",
                        "minimum": 10,
                        "maximum": 100,
                        "default": 10
                    },
                    "tweet_fields": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Additional tweet fields to include",
                        "enum": ["created_at", "author_id", "public_metrics", "context_annotations"]
                    }
                },
                "required": ["query"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "get_user_info",
            "Get information about a Twitter user",
            json!({
                "type": "object",
                "properties": {
                    "username": {
                        "type": "string",
                        "description": "Twitter username (without @)"
                    },
                    "user_id": {
                        "type": "string",
                        "description": "Twitter user ID"
                    }
                },
                "oneOf": [
                    {"required": ["username"]},
                    {"required": ["user_id"]}
                ]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "get_user_timeline",
            "Get recent tweets from a user's timeline",
            json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "Twitter user ID"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of tweets (5-100)",
                        "minimum": 5,
                        "maximum": 100,
                        "default": 10
                    },
                    "exclude_replies": {
                        "type": "boolean",
                        "description": "Exclude replies from timeline",
                        "default": false
                    },
                    "exclude_retweets": {
                        "type": "boolean",
                        "description": "Exclude retweets from timeline",
                        "default": false
                    }
                },
                "required": ["user_id"]
            }),
        )).await?;

        server.add_tool(Tool::new(
            "get_tweet_analytics",
            "Get analytics data for tweets",
            json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "User ID to get analytics for"
                    },
                    "start_time": {
                        "type": "string",
                        "description": "Start time for analytics (ISO 8601 format)"
                    },
                    "end_time": {
                        "type": "string",
                        "description": "End time for analytics (ISO 8601 format)"
                    },
                    "metrics": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Metrics to include",
                        "enum": ["impressions", "engagements", "retweets", "likes", "replies"]
                    }
                }
            }),
        )).await?;

        // Register resources
        server.add_resource(Resource::new(
            "twitter://user/{user_id}",
            "Twitter user profile information",
            Some("application/json".to_string()),
        )).await?;

        server.add_resource(Resource::new(
            "twitter://tweet/{tweet_id}",
            "Individual tweet content and metadata",
            Some("application/json".to_string()),
        )).await?;

        server.add_resource(Resource::new(
            "twitter://trends/{location}",
            "Trending topics for a specific location",
            Some("application/json".to_string()),
        )).await?;

        // Set tool handlers
        server.set_tool_handler(|call: ToolCall| async move {
            self.handle_tool_call(call).await
        }).await?;

        // Set resource handler
        server.set_resource_handler(|uri: &str| async move {
            self.handle_resource_request(uri).await
        }).await?;

        server.start().await?;
        Ok(())
    }

    async fn handle_tool_call(&self, call: ToolCall) -> Result<ToolResult> {
        match call.name.as_str() {
            "post_tweet" => {
                let req: PostTweetRequest = serde_json::from_value(call.arguments)?;
                let result = self.client.post_tweet(&req.text, req.reply_to.as_deref(), req.media_ids.as_deref()).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: format!("Tweet posted successfully! ID: {}\nURL: https://twitter.com/i/status/{}", 
                                    result.id, result.id),
                    }],
                    is_error: false,
                })
            }
            "search_tweets" => {
                let req: SearchTweetsRequest = serde_json::from_value(call.arguments)?;
                let results = self.client.search_tweets(
                    &req.query, 
                    req.max_results, 
                    req.tweet_fields.as_deref()
                ).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&results)?,
                    }],
                    is_error: false,
                })
            }
            "get_user_info" => {
                let req: GetUserRequest = serde_json::from_value(call.arguments)?;
                let user = if let Some(username) = req.username {
                    self.client.get_user_by_username(&username).await?
                } else if let Some(user_id) = req.user_id {
                    self.client.get_user_by_id(&user_id).await?
                } else {
                    return Err(anyhow!("Either username or user_id must be provided"));
                };
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&user)?,
                    }],
                    is_error: false,
                })
            }
            "get_user_timeline" => {
                let user_id = call.arguments.get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("user_id is required"))?;
                let max_results = call.arguments.get("max_results")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u32;
                let exclude_replies = call.arguments.get("exclude_replies")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let exclude_retweets = call.arguments.get("exclude_retweets")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let timeline = self.client.get_user_timeline(
                    user_id, 
                    max_results, 
                    exclude_replies, 
                    exclude_retweets
                ).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&timeline)?,
                    }],
                    is_error: false,
                })
            }
            "get_tweet_analytics" => {
                let req: AnalyticsRequest = serde_json::from_value(call.arguments)?;
                let analytics = self.client.get_analytics(
                    req.user_id.as_deref(),
                    req.start_time.as_deref(),
                    req.end_time.as_deref(),
                    req.metrics.as_deref(),
                ).await?;
                
                Ok(ToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&analytics)?,
                    }],
                    is_error: false,
                })
            }
            _ => Err(anyhow!("Unknown tool: {}", call.name)),
        }
    }

    async fn handle_resource_request(&self, uri: &str) -> Result<Resource> {
        if let Some(user_id) = uri.strip_prefix("twitter://user/") {
            let user = self.client.get_user_by_id(user_id).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("@{}", user.username)),
                description: Some(format!("Profile information for {}", user.name)),
                mime_type: Some("application/json".to_string()),
                text: Some(serde_json::to_string_pretty(&user)?),
                blob: None,
            })
        } else if let Some(tweet_id) = uri.strip_prefix("twitter://tweet/") {
            let tweet = self.client.get_tweet(tweet_id).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("Tweet {}", tweet_id)),
                description: Some("Tweet content and metadata".to_string()),
                mime_type: Some("application/json".to_string()),
                text: Some(serde_json::to_string_pretty(&tweet)?),
                blob: None,
            })
        } else if let Some(location) = uri.strip_prefix("twitter://trends/") {
            let trends = self.client.get_trends(location).await?;
            
            Ok(Resource {
                uri: uri.to_string(),
                name: Some(format!("Trends for {}", location)),
                description: Some(format!("Trending topics in {}", location)),
                mime_type: Some("application/json".to_string()),
                text: Some(serde_json::to_string_pretty(&trends)?),
                blob: None,
            })
        } else {
            Err(anyhow!("Unknown resource URI: {}", uri))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = McpTwitterServer::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            Some("test_token".to_string()),
            Some("test_token_secret".to_string()),
            Some("test_bearer".to_string()),
        );
        assert!(server.is_ok());
    }

    #[test]
    fn test_post_tweet_request_serialization() {
        let request = PostTweetRequest {
            text: "Hello, world!".to_string(),
            reply_to: None,
            media_ids: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_search_tweets_request_serialization() {
        let request = SearchTweetsRequest {
            query: "rust programming".to_string(),
            max_results: Some(50),
            tweet_fields: Some(vec!["created_at".to_string(), "public_metrics".to_string()]),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("rust programming"));
    }
}