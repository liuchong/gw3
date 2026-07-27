use crate::api::{ApiClient, ApiRequest, ClientConfig, Gw3Error};
use crate::wiki::WikiClient;
use clap::{Args, Parser, Subcommand};
use serde_json::Value;

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
    api_base_url: String,
    #[arg(
        long,
        env = "GW3_WIKI_BASE_URL",
        default_value = "https://wiki.guildwars2.com/api.php"
    )]
    wiki_base_url: String,
    #[arg(long, env = "GW2_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
    #[arg(long)]
    lang: Option<String>,
    #[arg(long)]
    schema_version: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
enum ApiCommands {
    Routes,
    Get(ApiGetArgs),
}

#[derive(Debug, Args)]
struct ApiGetArgs {
    path: String,
    #[arg(long)]
    id: Option<String>,
    #[arg(long, value_delimiter = ',')]
    ids: Vec<String>,
    #[arg(long)]
    page: Option<u32>,
    #[arg(long)]
    page_size: Option<u32>,
    #[arg(long)]
    lang: Option<String>,
    #[arg(long)]
    schema_version: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
}

#[derive(Debug, Subcommand)]
enum TokenCommands {
    Info,
}

#[derive(Debug, Subcommand)]
enum ItemCommands {
    Get(IdsArgs),
    Prices(IdsArgs),
}

#[derive(Debug, Args)]
struct IdsArgs {
    #[arg(long, value_delimiter = ',')]
    ids: Vec<String>,
    #[arg(long)]
    lang: Option<String>,
}

#[derive(Debug, Subcommand)]
enum AccountCommands {
    Info,
}

#[derive(Debug, Subcommand)]
enum CharacterCommands {
    List,
}

#[derive(Debug, Subcommand)]
enum WikiCommands {
    Search(QueryArgs),
    Page(QueryArgs),
}

#[derive(Debug, Args)]
struct QueryArgs {
    query: Vec<String>,
}

pub async fn run() -> Result<(), Gw3Error> {
    run_from(Cli::parse()).await
}

async fn run_from(cli: Cli) -> Result<(), Gw3Error> {
    let api_client = ApiClient::new(ClientConfig {
        base_url: cli.api_base_url.clone(),
        wiki_base_url: cli.wiki_base_url.clone(),
        lang: cli.lang.clone(),
        schema_version: cli.schema_version.clone(),
        api_key: cli.api_key.clone(),
        ..ClientConfig::default()
    })?;
    let wiki_client = WikiClient::new(cli.wiki_base_url.clone())?;

    let value = match cli.command {
        Commands::Api { command } => match command {
            ApiCommands::Routes => api_client.routes().await?,
            ApiCommands::Get(args) => {
                let api_key = args.api_key.or(cli.api_key);
                let client = ApiClient::new(ClientConfig {
                    base_url: cli.api_base_url,
                    wiki_base_url: cli.wiki_base_url,
                    lang: cli.lang,
                    schema_version: cli.schema_version,
                    api_key,
                    ..ClientConfig::default()
                })?;
                let mut request = ApiRequest::new(args.path)
                    .with_ids(args.ids)
                    .with_lang(args.lang)
                    .with_schema_version(args.schema_version)
                    .with_page(args.page, args.page_size);
                if let Some(id) = args.id {
                    request = request.with_id(id);
                }
                client.get_json(request).await?
            }
        },
        Commands::Token { command } => match command {
            TokenCommands::Info => api_client.token_info().await?,
        },
        Commands::Item { command } => match command {
            ItemCommands::Get(args) => {
                require_ids(api_client.item_lookup(args.ids, args.lang).await?)
            }
            ItemCommands::Prices(args) => require_ids(api_client.item_prices(args.ids).await?),
        },
        Commands::Account { command } => match command {
            AccountCommands::Info => api_client.account_info().await?,
        },
        Commands::Character { command } => match command {
            CharacterCommands::List => api_client.character_list().await?,
        },
        Commands::Wiki { command } => match command {
            WikiCommands::Search(args) => wiki_client.search(&join_query(args.query)).await?,
            WikiCommands::Page(args) => wiki_client.page(&join_query(args.query)).await?,
        },
    };

    print_json(&value);
    Ok(())
}

fn require_ids(value: Value) -> Value {
    value
}

fn join_query(query: Vec<String>) -> String {
    query.join(" ")
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(output) => println!("{output}"),
        Err(_) => println!("{value}"),
    }
}
