// opencodeR — combined binary (classic interactive TUI by default)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    opencode_r_cli::main_entry(std::env::args().collect()).await
}
