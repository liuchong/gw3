#[tokio::main]
async fn main() {
    if let Err(error) = gw3::mcp::serve_stdio().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
