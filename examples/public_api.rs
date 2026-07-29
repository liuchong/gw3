use gw3::api::ApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApiClient::from_env()?;

    let routes = client.public_routes().await?;
    println!(
        "Public route registry:\n{}",
        serde_json::to_string_pretty(&routes)?
    );

    let build = client
        .public_get("build", None, Vec::<String>::new(), None, None)
        .await?;
    println!(
        "\nCurrent build:\n{}",
        serde_json::to_string_pretty(&build)?
    );

    let skins = client
        .public_get(
            "skins",
            None,
            vec!["4674".to_string()],
            Some("zh".to_string()),
            None,
        )
        .await?;
    println!("\nSkins:\n{}", serde_json::to_string_pretty(&skins)?);

    let prices = client
        .public_get(
            "commerce_prices",
            None,
            vec!["19684".to_string()],
            None,
            None,
        )
        .await?;
    println!(
        "\nTrading post prices:\n{}",
        serde_json::to_string_pretty(&prices)?
    );

    let recipes = client
        .public_call(
            "recipes_search",
            Vec::<(String, String)>::new(),
            [("input".to_string(), "46747".to_string())],
            None,
            None,
        )
        .await?;
    println!(
        "\nRecipes by ingredient:\n{}",
        serde_json::to_string_pretty(&recipes)?
    );

    Ok(())
}
