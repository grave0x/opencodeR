// opencodeR-server — server binary (server TUI by default, --headless for headless)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--headless");
    let filtered: Vec<String> = args.into_iter()
        .filter(|a| a != "--headless")
        .collect();
    opencode_r_cli::server_entry(filtered, headless).await
}
