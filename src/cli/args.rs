use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "gw3",
    version,
    about = "Guild Wars 2 official API and Wiki CLI"
)]
pub struct Cli {
    #[arg(
        long,
        env = "GW3_API_BASE_URL",
        default_value = "https://api.guildwars2.com"
    )]
    pub(super) api_base_url: String,
    #[arg(
        long,
        env = "GW3_WIKI_BASE_URL",
        default_value = "https://wiki.guildwars2.com/api.php"
    )]
    pub(super) wiki_base_url: String,
    #[arg(long, env = "GW2_API_KEY", hide_env_values = true)]
    pub(super) api_key: Option<String>,
    #[arg(long)]
    pub(super) lang: Option<String>,
    #[arg(long)]
    pub(super) schema_version: Option<String>,
    #[command(subcommand)]
    pub(super) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(super) enum Commands {
    Api {
        #[command(subcommand)]
        command: ApiCommands,
    },
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },
    Item {
        #[command(subcommand)]
        command: ItemCommands,
    },
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },
    Character {
        #[command(subcommand)]
        command: CharacterCommands,
    },
    Wiki {
        #[command(subcommand)]
        command: WikiCommands,
    },
}

#[derive(Debug, Subcommand)]
pub(super) enum ApiCommands {
    Routes,
    Get(ApiGetArgs),
}

#[derive(Debug, Args)]
pub(super) struct ApiGetArgs {
    pub(super) path: String,
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub(super) ids: Vec<String>,
    #[arg(long)]
    pub(super) page: Option<u32>,
    #[arg(long)]
    pub(super) page_size: Option<u32>,
    #[arg(long)]
    pub(super) lang: Option<String>,
    #[arg(long)]
    pub(super) schema_version: Option<String>,
    #[arg(long)]
    pub(super) api_key: Option<String>,
}

#[derive(Debug, Subcommand)]
pub(super) enum TokenCommands {
    Info,
}

#[derive(Debug, Subcommand)]
pub(super) enum ItemCommands {
    Get(IdsArgs),
    Prices(IdsArgs),
}

#[derive(Debug, Args)]
pub(super) struct IdsArgs {
    #[arg(long, value_delimiter = ',')]
    pub(super) ids: Vec<String>,
    #[arg(long)]
    pub(super) lang: Option<String>,
}

#[derive(Debug, Subcommand)]
pub(super) enum AccountCommands {
    Info,
}

#[derive(Debug, Subcommand)]
pub(super) enum CharacterCommands {
    List,
}

#[derive(Debug, Subcommand)]
pub(super) enum WikiCommands {
    Search(QueryArgs),
    Page(QueryArgs),
}

#[derive(Debug, Args)]
pub(super) struct QueryArgs {
    pub(super) query: Vec<String>,
}
