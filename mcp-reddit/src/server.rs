use anyhow::Context;
use anyhow::Result;
use async_mcp::server::{Server, ServerBuilder};
use async_mcp::transport::Transport;
use async_mcp::types::{
    CallToolRequest, CallToolResponse, ListRequest, PromptsListResponse, Resource,
    ResourcesListResponse, ServerCapabilities, Tool, ToolResponseContent,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tracing::info;
use url::Url;

const REDDIT_API_BASE: &str = "https://oauth.reddit.com";
const REDDIT_OAUTH_URL: &str = "https://www.reddit.com/api/v1/access_token";

#[derive(Debug, Serialize, Deserialize)]
struct RedditToken {
    access_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditListing<T> {
    data: RedditListingData<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditListingData<T> {
    children: Vec<RedditChild<T>>,
    after: Option<String>,
    before: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditChild<T> {
    data: T,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditPost {
    id: String,
    title: String,
    selftext: String,
    author: String,
    subreddit: String,
    score: i32,
    upvote_ratio: f64,
    num_comments: i32,
    created_utc: f64,
    url: String,
    permalink: String,
    is_self: bool,
    domain: String,
    over_18: bool,
    spoiler: bool,
    stickied: bool,
    locked: bool,
    archived: bool,
    clicked: bool,
    hidden: bool,
    saved: bool,
    is_video: bool,
    thumbnail: String,
    preview: Option<RedditPreview>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditPreview {
    images: Vec<RedditImage>,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditImage {
    source: RedditImageSource,
    resolutions: Vec<RedditImageSource>,
    variants: Option<serde_json::Value>,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditImageSource {
    url: String,
    width: i32,
    height: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RedditComment {
    id: String,
    body: String,
    author: String,
    score: i32,
    created_utc: f64,
    parent_id: String,
    permalink: String,
    replies: Option<RedditListing<RedditComment>>,
    depth: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditSubreddit {
    display_name: String,
    title: String,
    description: String,
    public_description: String,
    subscribers: i32,
    active_user_count: i32,
    created_utc: f64,
    url: String,
    over18: bool,
    lang: String,
    banner_img: String,
    icon_img: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditUser {
    name: String,
    created_utc: f64,
    link_karma: i32,
    comment_karma: i32,
    is_gold: bool,
    is_mod: bool,
    has_verified_email: bool,
    icon_img: String,
    subreddit: Option<RedditUserSubreddit>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditUserSubreddit {
    display_name: String,
    title: String,
    public_description: String,
    subscribers: i32,
    created_utc: f64,
}

struct RedditClient {
    client: Client,
    access_token: String,
}

impl RedditClient {
    async fn new() -> Result<Self> {
        let client_id = env::var("REDDIT_CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("REDDIT_CLIENT_ID not found in environment"))?;
        let client_secret = env::var("REDDIT_CLIENT_SECRET")
            .map_err(|_| anyhow::anyhow!("REDDIT_CLIENT_SECRET not found in environment"))?;
        let user_agent = env::var("REDDIT_USER_AGENT")
            .unwrap_or_else(|_| "MCP-Reddit-Server/1.0".to_string());

        let client = Client::new();
        
        // Get access token using client credentials flow
        let token_response = client
            .post(REDDIT_OAUTH_URL)
            .basic_auth(&client_id, Some(&client_secret))
            .form(&[("grant_type", "client_credentials")])
            .header("User-Agent", &user_agent)
            .send()
            .await?
            .json::<RedditToken>()
            .await?;

        Ok(RedditClient {
            client,
            access_token: token_response.access_token,
        })
    }

    async fn get_posts(&self, subreddit: &str, sort: &str, limit: i32) -> Result<Vec<RedditPost>> {
        let url = format!("{}/r/{}/{}", REDDIT_API_BASE, subreddit, sort);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json::<RedditListing<RedditPost>>()
            .await?;

        Ok(response.data.children.into_iter().map(|child| child.data).collect())
    }

    async fn search_posts(&self, query: &str, subreddit: Option<&str>, sort: &str, limit: i32) -> Result<Vec<RedditPost>> {
        let url = if let Some(sub) = subreddit {
            format!("{}/r/{}/search", REDDIT_API_BASE, sub)
        } else {
            format!("{}/search", REDDIT_API_BASE)
        };
        
        let query_params = vec![
            ("q", query.to_string()),
            ("sort", sort.to_string()),
            ("limit", limit.to_string()),
            ("type", "link".to_string()),
        ];

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .query(&query_params)
            .send()
            .await?
            .json::<RedditListing<RedditPost>>()
            .await?;

        Ok(response.data.children.into_iter().map(|child| child.data).collect())
    }

    async fn get_comments(&self, post_id: &str, limit: i32) -> Result<Vec<RedditComment>> {
        let url = format!("{}/comments/{}", REDDIT_API_BASE, post_id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json::<Vec<RedditListing<RedditComment>>>()
            .await?;

        // The first element is the post, second is comments
        if response.len() > 1 {
            Ok(response[1].data.children.iter().map(|child| child.data.clone()).collect())
        } else {
            Ok(vec![])
        }
    }

    async fn get_subreddit_info(&self, subreddit: &str) -> Result<RedditSubreddit> {
        let url = format!("{}/r/{}/about", REDDIT_API_BASE, subreddit);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .send()
            .await?
            .json::<RedditChild<RedditSubreddit>>()
            .await?;

        Ok(response.data)
    }

    async fn get_user_info(&self, username: &str) -> Result<RedditUser> {
        let url = format!("{}/user/{}/about", REDDIT_API_BASE, username);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .send()
            .await?
            .json::<RedditChild<RedditUser>>()
            .await?;

        Ok(response.data)
    }

    async fn get_trending_subreddits(&self, limit: i32) -> Result<Vec<RedditSubreddit>> {
        let url = format!("{}/subreddits/popular", REDDIT_API_BASE);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json::<RedditListing<RedditSubreddit>>()
            .await?;

        Ok(response.data.children.into_iter().map(|child| child.data).collect())
    }

    async fn get_user_posts(&self, username: &str, limit: i32) -> Result<Vec<RedditPost>> {
        let url = format!("{}/user/{}/submitted", REDDIT_API_BASE, username);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json::<RedditListing<RedditPost>>()
            .await?;

        Ok(response.data.children.into_iter().map(|child| child.data).collect())
    }

    async fn get_user_comments(&self, username: &str, limit: i32) -> Result<Vec<RedditComment>> {
        let url = format!("{}/user/{}/comments", REDDIT_API_BASE, username);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "MCP-Reddit-Server/1.0")
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json::<RedditListing<RedditComment>>()
            .await?;

        Ok(response.data.children.into_iter().map(|child| child.data).collect())
    }
}

pub fn build<T: Transport>(t: T) -> Result<Server<T>> {
    let mut server = Server::builder(t)
        .capabilities(ServerCapabilities {
            tools: Some(json!({})),
            ..Default::default()
        })
        .request_handler("resources/list", |_req: ListRequest| {
            Box::pin(async move { Ok(list_resources()) })
        })
        .request_handler("prompts/list", |_req: ListRequest| {
            Box::pin(async move {
                Ok(PromptsListResponse {
                    prompts: vec![],
                    next_cursor: None,
                    meta: None,
                })
            })
        });

    register_tools(&mut server)?;

    let server = server.build();
    Ok(server)
}

fn list_resources() -> ResourcesListResponse {
    let base = Url::parse("https://reddit.com/").unwrap();
    let resources = ["posts", "comments", "subreddits", "users"]
        .iter()
        .map(|r| Resource {
            uri: base.join(r).unwrap(),
            name: r.to_string(),
            description: None,
            mime_type: Some("application/json".to_string()),
        })
        .collect();
    ResourcesListResponse {
        resources,
        next_cursor: None,
        meta: None,
    }
}

fn register_tools<T: Transport>(server: &mut ServerBuilder<T>) -> Result<()> {
    // Get Posts Tool
    let get_posts_tool = Tool {
        name: "get_posts".to_string(),
        description: Some("Get posts from a subreddit".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "subreddit": {"type": "string", "description": "Subreddit name"},
                "sort": {"type": "string", "enum": ["hot", "new", "top", "rising"], "default": "hot", "description": "Sort order"},
                "limit": {"type": "integer", "default": 25, "description": "Number of posts to return"}
            },
            "required": ["subreddit"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "posts": {"type": "array", "items": {"type": "object"}}
            },
        })),
    };

    // Search Posts Tool
    let search_posts_tool = Tool {
        name: "search_posts".to_string(),
        description: Some("Search for posts on Reddit".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "subreddit": {"type": "string", "description": "Optional subreddit to search in"},
                "sort": {"type": "string", "enum": ["relevance", "hot", "top", "new", "comments"], "default": "relevance", "description": "Sort order"},
                "limit": {"type": "integer", "default": 25, "description": "Number of posts to return"}
            },
            "required": ["query"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "posts": {"type": "array", "items": {"type": "object"}}
            },
        })),
    };

    // Get Comments Tool
    let get_comments_tool = Tool {
        name: "get_comments".to_string(),
        description: Some("Get comments for a post".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "post_id": {"type": "string", "description": "Post ID"},
                "limit": {"type": "integer", "default": 25, "description": "Number of comments to return"}
            },
            "required": ["post_id"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "comments": {"type": "array", "items": {"type": "object"}}
            },
        })),
    };

    // Get Subreddit Info Tool
    let get_subreddit_info_tool = Tool {
        name: "get_subreddit_info".to_string(),
        description: Some("Get information about a subreddit".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "subreddit": {"type": "string", "description": "Subreddit name"}
            },
            "required": ["subreddit"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "subreddit": {"type": "object"}
            },
        })),
    };

    // Get User Info Tool
    let get_user_info_tool = Tool {
        name: "get_user_info".to_string(),
        description: Some("Get information about a Reddit user".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "username": {"type": "string", "description": "Reddit username"}
            },
            "required": ["username"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "user": {"type": "object"}
            },
        })),
    };

    // Get Trending Subreddits Tool
    let get_trending_subreddits_tool = Tool {
        name: "get_trending_subreddits".to_string(),
        description: Some("Get trending/popular subreddits".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "default": 25, "description": "Number of subreddits to return"}
            },
            "required": [],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "subreddits": {"type": "array", "items": {"type": "object"}}
            },
        })),
    };

    // Get User Posts Tool
    let get_user_posts_tool = Tool {
        name: "get_user_posts".to_string(),
        description: Some("Get posts submitted by a user".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "username": {"type": "string", "description": "Reddit username"},
                "limit": {"type": "integer", "default": 25, "description": "Number of posts to return"}
            },
            "required": ["username"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "posts": {"type": "array", "items": {"type": "object"}}
            },
        })),
    };

    // Get User Comments Tool
    let get_user_comments_tool = Tool {
        name: "get_user_comments".to_string(),
        description: Some("Get comments made by a user".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "username": {"type": "string", "description": "Reddit username"},
                "limit": {"type": "integer", "default": 25, "description": "Number of comments to return"}
            },
            "required": ["username"],
            "additionalProperties": false
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "comments": {"type": "array", "items": {"type": "object"}}
            },
        })),
    };

    // Register get_posts tool
    server.register_tool(get_posts_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let subreddit = args["subreddit"].as_str().context("subreddit is missing")?;
                let sort = args.get("sort").and_then(|v| v.as_str()).unwrap_or("hot");
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as i32;

                let posts = client.get_posts(subreddit, sort, limit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "posts": posts }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register search_posts tool
    server.register_tool(search_posts_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let query = args["query"].as_str().context("query is missing")?;
                let subreddit = args.get("subreddit").and_then(|v| v.as_str());
                let sort = args.get("sort").and_then(|v| v.as_str()).unwrap_or("relevance");
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as i32;

                let posts = client.search_posts(query, subreddit, sort, limit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "posts": posts }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register get_comments tool
    server.register_tool(get_comments_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let post_id = args["post_id"].as_str().context("post_id is missing")?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as i32;

                let comments = client.get_comments(post_id, limit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "comments": comments }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register get_subreddit_info tool
    server.register_tool(get_subreddit_info_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let subreddit = args["subreddit"].as_str().context("subreddit is missing")?;

                let subreddit_info = client.get_subreddit_info(subreddit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "subreddit": subreddit_info }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register get_user_info tool
    server.register_tool(get_user_info_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let username = args["username"].as_str().context("username is missing")?;

                let user_info = client.get_user_info(username).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "user": user_info }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register get_trending_subreddits tool
    server.register_tool(get_trending_subreddits_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as i32;

                let subreddits = client.get_trending_subreddits(limit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "subreddits": subreddits }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register get_user_posts tool
    server.register_tool(get_user_posts_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let username = args["username"].as_str().context("username is missing")?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as i32;

                let posts = client.get_user_posts(username, limit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "posts": posts }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    // Register get_user_comments tool
    server.register_tool(get_user_comments_tool, |req: CallToolRequest| {
        Box::pin(async move {
            let args = req.arguments.unwrap_or_default();
            let result: Result<CallToolResponse, anyhow::Error> = async {
                let client = RedditClient::new().await?;
                let username = args["username"].as_str().context("username is missing")?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as i32;

                let comments = client.get_user_comments(username, limit).await?;

                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text {
                        text: serde_json::to_string(&json!({ "comments": comments }))?,
                    }],
                    is_error: None,
                    meta: None,
                })
            }
            .await;

            match result {
                Ok(response) => Ok(response),
                Err(e) => {
                    info!("Error handling request: {:#?}", e);
                    Ok(CallToolResponse {
                        content: vec![ToolResponseContent::Text {
                            text: format!("{}", e),
                        }],
                        is_error: Some(true),
                        meta: None,
                    })
                }
            }
        })
    });

    Ok(())
}