pub mod tui;

use std::net::SocketAddr;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use crate::tui::{LogBuffer, TuiLogLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::sync::Mutex;

// ── Shared CLI definition ──────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "opencodeR", version, about = "OpenCode AI coding agent (Rust)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the headless HTTP server
    Server {
        #[arg(short, long, default_value = "8081", env = "OPENCODE_PORT")]
        port: u16,
        #[arg(short = 'P', long, env = "OPENCODE_PASSWORD")]
        password: Option<String>,
    },
    /// Connect to a remote server interactively
    Client {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
        #[arg(short, long)]
        session: Option<String>,
        #[arg(short, long)]
        one_shot: Option<String>,
    },
    /// Run a one-shot prompt and exit
    Run {
        prompt: Vec<String>,
        #[arg(short, long, default_value = "8081", env = "OPENCODE_PORT")]
        port: u16,
        #[arg(short = 'P', long, env = "OPENCODE_PASSWORD")]
        password: Option<String>,
    },
    /// Launch the server dashboard TUI
    Tui {
        #[arg(short, long, default_value = "8081", env = "OPENCODE_PORT")]
        port: u16,
        #[arg(short = 'P', long, env = "OPENCODE_PASSWORD")]
        password: Option<String>,
    },
    /// List sessions
    Sessions {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// List available models
    Models {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
        #[arg(short, long)]
        provider: Option<String>,
    },
    /// List configured providers
    Providers {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
    },
    /// List available agents
    Agents {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
    },
    /// Export a session as JSON
    Export {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
        session_id: String,
    },
    /// Import session data from JSON file
    Import {
        /// Path to JSON file to import
        file: String,
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
    },
    /// Show cost and token usage summary
    Cost {
        #[arg(short, long, default_value = "http://127.0.0.1:8081", env = "OPENCODE_BASE_URL")]
        base_url: String,
    },
    /// Generate shell completion script
    Completion {
        /// Shell to generate completion for (bash, zsh, fish, powershell, elvish)
        shell: String,
    },
}

// ── Combined binary entry (opencodeR) ───────────────────────────────────

pub async fn main_entry(args: Vec<String>) -> anyhow::Result<()> {
    let cli = Cli::parse_from(args);

    match cli.command {
        None => {
            // Default: classic interactive TUI
            init_tracing(false, None);
            run_interactive(8081, None, None).await
        }
        Some(Commands::Server { port, password }) => {
            init_tracing(false, None);
            run_server(port, password).await
        }
        Some(Commands::Client { base_url, session, one_shot }) => {
            init_tracing(false, None);
            run_client(base_url, session, one_shot).await
        }
        Some(Commands::Run { prompt, port, password }) => {
            init_tracing(false, None);
            let prompt = Commands::Run { prompt, port, password: password.clone() };
            run_interactive(port, password, Some(prompt)).await
        }
        Some(Commands::Tui { port, password }) => {
            let log_buffer = LogBuffer::new();
            init_tracing(true, Some(log_buffer.clone()));
            run_tui_with_server(port, password, log_buffer).await
        }
        Some(Commands::Sessions { base_url, limit }) => {
            init_tracing(false, None);
            cmd_sessions(base_url, limit).await
        }
        Some(Commands::Models { base_url, provider }) => {
            init_tracing(false, None);
            cmd_models(base_url, provider).await
        }
        Some(Commands::Providers { base_url }) => {
            init_tracing(false, None);
            cmd_providers(base_url).await
        }
        Some(Commands::Agents { base_url }) => {
            init_tracing(false, None);
            cmd_agents(base_url).await
        }
        Some(Commands::Export { base_url, session_id }) => {
            init_tracing(false, None);
            cmd_export(base_url, session_id).await
        }
        Some(Commands::Completion { shell }) => {
            cmd_completion(shell).await
        }
        Some(Commands::Import { file, base_url }) => {
            init_tracing(false, None);
            cmd_import(base_url, file).await
        }
        Some(Commands::Cost { base_url }) => {
            init_tracing(false, None);
            cmd_cost(base_url).await
        }
    }
}

// ── Server binary entry (opencodeR-server) ──────────────────────────────

pub async fn server_entry(args: Vec<String>, headless: bool) -> anyhow::Result<()> {
    let port = parse_port(&args, "8081");
    let password = parse_password(&args);
    if headless {
        init_tracing(false, None);
        run_server(port, password).await
    } else {
        let log_buffer = LogBuffer::new();
        init_tracing(true, Some(log_buffer.clone()));
        run_tui_with_server(port, password, log_buffer).await
    }
}

fn parse_port(args: &[String], default: &str) -> u16 {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == "--port" || args[i] == "-p" {
            return args[i + 1].parse().unwrap_or_else(|_| default.parse().unwrap());
        }
    }
    default.parse().unwrap()
}

fn parse_password(args: &[String]) -> Option<String> {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == "--password" || args[i] == "-P" {
            let val = args[i + 1].clone();
            return if val.is_empty() { None } else { Some(val) };
        }
    }
    None
}

// ── Client binary entry (opencodeR-client) ──────────────────────────────
// (handled by main_entry with injected "client" subcommand)

// ── Tracing init ────────────────────────────────────────────────────────

fn init_tracing(tui_mode: bool, log_buffer: Option<Arc<Mutex<LogBuffer>>>) {
    let filter = EnvFilter::from_default_env()
        .add_directive("opencode_r_server=info".parse().unwrap())
        .add_directive("opencode_r_core=info".parse().unwrap())
        .add_directive("opencode_r_client=info".parse().unwrap())
        .add_directive("tower_http=warn".parse().unwrap());

    if tui_mode {
        let layer = TuiLogLayer { buffer: log_buffer.unwrap() };
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .try_init();
    }
}

// ── Classic interactive TUI (opencodeR default) ─────────────────────────

async fn run_interactive(port: u16, password: Option<String>, cmd: Option<Commands>) -> anyhow::Result<()> {
    // Start an in-process server
    if let Some(pwd) = password {
        opencode_r_server::middleware::auth::set_password(pwd);
    }
    let state = Arc::new(opencode_r_server::state::AppState::new());
    let app = opencode_r_server::build_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // If --run was passed, execute one-shot and exit
    if let Some(Commands::Run { prompt, .. }) = cmd {
        let msg = prompt.join(" ");
        if !msg.is_empty() {
            run_client_inner(&format!("http://127.0.0.1:{}", port), None, Some(msg)).await?;
        }
        return Ok(());
    }

    // Interactive REPL: read prompts from stdin, print responses
    println!("OpenCodeR interactive mode. Type /help for commands, /quit to exit.");
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create a session
    let client = reqwest::Client::new();
    let resp = client.post(format!("{}/api/session", base_url))
        .json(&serde_json::json!({"agent": "build"}))
        .send().await?;
    let body: serde_json::Value = resp.json().await?;
    let sid = body["data"]["id"].as_str().unwrap_or("?").to_string();
    println!("Session: {}", &sid[..16]);

    loop {
        let mut input = String::new();
        print!("\n> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        if std::io::stdin().read_line(&mut input)? == 0 { break; }
        let input = input.trim().to_string();
        if input.is_empty() { continue; }
        if input == "/quit" || input == "/q" || input == "exit" { break; }
        if input == "/help" {
            println!("Commands: /quit, /q, exit — exit");
            println!("          /help           — this help");
            println!("          anything else   — send as prompt");
            continue;
        }

        let resp = client.post(format!("{}/api/session/{}/prompt", base_url, sid))
            .json(&serde_json::json!({"prompt": input, "resume": false}))
            .send().await;
        match resp {
            Ok(r) => {
                let body: serde_json::Value = r.json().await.unwrap_or_default();
                if let Some(id) = body["data"]["id"].as_str() {
                    println!("✓ admitted (msg: {})", &id[..16]);
                } else {
                    println!("✓ {:?}", body["data"]);
                }
            }
            Err(e) => println!("✗ error: {}", e),
        }
    }

    Ok(())
}

// ── Server ──────────────────────────────────────────────────────────────

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

// ── Server TUI dashboard ────────────────────────────────────────────────

async fn run_tui_with_server(port: u16, password: Option<String>, log_buffer: Arc<Mutex<LogBuffer>>) -> anyhow::Result<()> {
    tracing::info!("Starting TUI mode — server on port {}", port);
    if let Some(pwd) = password {
        opencode_r_server::middleware::auth::set_password(pwd);
    }
    let state = Arc::new(opencode_r_server::state::AppState::new());
    let app = opencode_r_server::build_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tracing::info!("OpenCodeR server started. Launching TUI...");
    crate::tui::run_tui(port, None, log_buffer).await?;
    Ok(())
}

// ── Client (remote) ─────────────────────────────────────────────────────

async fn run_client(base_url: String, session: Option<String>, one_shot: Option<String>) -> anyhow::Result<()> {
    run_client_inner(&base_url, session, one_shot).await
}

async fn run_client_inner(base_url: &str, session: Option<String>, one_shot: Option<String>) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();

    // Health check
    let resp = client.get(format!("{}/api/health", base_url)).send().await;
    if resp.is_err() {
        anyhow::bail!("Cannot connect to server at {}", base_url);
    }

    // Get or create session
    let sid = match session {
        Some(id) => id,
        None => {
            let resp = client.post(format!("{}/api/session", base_url))
                .json(&serde_json::json!({"agent": "build"}))
                .send().await?;
            let body: serde_json::Value = resp.json().await?;
            body["data"]["id"].as_str().unwrap_or("?").to_string()
        }
    };

    // One-shot mode
    if let Some(msg) = one_shot {
        let resp = client.post(format!("{}/api/session/{}/prompt", base_url, sid))
            .json(&serde_json::json!({"prompt": msg, "resume": false}))
            .send().await?;
        let body: serde_json::Value = resp.json().await?;
        println!("{}", serde_json::to_string_pretty(&body["data"]).unwrap_or_default());
        return Ok(());
    }

    // Interactive remote REPL
    println!("Connected to {}. Type /quit to exit.", base_url);

    // Show session info
    let resp = client.get(format!("{}/api/session/{}", base_url, sid)).send().await?;
    if let Ok(body) = resp.json::<serde_json::Value>().await {
        println!("Session: {} | Agent: {}",
            body["data"]["id"].as_str().unwrap_or("?").chars().take(16).collect::<String>(),
            body["data"]["agent"].as_str().unwrap_or("?"));
    }

    loop {
        let mut input = String::new();
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        if std::io::stdin().read_line(&mut input)? == 0 { break; }
        let input = input.trim().to_string();
        if input.is_empty() { continue; }
        if input == "/quit" || input == "/q" || input == "exit" { break; }

        match client.post(format!("{}/api/session/{}/prompt", base_url, sid))
            .json(&serde_json::json!({"prompt": input, "resume": false}))
            .send().await
        {
            Ok(r) => {
                if let Ok(body) = r.json::<serde_json::Value>().await {
                    println!("✓ {}", serde_json::to_string_pretty(&body["data"]).unwrap_or_default());
                }
            }
            Err(e) => println!("✗ {}", e),
        }
    }

    Ok(())
}

// ── CLI subcommand handlers ─────────────────────────────────────────────

async fn cmd_sessions(base_url: String, limit: Option<u32>) -> anyhow::Result<()> {

    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();
    let url = match limit {
        Some(l) => format!("{}/api/session?limit={}", base_url, l),
        None => format!("{}/api/session", base_url),
    };
    let resp = client.get(&url).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let Some(sessions) = body["data"].as_array() else {
        println!("No sessions found.");
        return Ok(());
    };
    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }
    println!("Sessions ({}):", sessions.len());
    for s in sessions {
        let id = s["id"].as_str().unwrap_or("?");
        let agent = s["agent"].as_str().unwrap_or("?");
        let model = s["model"].as_str().unwrap_or("?");
        let title = s["title"].as_str().unwrap_or("Untitled");
        let created = s["time"]["created"].as_str().unwrap_or("?");
        let id_short = if id.len() > 16 { &id[..16] } else { id };
        let created_short = if created.len() > 19 { &created[..19] } else { created };
        println!("  {}  {}  {}  {}  {}", id_short, agent, model, title, created_short);
    }
    Ok(())
}

async fn cmd_models(base_url: String, provider: Option<String>) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();
    let resp = client.get(format!("{}/api/model", base_url)).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let Some(models) = body["data"].as_array() else {
        println!("No models found.");
        return Ok(());
    };
    let filtered: Vec<&serde_json::Value> = match &provider {
        Some(p) => models.iter().filter(|m| m["provider_id"].as_str() == Some(p.as_str())).collect(),
        None => models.iter().collect(),
    };
    if filtered.is_empty() {
        println!("No models found.");
        return Ok(());
    }
    println!("Models:");
    for m in filtered {
        let id = m["id"].as_str().unwrap_or("?");
        let name = m["name"].as_str().unwrap_or("?");
        let prov = m["provider_id"].as_str().unwrap_or("?");
        println!("  {}  ({})  {}", id, prov, name);
    }
    Ok(())
}

async fn cmd_providers(base_url: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();
    let resp = client.get(format!("{}/api/provider", base_url)).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let Some(providers) = body["data"].as_array() else {
        println!("No providers configured.");
        return Ok(());
    };
    if providers.is_empty() {
        println!("No providers configured.");
        return Ok(());
    }
    println!("Providers:");
    for p in providers {
        let id = p["id"].as_str().unwrap_or("?");
        let name = p["name"].as_str().unwrap_or("?");
        let base = p["base_url"].as_str().unwrap_or("default");
        let models = p["models"].as_array().map(|a| a.len()).unwrap_or(0);
        println!("  {}  {}  {}  ({} models)", id, name, base, models);
    }
    Ok(())
}

async fn cmd_agents(base_url: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();
    let resp = client.get(format!("{}/api/agent", base_url)).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let Some(agents) = body["data"].as_array() else {
        println!("No agents available.");
        return Ok(());
    };
    if agents.is_empty() {
        println!("No agents available.");
        return Ok(());
    }
    println!("Agents:");
    for a in agents {
        let id = a["id"].as_str().unwrap_or("?");
        let desc = a["description"].as_str().unwrap_or("");
        let mode = a["mode"].as_str().unwrap_or("?");
        println!("  {}  ({})  {}", id, mode, desc);
    }
    Ok(())
}

async fn cmd_export(base_url: String, session_id: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();

    let resp = client.get(format!("{}/api/session/{}", base_url, session_id)).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Session not found: {}", session_id);
    }
    let session: serde_json::Value = resp.json().await?;

    let resp = client.get(format!("{}/api/session/{}/message", base_url, session_id)).send().await?;
    let messages: serde_json::Value = resp.json().await?;

    let resp = client.get(format!("{}/api/session/{}/event", base_url, session_id)).send().await?;
    let events: serde_json::Value = resp.json().await?;

    let export = serde_json::json!({
        "session": session["data"],
        "messages": messages["data"],
        "events": events["data"],
        "exported_at": chrono::Utc::now().to_rfc3339(),
    });

    println!("{}", serde_json::to_string_pretty(&export)?);
    Ok(())
}

async fn cmd_completion(shell: String) -> anyhow::Result<()> {
    use clap::CommandFactory;
    use clap_complete::{Generator, Shell};
    let mut cmd = Cli::command();
    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "ps" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => anyhow::bail!("Unsupported shell: {}. Supported: bash, zsh, fish, powershell, elvish", shell),
    };
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}

async fn cmd_import(base_url: String, file: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();

    let json_data = tokio::fs::read_to_string(&file).await
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file, e))?;
    let data: serde_json::Value = serde_json::from_str(&json_data)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;

    let session = &data["session"];
    let messages = data["messages"].as_array();
    let session_id = session["id"].as_str().unwrap_or("?");

    // Check if session already exists
    let exists = client.get(format!("{}/api/session/{}", base_url, session_id))
        .send().await.map(|r| r.status().is_success()).unwrap_or(false);

    if !exists {
        // Create the session
        let create_payload = serde_json::json!({
            "id": session_id,
            "agent": session["agent"],
            "model": session["model"],
        });
        client.post(format!("{}/api/session", base_url))
            .json(&create_payload)
            .send().await?;
        println!("Imported session: {}", session_id);
    } else {
        println!("Session already exists: {}", session_id);
    }

    // Replay prompts
    if let Some(msgs) = messages {
        for msg in msgs {
            if msg["role"] == "user" {
                if let Some(text) = msg["content"].as_array()
                    .and_then(|c| c.first())
                    .and_then(|c| c["text"].as_str())
                {
                    client.post(format!("{}/api/session/{}/prompt", base_url, session_id))
                        .json(&serde_json::json!({"prompt": text, "resume": false}))
                        .send().await?;
                    println!("  Replayed message: {}...", &text[..40.min(text.len())]);
                }
            }
        }
    }

    println!("Import complete.");
    Ok(())
}


async fn cmd_cost(base_url: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = base_url.trim_end_matches('/').to_string();
    let resp = client.get(format!("{}/api/cost/summary", base_url)).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let data = &body["data"];

    let sessions = data["total_sessions"].as_u64().unwrap_or(0);
    let total_cost = data["total_cost"].as_f64().unwrap_or(0.0);
    let tokens = &data["total_tokens"];

    println!("Cost Summary");
    println!("  Sessions: {}", sessions);
    println!("  Total cost: {:.6}", total_cost);
    println!("  Tokens — input: {}  output: {}  reasoning: {}",
        tokens["input"].as_f64().unwrap_or(0.0) as u64,
        tokens["output"].as_f64().unwrap_or(0.0) as u64,
        tokens["reasoning"].as_f64().unwrap_or(0.0) as u64);
    println!("  Cache — read: {}  write: {}",
        tokens["cache"]["read"].as_f64().unwrap_or(0.0) as u64,
        tokens["cache"]["write"].as_f64().unwrap_or(0.0) as u64);
    println!("");

    if let Some(by_provider) = data["by_provider"].as_object() {
        if !by_provider.is_empty() {
            println!("  By provider:");
            for (provider, cost) in by_provider {
                println!("    {}: {:.6}", provider, cost.as_f64().unwrap_or(0.0));
            }
        }
    }
    if let Some(by_model) = data["by_model"].as_object() {
        if !by_model.is_empty() {
            println!("  By model:");
            for (model, cost) in by_model {
                println!("    {}: {:.6}", model, cost.as_f64().unwrap_or(0.0));
            }
        }
    }

    Ok(())
}
