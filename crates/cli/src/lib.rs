use std::net::SocketAddr;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "opencodeR", version, about = "OpenCode AI coding agent (Rust)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, default_value = "8081", env = "OPENCODE_PORT")]
    pub port: u16,

    #[arg(short = 'P', long, env = "OPENCODE_PASSWORD")]
    pub password: Option<String>,

    #[arg(short = 'u', long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
    pub base_url: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP server
    Server,
    /// Run the CLI client
    Client {
        #[arg(short, long)]
        session: Option<String>,
        prompt: Vec<String>,
    },
}

pub async fn main_entry(args: Vec<String>) -> anyhow::Result<()> {
    // Only init tracing once
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("opencode_r_server=info".parse().unwrap())
            .add_directive("opencode_r_client=info".parse().unwrap()))
        .try_init();

    let cli = Cli::parse_from(args);

    match cli.command.unwrap_or(Commands::Server) {
        Commands::Server => run_server(cli.port, cli.password).await,
        Commands::Client { session, prompt } => run_client(cli.base_url, session, prompt).await,
    }
}

async fn run_server(port: u16, password: Option<String>) -> anyhow::Result<()> {
    if let Some(pwd) = password {
        opencode_r_server::middleware::auth::set_password(pwd);
        tracing::info!("Auth enabled (password-based)");
    }

    let state = Arc::new(opencode_r_server::state::AppState::new());
    let app = opencode_r_server::build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("OpenCodeR server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_client(base_url: String, session_id: Option<String>, prompts: Vec<String>) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();

    let resp = client.get(format!("{}/api/health", base_url)).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Server not reachable at {}", base_url);
    }

    let sid = match session_id {
        Some(id) => id,
        None => {
            let resp = client.post(format!("{}/api/session", base_url))
                .json(&serde_json::json!({"agent": "build"}))
                .send().await?;
            let body: serde_json::Value = resp.json().await?;
            let id = body["data"]["id"].as_str().unwrap().to_string();
            println!("Created session: {}", id);
            id
        }
    };

    if prompts.is_empty() {
        let resp = client.get(format!("{}/api/session/{}", base_url, sid)).send().await?;
        let body: serde_json::Value = resp.json().await?;
        println!("Session: {} | Agent: {}",
            body["data"]["id"].as_str().unwrap_or("?"),
            body["data"]["agent"].as_str().unwrap_or("?")
        );
        let resp = client.get(format!("{}/api/session/{}/message", base_url, sid)).send().await?;
        let body: serde_json::Value = resp.json().await?;
        if let Some(msgs) = body["data"].as_array() {
            for msg in msgs {
                let role = msg["role"].as_str().unwrap_or("?");
                let text = msg["content"].as_array()
                    .and_then(|c| c.first())
                    .and_then(|c| c["text"].as_str())
                    .unwrap_or("...");
                println!("[{}] {}", role, text);
            }
        }
    } else {
        for prompt in &prompts {
            println!(">>> {}", prompt);
            let resp = client.post(format!("{}/api/session/{}/prompt", base_url, sid))
                .json(&serde_json::json!({"prompt": prompt, "resume": false}))
                .send().await?;
            let body: serde_json::Value = resp.json().await?;
            println!("<<< {:?}", body["data"]);
        }
    }
    Ok(())
}
