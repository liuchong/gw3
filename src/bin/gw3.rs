#[tokio::main]
async fn main() {
    if let Err(error) = gw3::cli::run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
