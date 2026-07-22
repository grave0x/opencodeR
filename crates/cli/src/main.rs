use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("opencode_server=info".parse().unwrap())
            .add_directive("tower_http=info".parse().unwrap()))
        .init();

    // Read optional auth password from env
    if let Ok(password) = std::env::var("OPENCODE_PASSWORD") {
        opencode_server::middleware::auth::set_password(password);
        tracing::info!("Auth enabled (password-based)");
    }

    let state = Arc::new(opencode_server::state::AppState::new());
    let app = opencode_server::build_router(state);

    let port = std::env::var("OPENCODE_PORT").ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8081u16);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("OpenCode Rust server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
