mod cli;
mod client;
mod crypto;
mod decoder;
mod m3u8;
mod player;
mod tui;
mod types;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::run_cli().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
