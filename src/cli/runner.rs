use super::args::{
    AccountCommands, ApiCommands, CharacterCommands, Cli, Commands, ItemCommands, TokenCommands,
    WikiCommands,
};
use super::output::{join_query, print_json};
use crate::api::{ApiClient, ApiRequest};
use crate::config::ClientConfig;
use crate::error::Gw3Error;
use crate::wiki::WikiClient;
use clap::Parser;

pub async fn run() -> Result<(), Gw3Error> {
    run_from(Cli::parse()).await
}

async fn run_from(cli: Cli) -> Result<(), Gw3Error> {
    if let Commands::Mcp { command: _ } = &cli.command {
        crate::mcp::serve_stdio()
            .await
            .map_err(|error| Gw3Error::Runtime(error.to_string()))?;
        return Ok(());
    }

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
            ItemCommands::Get(args) => api_client.item_lookup(args.ids, args.lang).await?,
            ItemCommands::Prices(args) => api_client.item_prices(args.ids).await?,
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
        Commands::Mcp { command: _ } => unreachable!("mcp commands return before API clients run"),
    };

    print_json(&value);
    Ok(())
}
