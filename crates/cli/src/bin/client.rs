// Client-only binary — thin wrapper around `opencode client`
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut client_args = vec!["opencode-client".to_string(), "client".to_string()];
    client_args.extend(args.into_iter().skip(1));
    opencode_r_cli::main_entry(client_args).await
}
