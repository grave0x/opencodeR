# opencodeR Feature Roadmap

Tracking the porting of orchestration features from external sources (OctoAlly, LLM Orchestra,
LaneConductor, FORGE, Sandcastle, Concilium, LLMTrio) into opencodeR.

## Legend

| Status | Meaning |
|--------|---------|
| 🟢 Done | Implemented, tested, merged |
| 🟡 In Progress | Active development on branch |
| 🔵 Planned | Spec written, not started |
| ⚪ Not Applicable | Out of scope for opencodeR |
| 🔴 Blocked | Dependency missing |

---

## Features

### 1. 🟡 Session table with status, tokens, cost

**Sources:** OctoAlly, LLM Orchestra
**Orchestrator status:** Done — dashboard.rs
**opencodeR status:** Partial — `/api/session` returns session info with `tokens` and `cost` fields.
**Gap:** No status field on sessions, no cost aggregation endpoint.
**Plan:**
- [ ] Add `status` field to `SessionInfo` schema (active/archived/error)
- [ ] Add `GET /api/session/stats` endpoint for aggregated cost/token data
- [ ] Update TUI dashboard with sessions table view

---

### 2. 🟢 Real-time event stream

**Sources:** LLM Orchestra, LaneConductor
**Orchestrator status:** Done — StreamEvents gRPC
**opencodeR status:** ✅ `GET /api/session/:id/event` returns SSE with replay + live streaming.
**Notes:** Implemented via `tokio::sync::broadcast`. Session events pushed on create/prompt/switch.

---

### 3. 🔵 Pause / Freeze / Continue / Kill

**Sources:** FORGE, OctoAlly
**Orchestrator status:** Proto defined, TUI not wired
**opencodeR status:** Missing
**Plan:**
- [ ] Add pause/freeze/continue/kill endpoints to session API
- [ ] Add `status` field to sessions (active/paused/frozen/terminated)
- [ ] Wire TUI controls for session lifecycle
- [ ] Add `POST /api/session/:id/pause`
- [ ] Add `POST /api/session/:id/resume`
- [ ] Add `POST /api/session/:id/terminate`
- [ ] Add `POST /api/session/:id/freeze`

---

### 4. 🔵 Cost breakdown by model/provider

**Sources:** LLM Orchestra, LLMTrio
**Orchestrator status:** Missing — only per-session total
**opencodeR status:** Missing — session has `cost: f64` but no breakdown
**Plan:**
- [ ] Add `cost_breakdown` field to `SessionInfo`:
  ```rust
  struct CostBreakdown {
      by_provider: HashMap<String, f64>,
      by_model: HashMap<String, f64>,
      input_tokens: u64,
      output_tokens: u64,
      reasoning_tokens: u64,
  }
  ```
- [ ] Add `GET /api/cost/summary` — aggregate costs across all sessions
- [ ] Add `GET /api/cost/by-provider` — costs grouped by provider
- [ ] Add `GET /api/cost/by-model` — costs grouped by model
- [ ] Wire TUI cost panel

---

### 5. 🔵 Session detail / trace viewer

**Sources:** LLM Orchestra
**Orchestrator status:** Missing — proto has GetSessionLog
**opencodeR status:** Partial — `GET /api/session/:id/message` returns messages, `GET /api/session/:id/event` returns events
**Plan:**
- [ ] Add structured tool call tracing to session events
- [ ] Add `GET /api/session/:id/trace` — full execution trace with tool calls, LLM requests/responses
- [ ] Build TUI trace viewer panel (expandable message tree, tool call details)
- [ ] Support JSONL export of traces for debugging

---

### 6. 🔵 Built-in per-session terminal console

**Sources:** Concilium, OctoAlly
**Orchestrator status:** Missing — no I/O streaming in proto
**opencodeR status:** Partial — PTY API exists, WebSocket connect stub works, no terminal UI
**Plan:**
- [ ] Wire PTY process stdin/stdout to WebSocket in `connect` handler
- [ ] Build a terminal widget in the TUI (or integrate with xterm.js via web)
- [ ] Add `POST /api/session/:id/terminal` — create terminal for a session
- [ ] Support terminal resize signals via WebSocket

---

### 7. 🔵 Kanban / workspace grouping

**Sources:** LaneConductor, LLMTrio
**Orchestrator status:** Missing — working_directory field exists in Session
**opencodeR status:** Missing — no workspace/grouping concept
**Plan:**
- [ ] Add `group` / `workspace` field to `SessionInfo`
- [ ] Add `GET /api/session?group=<id>` filter
- [ ] Add `PATCH /api/session/:id/group` — move session to group
- [ ] Build TUI kanban view (columns by status/group)
- [ ] Add drag-and-drop between columns (TUI via mouse events)

---

### 8. 🔵 Multi-select + batch operations

**Sources:** LaneConductor
**Orchestrator status:** Missing
**opencodeR status:** Missing
**Plan:**
- [ ] Add batch endpoints:
  - `POST /api/session/batch/archive`
  - `POST /api/session/batch/delete`
  - `POST /api/session/batch/export`
  - `POST /api/session/batch/change-group`
- [ ] Add multi-select mode to TUI sessions list (checkbox, space to toggle)
- [ ] Add batch action bar in TUI

---

### 9. 🔵 Audit log + JSONL export

**Sources:** Sandcastle, plan.md Sprint 2
**Orchestrator status:** Proto defined (GetSessionLog), TUI not wired
**opencodeR status:** Partial — `GET /api/session/:id/history` returns events, no JSONL export
**Plan:**
- [ ] Add `GET /api/session/:id/log?format=jsonl` — streaming JSONL export
- [ ] Add `opencodeR export --format jsonl` flag
- [ ] Add global audit log endpoint: `GET /api/audit-log`
- [ ] Add TUI audit log viewer (filterable by event type, session, time range)

---

### 10. 🔵 Omnibox / command palette (NL commands)

**Sources:** Sandcastle, FORGE
**Orchestrator status:** Missing
**opencodeR status:** Missing
**Plan:**
- [ ] Add `opencodeR command <natural language>` — interpret and execute
- [ ] Build TUI command palette (Ctrl+P / Cmd+P) with fuzzy search
- [ ] Supported commands: create session, switch agent, export, search, filter, kill
- [ ] Use LLM to parse natural language commands (optional, falls back to keyword matching)

---

### 11. ⚪ Resource limits per instance (CPU/mem)

**Sources:** FORGE
**Orchestrator status:** Missing
**opencodeR status:** Not applicable — opencodeR manages sessions in-process, not containers
**Notes:** Would require container runtime integration (Docker/K8s). Out of scope for initial port.

---

### 12. 🔵 Session search / filter

**Sources:** (common)
**Orchestrator status:** Missing
**opencodeR status:** Partial — `GET /api/session?search=<q>&agent=<id>&model=<ref>` query params exist but are not implemented in the in-memory store
**Plan:**
- [ ] Implement search filtering in `InMemorySessionService::list()`:
  - [ ] Full-text search on `title`
  - [ ] Filter by `agent`
  - [ ] Filter by `model`
  - [ ] Filter by date range (`before`, `after`)
  - [ ] Filter by status
- [ ] Add TUI search bar with real-time filtering

---

### 13. ⚪ Cost optimization / model routing

**Sources:** FORGE, LLMTrio
**Orchestrator status:** Out of scope — orchestrator monitors, doesn't route
**opencodeR status:** Out of scope — opencodeR sessions use the model they're configured with

---

### 14. ⚪ Parallel agent execution

**Sources:** LLMTrio
**Orchestrator status:** Out of scope — opencode sessions are independent
**opencodeR status:** Out of scope — parallel sessions are already independent

---

### 15. ⚪ Specialist agent library

**Sources:** OctoAlly (36 agents)
**Orchestrator status:** Out of scope — opencode is the agent
**opencodeR status:** Out of scope — opencodeR is the agent runtime

---

## Implementation order

The features are ordered by impact-to-effort ratio:

1. **Session search/filter** (#12) — quick win, fills existing API contract
2. **Cost breakdown** (#4) — moderate, enables cost visibility
3. **Session status + lifecycle** (#3) — unlocks pause/freeze/kill
4. **Session trace viewer** (#5) — builds on existing event stream
5. **Audit log + JSONL export** (#9) — complements export feature
6. **Per-session terminal** (#6) — completes PTY feature
7. **Multi-select + batch** (#8) — productivity multiplier
8. **Kanban / workspace** (#7) — advanced organization
9. **Omnibox / command palette** (#10) — UX polish
