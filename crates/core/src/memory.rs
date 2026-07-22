// In-memory service implementations for development/testing
use crate::*;
use opencode_r_schema::agent::{AgentMode, AgentID};
use opencode_r_schema::command::Command;
use opencode_r_schema::integration::{Integration, IntegrationKind};
use opencode_r_schema::model::{ModelInfo, ModelLimits, ModelRef};
use opencode_r_schema::permission::{PermissionAction, PermissionRule, PermissionTarget};
use opencode_r_schema::project::ProjectID;
use opencode_r_schema::provider::{ProviderInfo, ProviderRequest};
use opencode_r_schema::pty::PtyInfo;
use opencode_r_schema::pty_ticket::PtyTicket;
use opencode_r_schema::question::Question;
use opencode_r_schema::reference::Reference;
use opencode_r_schema::revert::{RevertKind, RevertState};
use opencode_r_schema::session::{CacheUsage, SessionInfo, SessionTime, TokenUsage};
use opencode_r_schema::session_event::{SessionEvent, SessionEventKind};
use opencode_r_schema::session_id::SessionID;
use opencode_r_schema::session_message::{MessageContent, MessageRole, SessionMessage, SessionMessageID};
use opencode_r_schema::skill::Skill;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::info;
use chrono::Utc;

// ---- InMemoryAgentService ----

pub struct InMemoryAgentService;

impl AgentService for InMemoryAgentService {
    fn list(&self) -> Vec<AgentInfo> {
        vec![
            AgentInfo {
                id: AgentID("build".into()),
                model: None,
                request: ProviderRequest {
                    headers: HashMap::new(),
                    body: HashMap::new(),
                },
                system: None,
                description: Some("Full-access agent for development work".into()),
                mode: AgentMode::Primary,
                hidden: false,
                color: None,
                steps: None,
                permissions: vec![PermissionRule {
                    action: PermissionAction::Admin,
                    target: PermissionTarget::All,
                    allow: true,
                }],
            },
            AgentInfo {
                id: AgentID("plan".into()),
                model: None,
                request: ProviderRequest {
                    headers: HashMap::new(),
                    body: HashMap::new(),
                },
                system: None,
                description: Some("Read-only agent for analysis and exploration".into()),
                mode: AgentMode::Primary,
                hidden: false,
                color: None,
                steps: None,
                permissions: vec![PermissionRule {
                    action: PermissionAction::Read,
                    target: PermissionTarget::All,
                    allow: true,
                }],
            },
        ]
    }
}

// ---- InMemoryCatalogService ----

pub struct InMemoryCatalogService;

impl CatalogService for InMemoryCatalogService {
    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: ModelRef("anthropic/claude-sonnet-4".into()),
                provider_id: "anthropic".into(),
                name: "Claude Sonnet 4".into(),
                limits: Some(ModelLimits {
                    max_input: Some(200_000),
                    max_output: Some(8192),
                }),
            },
            ModelInfo {
                id: ModelRef("openai/gpt-4o".into()),
                provider_id: "openai".into(),
                name: "GPT-4o".into(),
                limits: Some(ModelLimits {
                    max_input: Some(128_000),
                    max_output: Some(4096),
                }),
            },
        ]
    }

    fn list_providers(&self) -> Vec<ProviderInfo> {
        vec![
            ProviderInfo {
                id: "anthropic".into(),
                name: "Anthropic".into(),
                base_url: None,
                models: vec!["claude-sonnet-4".into(), "claude-haiku-3".into()],
            },
            ProviderInfo {
                id: "openai".into(),
                name: "OpenAI".into(),
                base_url: None,
                models: vec!["gpt-4o".into(), "gpt-4o-mini".into()],
            },
        ]
    }

    fn get_provider(&self, id: &str) -> Option<ProviderInfo> {
        self.list_providers().into_iter().find(|p| p.id == id)
    }
}

// ---- InMemorySessionService ----

struct SessionStore {
    sessions: HashMap<SessionID, SessionInfo>,
    messages: HashMap<SessionID, Vec<SessionMessage>>,
    events: HashMap<SessionID, Vec<SessionEvent>>,
    event_tx: tokio::sync::broadcast::Sender<SessionEvent>,
}

impl SessionStore {
    fn new() -> (Self, tokio::sync::broadcast::Receiver<SessionEvent>) {
        let (tx, rx) = tokio::sync::broadcast::channel(256);
        (Self {
            sessions: HashMap::new(),
            messages: HashMap::new(),
            events: HashMap::new(),
            event_tx: tx,
        }, rx)
    }

    /// Leetopt: push an event under the same lock — no second mutex acquisition.
    fn push_event(&mut self, session_id: &SessionID, kind: SessionEventKind, data: serde_json::Value) {
        let now = chrono::Utc::now();
        let event = SessionEvent {
            id: opencode_r_schema::identifier::ascending(),
            session_id: session_id.clone(),
            kind,
            data,
            timestamp: now,
        };
        self.events.entry(session_id.clone()).or_default().push(event.clone());
        let _ = self.event_tx.send(event);
    }
}

pub struct InMemorySessionService {
    inner: Mutex<SessionStore>,
    event_rx: tokio::sync::broadcast::Receiver<SessionEvent>,
}

impl InMemorySessionService {
    pub fn new() -> Self {
        let (store, rx) = SessionStore::new();
        Self { inner: Mutex::new(store), event_rx: rx }
    }
}

impl SessionService for InMemorySessionService {
    fn list(&self, query: SessionListQuery) -> SessionListResult {
        let store = self.inner.lock().unwrap();

        let mut refs: Vec<&SessionInfo> = store.sessions.values().collect();

        // Leetopt: search filter — case-insensitive match on title, agent, model, id
        if let Some(search) = &query.search {
            let lower = search.to_lowercase();
            refs.retain(|s| {
                s.title.to_lowercase().contains(&lower)
                    || s.id.0.to_lowercase().contains(&lower)
                    || s.agent.as_ref().is_some_and(|a| a.0.to_lowercase().contains(&lower))
                    || s.model.as_ref().is_some_and(|m| m.0.to_lowercase().contains(&lower))
            });
        }

        let asc = matches!(query.order.as_deref(), Some("asc"));
        refs.sort_by(|a, b| a.time.created.cmp(&b.time.created));
        if !asc {
            refs.reverse();
        }

        let limit = query.limit.unwrap_or(50) as usize;
        let (start, end) = if let Some(cursor_id) = &query.cursor_id {
            let dir = query.cursor_direction.as_deref().unwrap_or("next");
            if let Some(pos) = refs.iter().position(|s| s.id.0.as_str() == cursor_id.as_str()) {
                match dir {
                    "next" => {
                        let s = (pos + 1).min(refs.len());
                        (s, (s + limit).min(refs.len()))
                    }
                    _ => {
                        let end = pos;
                        let start = end.saturating_sub(limit);
                        (start, end)
                    }
                }
            } else {
                (0, limit.min(refs.len()))
            }
        } else {
            (0, limit.min(refs.len()))
        };

        if start >= refs.len() || end <= start {
            return Vec::new();
        }
        refs[start..end].iter().map(|s| (*s).clone()).collect()
    }

    fn create(&self, input: SessionCreateInput) -> SessionInfo {
        let mut store = self.inner.lock().unwrap();
        let now = Utc::now();
        let id = input.id.unwrap_or_else(|| SessionID(opencode_r_schema::identifier::ascending()));
        let info = SessionInfo {
            id: id.clone(),
            parent_id: None,
            project_id: ProjectID("default".into()),
            agent: input.agent.map(|a| opencode_r_schema::agent::AgentID(a)),
            model: input.model.map(|m| ModelRef(m)),
            cost: 0.0,
            tokens: TokenUsage {
                input: 0.0,
                output: 0.0,
                reasoning: 0.0,
                cache: CacheUsage { read: 0.0, write: 0.0 },
            },
            time: SessionTime { created: now, updated: now, archived: None },
            title: "New Session".into(),
            location: opencode_r_schema::location::LocationRef(input.location.unwrap_or_else(|| "local".into())),
            subpath: None,
            revert: None,
            status: opencode_r_schema::session::SessionStatus::Active,
            group: None,
        };
        store.sessions.insert(id.clone(), info.clone());
        store.push_event(&id, SessionEventKind::SessionCreated, serde_json::json!({"title": "New Session"}));
        info!(
            target: "opencode_r_core::session",
            session_id = %id.0,
            agent = ?info.agent.as_ref().map(|a| &a.0),
            "session_created"
        );
        info
    }

    fn active(&self) -> HashMap<SessionID, String> {
        HashMap::new()
    }

    fn get(&self, id: &SessionID) -> Option<SessionInfo> {
        let store = self.inner.lock().unwrap();
        store.sessions.get(id).cloned()
    }

    fn switch_agent(&self, session_id: &SessionID, agent: &str) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(session) = store.sessions.get_mut(session_id) {
            session.agent = Some(opencode_r_schema::agent::AgentID(agent.into()));
            session.time.updated = Utc::now();
            store.push_event(session_id, SessionEventKind::MessageAdded, serde_json::json!({"type": "agent_switch", "agent": agent}));
            info!(target: "opencode_r_core::session", session_id = %session_id.0, agent = %agent, "agent_switched");
            Ok(())
        } else {
            Err(())
        }
    }

    fn switch_model(&self, session_id: &SessionID, model: &str) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(session) = store.sessions.get_mut(session_id) {
            session.model = Some(ModelRef(model.into()));
            session.time.updated = Utc::now();
            store.push_event(session_id, SessionEventKind::MessageAdded, serde_json::json!({"type": "model_switch", "model": model}));
            info!(target: "opencode_r_core::session", session_id = %session_id.0, model = %model, "model_switched");
            Ok(())
        } else {
            Err(())
        }
    }

    fn prompt(&self, session_id: &SessionID, input: SessionPromptInput) -> Result<String, String> {
        let mut store = self.inner.lock().unwrap();
        if !store.sessions.contains_key(session_id) {
            return Err("Session not found".into());
        }
        let msg_id = input.id.unwrap_or_else(|| SessionMessageID(opencode_r_schema::identifier::ascending()));
        let now = Utc::now();
        let entry = store.messages.entry(session_id.clone()).or_default();
        entry.push(SessionMessage {
            id: msg_id.clone(),
            session_id: session_id.clone(),
            role: MessageRole::User,
            content: vec![MessageContent::Text { text: input.prompt }],
            created_at: now,
        });
        store.push_event(session_id, SessionEventKind::MessageAdded, serde_json::json!({"message_id": msg_id.0, "role": "user"}));
        info!(target: "opencode_r_core::session", session_id = %session_id.0, msg_id = %msg_id.0, "message_admitted");
        Ok(msg_id.0)
    }

    fn compact(&self, _session_id: &SessionID) -> Result<(), String> {
        Ok(())
    }

    fn wait(&self, session_id: &SessionID) -> Result<(), String> {
        let store = self.inner.lock().unwrap();
        if store.sessions.contains_key(session_id) { Ok(()) } else { Err("Session not found".into()) }
    }

    fn revert_stage(&self, _session_id: &SessionID, _input: SessionRevertStageInput) -> Result<RevertState, String> {
        Ok(RevertState {
            kind: RevertKind::File,
            checkpoint_id: "checkpoint_1".into(),
            timestamp: Utc::now().timestamp_millis(),
        })
    }

    fn revert_clear(&self, session_id: &SessionID) -> Result<(), String> {
        let mut store = self.inner.lock().unwrap();
        if let Some(session) = store.sessions.get_mut(session_id) {
            session.revert = None;
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    fn revert_commit(&self, session_id: &SessionID) -> Result<(), String> {
        let store = self.inner.lock().unwrap();
        if store.sessions.contains_key(session_id) { Ok(()) } else { Err("Session not found".into()) }
    }

    fn context(&self, session_id: &SessionID) -> Result<Vec<SessionMessage>, String> {
        let store = self.inner.lock().unwrap();
        if !store.sessions.contains_key(session_id) {
            return Err("Session not found".into());
        }
        Ok(store.messages.get(session_id).cloned().unwrap_or_default())
    }

    fn history(&self, session_id: &SessionID, _query: SessionHistoryQuery) -> Result<SessionHistoryResult, String> {
        let store = self.inner.lock().unwrap();
        if !store.sessions.contains_key(session_id) {
            return Err("Session not found".into());
        }
        let evts = store.events.get(session_id).cloned().unwrap_or_default();
        Ok(SessionHistoryResult { events: evts, has_more: false })
    }

    fn events(&self, session_id: &SessionID, _after: Option<u32>) -> Vec<SessionEvent> {
        let store = self.inner.lock().unwrap();
        store.events.get(session_id).cloned().unwrap_or_default()
    }

    fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<SessionEvent> {
        self.event_rx.resubscribe()
    }

    fn global_events(&self, _after: Option<u32>, limit: Option<u32>) -> Vec<SessionEvent> {
        let store = self.inner.lock().unwrap();
        let mut all: Vec<SessionEvent> = store.events.values().flat_map(|v| v.iter().cloned()).collect();
        all.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        if let Some(l) = limit {
            all.truncate(l as usize);
        }
        all
    }

    fn interrupt(&self, _session_id: &SessionID) {}

    fn pause(&self, session_id: &SessionID) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(s) = store.sessions.get_mut(session_id) {
            s.status = opencode_r_schema::session::SessionStatus::Paused;
            s.time.updated = Utc::now();
            store.push_event(session_id, SessionEventKind::MessageAdded,
                serde_json::json!({"type": "lifecycle", "action": "pause"}));
            info!(target: "opencode_r_core::session", session_id = %session_id.0, "session_paused");
            Ok(())
        } else { Err(()) }
    }

    fn resume(&self, session_id: &SessionID) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(s) = store.sessions.get_mut(session_id) {
            s.status = opencode_r_schema::session::SessionStatus::Active;
            s.time.updated = Utc::now();
            store.push_event(session_id, SessionEventKind::MessageAdded,
                serde_json::json!({"type": "lifecycle", "action": "resume"}));
            info!(target: "opencode_r_core::session", session_id = %session_id.0, "session_resumed");
            Ok(())
        } else { Err(()) }
    }

    fn freeze(&self, session_id: &SessionID) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(s) = store.sessions.get_mut(session_id) {
            s.status = opencode_r_schema::session::SessionStatus::Frozen;
            s.time.updated = Utc::now();
            store.push_event(session_id, SessionEventKind::MessageAdded,
                serde_json::json!({"type": "lifecycle", "action": "freeze"}));
            info!(target: "opencode_r_core::session", session_id = %session_id.0, "session_frozen");
            Ok(())
        } else { Err(()) }
    }

    fn terminate(&self, session_id: &SessionID) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(s) = store.sessions.get_mut(session_id) {
            s.status = opencode_r_schema::session::SessionStatus::Terminated;
            s.time.updated = Utc::now();
            store.push_event(session_id, SessionEventKind::SessionArchived,
                serde_json::json!({"type": "lifecycle", "action": "terminate"}));
            info!(target: "opencode_r_core::session", session_id = %session_id.0, "session_terminated");
            Ok(())
        } else { Err(()) }
    }

    fn set_group(&self, session_id: &SessionID, group: Option<String>) -> Result<(), ()> {
        let mut store = self.inner.lock().unwrap();
        if let Some(s) = store.sessions.get_mut(session_id) {
            s.group = group;
            s.time.updated = Utc::now();
            Ok(())
        } else { Err(()) }
    }

    fn list_groups(&self) -> Vec<(String, usize)> {
        let store = self.inner.lock().unwrap();
        let mut groups: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for s in store.sessions.values() {
            if let Some(g) = &s.group {
                *groups.entry(g.clone()).or_default() += 1;
            }
        }
        let mut result: Vec<(String, usize)> = groups.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    fn cost_summary(&self) -> opencode_r_schema::session::CostSummary {
        let store = self.inner.lock().unwrap();
        let total_sessions = store.sessions.len();
        let mut total_cost = 0.0_f64;
        let mut total_tokens = opencode_r_schema::session::TokenUsage {
            input: 0.0, output: 0.0, reasoning: 0.0,
            cache: opencode_r_schema::session::CacheUsage { read: 0.0, write: 0.0 },
        };
        let mut by_provider: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut by_model: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

        for s in store.sessions.values() {
            total_cost += s.cost;
            total_tokens.input += s.tokens.input;
            total_tokens.output += s.tokens.output;
            total_tokens.reasoning += s.tokens.reasoning;
            total_tokens.cache.read += s.tokens.cache.read;
            total_tokens.cache.write += s.tokens.cache.write;
            if let Some(model) = &s.model {
                *by_model.entry(model.0.clone()).or_default() += s.cost;
                let provider = model.0.split('/').next().unwrap_or("unknown").to_string();
                *by_provider.entry(provider).or_default() += s.cost;
            }
        }

        opencode_r_schema::session::CostSummary {
            total_sessions,
            total_cost,
            total_tokens,
            by_provider,
            by_model,
        }
    }

    fn cost_breakdown(&self, session_id: &SessionID) -> Option<opencode_r_schema::session::CostBreakdown> {
        let store = self.inner.lock().unwrap();
        let session = store.sessions.get(session_id)?;
        let mut by_provider = std::collections::HashMap::new();
        let mut by_model = std::collections::HashMap::new();
        if let Some(model) = &session.model {
            by_model.insert(model.0.clone(), session.cost);
            let provider = model.0.split('/').next().unwrap_or("unknown").to_string();
            by_provider.insert(provider, session.cost);
        }
        Some(opencode_r_schema::session::CostBreakdown {
            by_provider,
            by_model,
            total_cost: session.cost,
            total_tokens: session.tokens.clone(),
        })
    }

    fn messages(&self, query: SessionMessagesQuery) -> Result<Vec<SessionMessage>, String> {
        let store = self.inner.lock().unwrap();
        if !store.sessions.contains_key(&query.session_id) {
            return Err("Session not found".into());
        }
        let Some(raw) = store.messages.get(&query.session_id) else {
            return Ok(Vec::new());
        };

        // Leetopt: sort references, clone only what survives truncation
        let mut refs: Vec<&SessionMessage> = raw.iter().collect();
        let asc = matches!(query.order.as_deref(), Some("asc"));
        refs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        if !asc {
            refs.reverse();
        }
        let limit = query.limit.unwrap_or(50) as usize;
        refs.truncate(limit);
        Ok(refs.iter().map(|m| (*m).clone()).collect())
    }

    fn message(&self, session_id: &SessionID, message_id: &SessionMessageID) -> Option<SessionMessage> {
        let store = self.inner.lock().unwrap();
        store.messages.get(session_id)
            .and_then(|msgs| msgs.iter().find(|m| m.id == *message_id).cloned())
    }
}

// ---- InMemoryPtyService ----

use std::process::{Child, ChildStdin, ChildStdout, Stdio};

struct PtyProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
}

pub struct InMemoryPtyService {
    ptys: Mutex<HashMap<String, PtyInfo>>,
    processes: Mutex<HashMap<String, PtyProcess>>,
}

impl InMemoryPtyService {
    pub fn new() -> Self {
        Self {
            ptys: Mutex::new(HashMap::new()),
            processes: Mutex::new(HashMap::new()),
        }
    }
}

impl PtyService for InMemoryPtyService {
    fn list(&self) -> Vec<PtyInfo> {
        self.ptys.lock().unwrap().values().cloned().collect()
    }

    fn create(&self, input: PtyCreateInput) -> PtyInfo {
        let id = opencode_r_schema::identifier::ascending();
        let cmd = input.command.unwrap_or_else(|| "bash -c 'echo opencodeR-pty'".into());
        let shell = "sh";

        // Spawn a real shell process
        let result = std::process::Command::new(shell)
            .arg("-c")
            .arg(&cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let pid = result.as_ref().ok().map(|c| c.id());
        if pid.is_none() {
            info!(target: "opencode_r_core::pty", cmd = %cmd, "PTY spawn failed");
        }
        let info = PtyInfo {
            id: id.clone(),
            cols: input.cols,
            rows: input.rows,
            pid,
        };

        let mut ptys = self.ptys.lock().unwrap();
        ptys.insert(id.clone(), info.clone());

        if let Ok(mut child) = result {
            let stdin = child.stdin.take();
            let stdout = child.stdout.take();
            let mut processes = self.processes.lock().unwrap();
            processes.insert(id, PtyProcess { child, stdin, stdout });
        }

        info
    }

    fn get(&self, id: &str) -> Option<PtyInfo> {
        self.ptys.lock().unwrap().get(id).cloned()
    }

    fn update(&self, id: &str, input: PtyUpdateInput) -> Option<PtyInfo> {
        let mut ptys = self.ptys.lock().unwrap();
        if let Some(pty) = ptys.get_mut(id) {
            pty.cols = input.cols;
            pty.rows = input.rows;
            Some(pty.clone())
        } else {
            None
        }
    }

    fn connect_token(&self, id: &str) -> Option<PtyTicket> {
        if self.ptys.lock().unwrap().contains_key(id) {
            Some(PtyTicket {
                id: opencode_r_schema::identifier::ascending(),
                pty_id: id.into(),
                token: opencode_r_schema::identifier::ascending(),
                expires_at: Utc::now().timestamp_millis() + 300_000,
            })
        } else {
            None
        }
    }

    fn attach_stdio(&self, id: &str) -> Option<PtyStdio> {
        let mut processes = self.processes.lock().unwrap();
        if let Some(p) = processes.get_mut(id) {
            let stdin = p.stdin.take()?;
            let stdout = p.stdout.take()?;
            Some(PtyStdio { stdin, stdout })
        } else {
            None
        }
    }
}

// ---- InMemoryPermissionService ----

pub struct InMemoryPermissionService {
    requests: Mutex<Vec<(String, String, serde_json::Value)>>, // (session_id, request_id, data)
}

impl InMemoryPermissionService {
    pub fn new() -> Self {
        Self { requests: Mutex::new(Vec::new()) }
    }
}

impl PermissionService for InMemoryPermissionService {
    fn request_list(&self) -> Vec<serde_json::Value> {
        let all = self.requests.lock().unwrap();
        all.iter().map(|(_, _, v)| v.clone()).collect()
    }

    fn saved_list(&self, _project_id: Option<String>) -> Vec<serde_json::Value> { vec![] }

    fn session_list(&self, session_id: &str) -> Vec<serde_json::Value> {
        let all = self.requests.lock().unwrap();
        all.iter().filter(|(sid, _, _)| sid == session_id).map(|(_, _, v)| v.clone()).collect()
    }

    fn session_create(&self, input: PermissionCreateInput) -> serde_json::Value {
        let id = input.id.unwrap_or_else(|| opencode_r_schema::identifier::ascending());
        let data = serde_json::json!({
            "id": id,
            "session_id": input.session_id,
            "action": input.action,
            "resources": input.resources,
            "effect": "allow"
        });
        let mut all = self.requests.lock().unwrap();
        all.push((input.session_id, id.clone(), data.clone()));
        data
    }

    fn session_get(&self, session_id: &str, request_id: &str) -> Option<serde_json::Value> {
        let all = self.requests.lock().unwrap();
        all.iter()
            .find(|(sid, rid, _)| sid == session_id && rid == request_id)
            .map(|(_, _, v)| v.clone())
    }

    fn session_reply(&self, session_id: &str, request_id: &str, _input: PermissionReplyInput) -> Result<(), ()> {
        let all = self.requests.lock().unwrap();
        if all.iter().any(|(sid, rid, _)| sid == session_id && rid == request_id) {
            Ok(())
        } else {
            Err(())
        }
    }
}

// ---- InMemoryQuestionService ----

pub struct InMemoryQuestionService {
    questions: Mutex<Vec<Question>>,
}

impl InMemoryQuestionService {
    pub fn new() -> Self {
        Self { questions: Mutex::new(Vec::new()) }
    }
}

impl QuestionService for InMemoryQuestionService {
    fn request_list(&self) -> Vec<Question> {
        let all = self.questions.lock().unwrap();
        all.clone()
    }

    fn session_list(&self, session_id: &str) -> Vec<Question> {
        let all = self.questions.lock().unwrap();
        all.iter().filter(|q| q.id.starts_with(session_id)).cloned().collect()
    }

    fn session_reply(&self, _session_id: &str, request_id: &str, input: QuestionReplyInput) -> Result<(), ()> {
        let mut all = self.questions.lock().unwrap();
        if let Some(q) = all.iter_mut().find(|q| q.id == request_id) {
            q.answer = Some(input.answers.join(", "));
            Ok(())
        } else {
            Err(())
        }
    }

    fn session_reject(&self, _session_id: &str, request_id: &str) -> Result<(), ()> {
        let all = self.questions.lock().unwrap();
        if let Some(_q) = all.iter().position(|q| q.id == request_id) {
            Ok(())
        } else {
            Err(())
        }
    }
}

// ---- InMemoryFileSystemService ----

pub struct InMemoryFileSystemService;

impl InMemoryFileSystemService {
    fn mime_for_path(path: &str) -> &'static str {
        if path.ends_with(".rs") { "text/x-rust" }
        else if path.ends_with(".toml") { "text/toml" }
        else if path.ends_with(".md") { "text/markdown" }
        else if path.ends_with(".json") { "application/json" }
        else if path.ends_with(".py") { "text/x-python" }
        else if path.ends_with(".ts") { "text/typescript" }
        else if path.ends_with(".js") { "text/javascript" }
        else if path.ends_with(".css") { "text/css" }
        else if path.ends_with(".html") { "text/html" }
        else if path.ends_with(".png") { "image/png" }
        else if path.ends_with(".jpg") || path.ends_with(".jpeg") { "image/jpeg" }
        else { "text/plain" }
    }
}

impl FileSystemService for InMemoryFileSystemService {
    fn read(&self, query: FsReadQuery) -> Result<FsReadResult, String> {
        let path_str = query.path.0;
        let path = std::path::Path::new(&path_str);
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }
        if !path.is_file() {
            return Err(format!("Not a file: {}", path_str));
        }
        let content = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let mime = Self::mime_for_path(&path_str).to_string();
        Ok(FsReadResult { content, mime })
    }

    fn list(&self, query: FsListQuery) -> Vec<serde_json::Value> {
        let dir = query.path.map(|p| p.0).unwrap_or_else(|| ".".into());
        let path = std::path::Path::new(&dir);
        if !path.is_dir() {
            return vec![];
        }
        let mut entries = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                let fname = entry.file_name();
                let name = fname.to_string_lossy();
                let path_buf = entry.path();
                let entry_path = path_buf.to_string_lossy();
                let ft = entry.file_type().ok();
                entries.push(serde_json::json!({
                    "name": name.as_ref(),
                    "type": if ft.map(|t| t.is_dir()).unwrap_or(false) { "directory" } else { "file" },
                    "path": entry_path.as_ref(),
                }));
            }
        }
        entries
    }

    fn find(&self, query: FsFindQuery) -> Vec<serde_json::Value> {
        let mut results = Vec::new();
        let dir = std::path::Path::new(".");
        let query_lower = query.query.to_lowercase();
        let limit = query.limit.unwrap_or(50) as usize;
        visit_dirs(dir, &query_lower, &mut results, limit, 0);
        results
    }
}

fn visit_dirs(dir: &std::path::Path, query: &str, results: &mut Vec<serde_json::Value>, limit: usize, depth: usize) {
    if depth > 8 || results.len() >= limit { return; }
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            if results.len() >= limit { break; }
            let fname = entry.file_name();
            let name_lossy = fname.to_string_lossy();

            // Leetopt: byte-level case-insensitive contains — no String allocation
            if contains_ignore_ascii_case(name_lossy.as_bytes(), query.as_bytes()) {
                let path = entry.path();
                results.push(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "name": name_lossy.as_ref(),
                    "type": "file",
                }));
            }

            // Leetopt: use file_type from read_dir instead of path.is_dir() (saves a stat)
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name_ref = name_lossy.as_ref();
                    if !name_ref.starts_with('.') && name_ref != "target" && name_ref != "node_modules" {
                        visit_dirs(&entry.path(), query, results, limit, depth + 1);
                    }
                }
            }
        }
    }
}

/// Leetopt: case-insensitive contains on byte slices — zero allocation.
#[inline(always)]
fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() { return true; }
    if needle.len() > haystack.len() { return false; }
    haystack.windows(needle.len()).any(|w| w.eq_ignore_ascii_case(needle))
}

// ---- InMemoryIntegrationService ----

pub struct InMemoryIntegrationService;

impl IntegrationService for InMemoryIntegrationService {
    fn list(&self) -> Vec<Integration> {
        vec![
            Integration {
                id: "github".into(),
                name: "GitHub".into(),
                kind: IntegrationKind::GitHub,
                config: serde_json::json!({"scopes": ["repo", "read:org"]}),
            },
        ]
    }

    fn get(&self, id: &str) -> Option<Integration> {
        self.list().into_iter().find(|i| i.id == id)
    }

    fn connect_key(&self, _input: ConnectKeyInput) -> Result<(), ()> { Ok(()) }
    fn connect_oauth(&self, _input: ConnectOAuthInput) -> Result<serde_json::Value, ()> {
        Ok(serde_json::json!({"url": "https://github.com/login/oauth/authorize?client_id=test"}))
    }
    fn attempt_status(&self, _attempt_id: &str) -> Option<serde_json::Value> { None }
    fn attempt_complete(&self, _attempt_id: &str, _code: Option<String>) -> Result<(), String> { Ok(()) }
    fn attempt_cancel(&self, _attempt_id: &str) {}
}

// ---- InMemoryCredentialService ----

pub struct InMemoryCredentialService;

impl CredentialService for InMemoryCredentialService {
    fn update(&self, _id: &str, _input: CredentialUpdateInput) -> Result<(), ()> { Ok(()) }
    fn remove(&self, _id: &str) {}
}

// ---- InMemoryCommandService ----

pub struct InMemoryCommandService;

impl CommandService for InMemoryCommandService {
    fn list(&self) -> Vec<Command> {
        vec![
            Command { id: "npm-test".into(), name: "npm test".into(), description: Some("Run tests".into()), command: "npm".into(), args: vec!["test".into()] },
            Command { id: "cargo-build".into(), name: "cargo build".into(), description: Some("Build project".into()), command: "cargo".into(), args: vec!["build".into()] },
        ]
    }
}

// ---- InMemorySkillService ----

pub struct InMemorySkillService;

impl SkillService for InMemorySkillService {
    fn list(&self) -> Vec<Skill> {
        vec![
            Skill { id: "rust".into(), name: "Rust".into(), description: Some("Rust programming".into()), path: ".opencode/skills/rust.md".into() },
        ]
    }
}

// ---- InMemoryReferenceService ----

pub struct InMemoryReferenceService;

impl ReferenceService for InMemoryReferenceService {
    fn list(&self) -> Vec<Reference> {
        vec![]
    }
}

// ---- InMemoryEventService ----

pub struct InMemoryEventService;

impl EventService for InMemoryEventService {
    fn subscribe(&self) -> Vec<Event> {
        vec![]
    }
}

// ---- InMemoryProjectCopyService ----

pub struct InMemoryProjectCopyService;

impl ProjectCopyService for InMemoryProjectCopyService {
    fn create(&self, _input: ProjectCopyCreateInput) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"status": "created"}))
    }
    fn refresh(&self, _project_id: &str) -> Result<(), String> { Ok(()) }
}

// ---- Builder for easy construction ----

pub fn default_services() -> (
    Box<dyn AgentService + Send + Sync>,
    Box<dyn CatalogService + Send + Sync>,
    Box<dyn SessionService + Send + Sync>,
    Box<dyn PtyService + Send + Sync>,
    Box<dyn PermissionService + Send + Sync>,
    Box<dyn QuestionService + Send + Sync>,
    Box<dyn FileSystemService + Send + Sync>,
    Box<dyn IntegrationService + Send + Sync>,
    Box<dyn CredentialService + Send + Sync>,
    Box<dyn CommandService + Send + Sync>,
    Box<dyn SkillService + Send + Sync>,
    Box<dyn ReferenceService + Send + Sync>,
    Box<dyn EventService + Send + Sync>,
    Box<dyn ProjectCopyService + Send + Sync>,
) {
    (
        Box::new(InMemoryAgentService),
        Box::new(InMemoryCatalogService),
        Box::new(InMemorySessionService::new()),
        Box::new(InMemoryPtyService::new()),
        Box::new(InMemoryPermissionService::new()),
        Box::new(InMemoryQuestionService::new()),
        Box::new(InMemoryFileSystemService),
        Box::new(InMemoryIntegrationService),
        Box::new(InMemoryCredentialService),
        Box::new(InMemoryCommandService),
        Box::new(InMemorySkillService),
        Box::new(InMemoryReferenceService),
        Box::new(InMemoryEventService),
        Box::new(InMemoryProjectCopyService),
    )
}
