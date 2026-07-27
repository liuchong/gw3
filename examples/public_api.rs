use gw3::api::{ApiClient, ApiRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApiClient::from_env()?;

    let build = client.get_json(ApiRequest::new("/v2/build")).await?;
    println!("Current build:\n{}", serde_json::to_string_pretty(&build)?);

    let items = client
        .item_lookup(["19684", "19721"], Some("zh".to_string()))
        .await?;
    println!("\nItems:\n{}", serde_json::to_string_pretty(&items)?);

    let prices = client.item_prices(["19684"]).await?;
    println!(
        "\nTrading post prices:\n{}",
        serde_json::to_string_pretty(&prices)?
    );

    Ok(())
}
