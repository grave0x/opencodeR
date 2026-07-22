// opencodeR-server — server-only binary
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut server_args = vec!["opencodeR-server".to_string(), "server".to_string()];
    server_args.extend(std::env::args().skip(1));
    opencode_r_cli::main_entry(server_args).await
}
