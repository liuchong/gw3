use gw3::wiki::WikiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let query = if query.is_empty() {
        "Legendary armor".to_string()
    } else {
        query
    };

    let client = WikiClient::from_env()?;
    let search = client.search(&query).await?;

    println!("{}", serde_json::to_string_pretty(&search)?);

    Ok(())
}
