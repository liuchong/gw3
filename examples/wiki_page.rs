use gw3::wiki::WikiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let title = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let title = if title.is_empty() {
        "Legendary armor".to_string()
    } else {
        title
    };

    let client = WikiClient::from_env()?;
    let page = client.page(&title).await?;

    println!("{}", serde_json::to_string_pretty(&page)?);

    Ok(())
}
