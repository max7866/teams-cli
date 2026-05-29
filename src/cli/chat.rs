use clap::{Args, Subcommand};
use std::time::Instant;

use crate::api::csa::CsaClient;
use crate::api::messages::MessagesClient;
use crate::api::mt::MtClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct ChatArgs {
    #[command(subcommand)]
    pub command: ChatCommand,
}

#[derive(Subcommand)]
pub enum ChatCommand {
    /// List direct and group chats
    List {
        /// Show hidden chats too
        #[arg(long)]
        all: bool,
    },
    /// Get chat details
    Get {
        /// Chat ID
        chat_id: String,
    },
    /// Create a new 1:1 chat with a user (by email or MRI)
    Create {
        /// User email address or MRI (e.g. 8:orgid:...)
        user: String,
    },
}

/// Context needed for chat commands that require messaging tokens
pub struct ChatContext<'a> {
    pub tokens: &'a TokenSet,
    pub messaging_token: &'a str,
    pub http: &'a HttpClient,
    pub chat_service_url: &'a str,
    pub mt_url: &'a str,
}

pub async fn handle(
    args: &ChatArgs,
    tokens: &TokenSet,
    http: &HttpClient,
    format: OutputFormat,
) -> Result<()> {
    let csa = CsaClient::new(http, tokens);

    match &args.command {
        ChatCommand::List { all } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let chats: Vec<serde_json::Value> = conversations
                .chats
                .iter()
                .filter(|c| {
                    *all || !c
                        .get("hidden")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .map(|c| {
                    let title = c
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    serde_json::json!({
                        "id": c.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "title": title,
                        "chat_type": c.get("chatType").and_then(|v| v.as_str()).unwrap_or(""),
                        "members": c.get("members").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
                    })
                })
                .collect();
            output::print_output(format, chats, start.elapsed().as_millis() as u64);
        }
        ChatCommand::Get { chat_id } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let chat = conversations
                .chats
                .iter()
                .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(chat_id.as_str()))
                .ok_or_else(|| crate::error::TeamsError::NotFound(format!("chat {chat_id}")))?;
            output::print_output(format, chat, start.elapsed().as_millis() as u64);
        }
        ChatCommand::Create { .. } => {
            return Err(crate::error::TeamsError::InvalidInput(
                "chat create requires messaging context; this is handled in main.rs".into(),
            ));
        }
    }
    Ok(())
}

/// Handle chat create with full messaging context (needs authz tokens).
pub async fn handle_create(
    user: &str,
    ctx: &ChatContext<'_>,
    format: OutputFormat,
) -> Result<()> {
    let start = Instant::now();

    // Resolve email to MRI if needed
    let mri = if user.starts_with("8:") {
        user.to_string()
    } else {
        let mt = MtClient::new(ctx.http, ctx.tokens, ctx.mt_url);
        let user_info = mt.get_user(user).await?;
        if user_info.mri.is_empty() {
            return Err(crate::error::TeamsError::NotFound(format!(
                "MRI for user {user}"
            )));
        }
        user_info.mri
    };

    let msg_client = MessagesClient::new(ctx.http, ctx.messaging_token, ctx.chat_service_url);
    let thread_id = msg_client.create_conversation(&mri).await?;

    let result = serde_json::json!({
        "thread_id": thread_id,
        "user": user,
        "mri": mri,
    });
    output::print_output(format, result, start.elapsed().as_millis() as u64);
    Ok(())
}
