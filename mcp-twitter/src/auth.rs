#[derive(Debug, Clone)]
pub struct TwitterAuth {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: Option<String>,
    pub access_token_secret: Option<String>,
    pub bearer_token: Option<String>,
}

impl TwitterAuth {
    pub fn new(
        api_key: String,
        api_secret: String,
        access_token: Option<String>,
        access_token_secret: Option<String>,
        bearer_token: Option<String>,
    ) -> Self {
        Self {
            api_key,
            api_secret,
            access_token,
            access_token_secret,
            bearer_token,
        }
    }

    pub fn from_env() -> Result<Self, std::env::VarError> {
        let api_key = std::env::var("TWITTER_API_KEY")?;
        let api_secret = std::env::var("TWITTER_API_SECRET")?;
        let access_token = std::env::var("TWITTER_ACCESS_TOKEN").ok();
        let access_token_secret = std::env::var("TWITTER_ACCESS_TOKEN_SECRET").ok();
        let bearer_token = std::env::var("TWITTER_BEARER_TOKEN").ok();

        Ok(Self::new(
            api_key,
            api_secret,
            access_token,
            access_token_secret,
            bearer_token,
        ))
    }

    pub fn has_oauth_credentials(&self) -> bool {
        self.access_token.is_some() && self.access_token_secret.is_some()
    }

    pub fn has_bearer_token(&self) -> bool {
        self.bearer_token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let auth = TwitterAuth::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            Some("test_token".to_string()),
            Some("test_token_secret".to_string()),
            Some("test_bearer".to_string()),
        );

        assert_eq!(auth.api_key, "test_key");
        assert_eq!(auth.api_secret, "test_secret");
        assert!(auth.has_oauth_credentials());
        assert!(auth.has_bearer_token());
    }

    #[test]
    fn test_auth_without_oauth() {
        let auth = TwitterAuth::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            None,
            None,
            Some("test_bearer".to_string()),
        );

        assert!(!auth.has_oauth_credentials());
        assert!(auth.has_bearer_token());
    }
}