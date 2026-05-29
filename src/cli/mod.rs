pub mod auth;
pub mod calendar;
pub mod channel;
pub mod chat;
pub mod config_cmd;
pub mod mail;
pub mod message;
pub mod presence;
pub mod team;
pub mod tenant;
pub mod user;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "teams",
    about = "CLI for Microsoft Teams (using internal Skype/CSA APIs)",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format: json, human, plain (auto-detect from TTY)
    #[arg(long, global = true)]
    pub output: Option<String>,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(long, short, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Disable ANSI color codes
    #[arg(long, global = true)]
    pub no_color: bool,

    /// API region (emea, amer, apac)
    #[arg(long, global = true, default_value = "emea")]
    pub region: String,

    /// Named credential profile
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,

    /// Request timeout in seconds
    #[arg(long, global = true)]
    pub timeout: Option<u64>,

    /// Max retry attempts
    #[arg(long, global = true)]
    pub retry: Option<u32>,

    /// Disable automatic login when tokens are missing or expired
    #[arg(long = "no-auto-login", global = true)]
    pub no_auto_login: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authentication and credential management
    Auth(auth::AuthArgs),
    /// User profile and directory lookup
    User(user::UserArgs),
    /// Team management
    Team(team::TeamArgs),
    /// Channel operations
    Channel(channel::ChannelArgs),
    /// Direct and group chats
    Chat(chat::ChatArgs),
    /// Message operations
    Message(message::MessageArgs),
    /// Tenant information
    Tenant(tenant::TenantArgs),
    /// Email operations (Outlook)
    Mail(mail::MailArgs),
    /// Calendar operations (Outlook)
    Calendar(calendar::CalendarArgs),
    /// Presence and availability status
    Presence(presence::PresenceArgs),
    /// Configuration management
    Config(config_cmd::ConfigArgs),
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
