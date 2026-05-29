use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::models::{ConversationResponse, PinnedChannelsResponse};

use super::HttpClient;

const CSA_BASE: &str = "https://teams.microsoft.com/api/csa/api/v1";

pub struct CsaClient<'a> {
    http: &'a HttpClient,
    tokens: &'a TokenSet,
}

impl<'a> CsaClient<'a> {
    pub fn new(http: &'a HttpClient, tokens: &'a TokenSet) -> Self {
        Self { http, tokens }
    }

    pub async fn get_conversations(&self) -> Result<ConversationResponse> {
        let url =
            format!("{CSA_BASE}/teams/users/me?isPrefetch=false&enableMembershipSummary=true");
        let bearer = self.tokens.chatsvcagg_bearer();

        let resp = self
            .http
            .execute_with_retry(|| self.http.client.get(&url).header("Authorization", &bearer))
            .await?;

        resp.json::<ConversationResponse>()
            .await
            .map_err(|e| crate::error::TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse conversations: {e}"),
            })
    }

    /// Create a new 1:1 chat via the CSA endpoint.
    /// Returns the thread ID.
    pub async fn create_chat(&self, my_mri: &str, target_mri: &str) -> Result<String> {
        let url = format!("{CSA_BASE}/chats");
        let bearer = self.tokens.chatsvcagg_bearer();

        let body = serde_json::json!({
            "members": [
                {"id": my_mri, "role": "Admin"},
                {"id": target_mri, "role": "User"}
            ],
            "properties": {
                "threadType": "chat",
                "fixedRoster": "true",
                "uniquerosterthread": "true"
            }
        });

        let resp = self
            .http
            .execute_with_retry(|| {
                self.http
                    .client
                    .post(&url)
                    .header("Authorization", &bearer)
                    .json(&body)
            })
            .await?;

        let data: serde_json::Value =
            resp.json()
                .await
                .map_err(|e| crate::error::TeamsError::ApiError {
                    status: 0,
                    message: format!("failed to parse create chat response: {e}"),
                })?;

        // Try common response fields for the thread ID
        data.get("id")
            .or_else(|| data.get("threadId"))
            .or_else(|| data.get("chatId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| crate::error::TeamsError::ApiError {
                status: 0,
                message: format!(
                    "chat created but no thread ID in response: {}",
                    serde_json::to_string(&data).unwrap_or_default()
                ),
            })
    }

    pub async fn get_pinned_channels(&self) -> Result<PinnedChannelsResponse> {
        let url = format!("{CSA_BASE}/teams/users/me/pinnedChannels");
        let bearer = self.tokens.chatsvcagg_bearer();

        let resp = self
            .http
            .execute_with_retry(|| self.http.client.get(&url).header("Authorization", &bearer))
            .await?;

        resp.json::<PinnedChannelsResponse>().await.map_err(|e| {
            crate::error::TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse pinned channels: {e}"),
            }
        })
    }
}
