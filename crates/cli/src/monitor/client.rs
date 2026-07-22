use anyhow::{Context, Result};
use futures::stream::Stream;
use opencode_r_protocol::payload::{CursorResponse, DataResponse, HealthResponse};
use opencode_r_protocol::route;
use opencode_r_schema::session::SessionInfo;
use opencode_r_schema::session_event::SessionEvent;
use opencode_r_schema::session_message::SessionMessage;
use reqwest::Client;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

// ── Client ──────────────────────────────────────

#[derive(Clone)]
pub struct MonitorClient {
    base_url: String,
    http: Client,
}

impl MonitorClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            http: Client::new(),
        }
    }

    // ── Health ──

    pub async fn health(&self) -> Result<bool> {
        let resp = self
            .http
            .get(format!("{}{}", self.base_url, route::HEALTH))
            .send()
            .await
            .context("health check failed")?;
        let body: HealthResponse = resp.json().await.context("parsing health response")?;
        Ok(body.healthy)
    }

    /// Composite status: health + session count
    pub async fn status(&self) -> Result<DaemonStatus> {
        let healthy = self.health().await?;
        let session_count = self.list_sessions().await.map(|s| s.len()).unwrap_or(0);
        Ok(DaemonStatus {
            healthy,
            session_count,
        })
    }

    // ── Sessions ──

    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        self.list_sessions_filtered(None).await
    }

    pub async fn list_sessions_filtered(&self, search: Option<&str>) -> Result<Vec<SessionInfo>> {
        let mut url = format!("{}{}", self.base_url, route::SESSION_LIST);
        if let Some(q) = search {
            // Simple URL encoding for search query (ASCII-safe)
            let encoded: String = q
                .chars()
                .map(|c| match c {
                    ' ' => "%20".to_string(),
                    '#' => "%23".to_string(),
                    '&' => "%26".to_string(),
                    '=' => "%3D".to_string(),
                    '?' => "%3F".to_string(),
                    c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => {
                        c.to_string()
                    }
                    c => format!("%{:02X}", c as u8),
                })
                .collect();
            url.push_str(&format!("?search={encoded}"));
        }
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("listing sessions")?;
        let body: CursorResponse<Vec<SessionInfo>> =
            resp.json().await.context("parsing session list")?;
        Ok(body.data)
    }

    pub async fn get_session(&self, id: &str) -> Result<SessionInfo> {
        let url = route::session_get(id);
        let resp = self
            .http
            .get(format!("{}{url}", self.base_url))
            .send()
            .await
            .context("getting session")?;
        let body: DataResponse<SessionInfo> = resp.json().await.context("parsing session")?;
        Ok(body.data)
    }

    // ── Control ──

    pub async fn pause(&self, id: &str) -> Result<()> {
        let url = route::session_pause(id);
        self.http
            .post(format!("{}{url}", self.base_url))
            .send()
            .await
            .context("pausing session")?;
        Ok(())
    }

    pub async fn resume(&self, id: &str) -> Result<()> {
        let url = route::session_resume(id);
        self.http
            .post(format!("{}{url}", self.base_url))
            .send()
            .await
            .context("resuming session")?;
        Ok(())
    }

    pub async fn freeze(&self, id: &str) -> Result<()> {
        let url = route::session_freeze(id);
        self.http
            .post(format!("{}{url}", self.base_url))
            .send()
            .await
            .context("freezing session")?;
        Ok(())
    }

    pub async fn terminate(&self, id: &str) -> Result<()> {
        let url = route::session_terminate(id);
        self.http
            .post(format!("{}{url}", self.base_url))
            .send()
            .await
            .context("terminating session")?;
        Ok(())
    }

    // ── Messages ──

    pub async fn get_session_messages(&self, id: &str) -> Result<Vec<SessionMessage>> {
        let url = route::session_messages(id);
        let resp = self
            .http
            .get(format!("{}{url}", self.base_url))
            .send()
            .await
            .context("fetching session messages")?;
        let body: CursorResponse<Vec<SessionMessage>> =
            resp.json().await.context("parsing messages")?;
        Ok(body.data)
    }

    // ── Events (SSE) ──

    /// Subscribe to global events stream.
    pub async fn event_stream(&self) -> Result<SseEventStream> {
        let resp = self
            .http
            .get(format!("{}{}", self.base_url, route::EVENT_SUBSCRIBE))
            .send()
            .await
            .context("connecting to event stream")?;
        Ok(SseEventStream::new(resp))
    }
}

// ── Daemon status ───────────────────────────────

#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub healthy: bool,
    pub session_count: usize,
}

// ── SSE event stream ────────────────────────────

/// Parsed SSE event stream. Spawns a background task to read from reqwest,
/// sends parsed SessionEvents over a channel.
pub struct SseEventStream {
    rx: tokio::sync::mpsc::Receiver<Result<SessionEvent>>,
}

impl SseEventStream {
    fn new(response: reqwest::Response) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<SessionEvent>>(256);

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = Vec::new();

            loop {
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.extend_from_slice(&chunk);

                        // Parse complete SSE events (separated by \n\n)
                        while let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                            let event_bytes: Vec<u8> = buffer.drain(..pos + 2).collect();

                            // Extract data: lines
                            for line in event_bytes.split(|b| *b == b'\n') {
                                if let Some(payload) = line.strip_prefix(b"data: ") {
                                    match serde_json::from_slice::<SessionEvent>(payload) {
                                        Ok(evt) => {
                                            if tx.send(Ok(evt)).await.is_err() {
                                                return; // receiver dropped
                                            }
                                        }
                                        Err(_) => continue,
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        let _ = tx
                            .send(Err(anyhow::anyhow!("SSE read error: {e}")))
                            .await;
                        return;
                    }
                    None => return, // stream ended
                }
            }
        });

        Self { rx }
    }
}

impl Stream for SseEventStream {
    type Item = Result<SessionEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
