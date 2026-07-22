/// Path prefix for all API routes
pub const API_PREFIX: &str = "/api";

// ---- Health ----
pub const HEALTH: &str = "/api/health";

// axum path patterns (with :param syntax)
pub const HEALTH_PATTERN: &str = "/api/health";

// ---- Location ----
pub const LOCATION: &str = "/api/location";
pub const LOCATION_PATTERN: &str = "/api/location";

// ---- Agent ----
pub const AGENT_LIST: &str = "/api/agent";
pub const AGENT_LIST_PATTERN: &str = "/api/agent";

// ---- Session ----
pub const SESSION_LIST: &str = "/api/session";
pub const SESSION_CREATE: &str = "/api/session";
pub const SESSION_ACTIVE: &str = "/api/session/active";
pub const SESSION_LIST_PATTERN: &str = "/api/session";
pub const SESSION_ACTIVE_PATTERN: &str = "/api/session/active";
pub const SESSION_GET_PATTERN: &str = "/api/session/:sessionID";
pub const SESSION_SWITCH_AGENT_PATTERN: &str = "/api/session/:sessionID/agent";
pub const SESSION_SWITCH_MODEL_PATTERN: &str = "/api/session/:sessionID/model";
pub const SESSION_PROMPT_PATTERN: &str = "/api/session/:sessionID/prompt";
pub const SESSION_COMPACT_PATTERN: &str = "/api/session/:sessionID/compact";
pub const SESSION_WAIT_PATTERN: &str = "/api/session/:sessionID/wait";
pub const SESSION_REVERT_STAGE_PATTERN: &str = "/api/session/:sessionID/revert/stage";
pub const SESSION_REVERT_CLEAR_PATTERN: &str = "/api/session/:sessionID/revert/clear";
pub const SESSION_REVERT_COMMIT_PATTERN: &str = "/api/session/:sessionID/revert/commit";
pub const SESSION_CONTEXT_PATTERN: &str = "/api/session/:sessionID/context";
pub const SESSION_HISTORY_PATTERN: &str = "/api/session/:sessionID/history";
pub const SESSION_EVENTS_PATTERN: &str = "/api/session/:sessionID/event";
pub const SESSION_INTERRUPT_PATTERN: &str = "/api/session/:sessionID/interrupt";
pub const SESSION_PAUSE_PATTERN: &str = "/api/session/:sessionID/pause";
pub const SESSION_RESUME_PATTERN: &str = "/api/session/:sessionID/resume";
pub const SESSION_FREEZE_PATTERN: &str = "/api/session/:sessionID/freeze";
pub const SESSION_TERMINATE_PATTERN: &str = "/api/session/:sessionID/terminate";
pub const SESSION_GROUP_PATTERN: &str = "/api/session/:sessionID/group";
pub const SESSION_GROUPS_PATTERN: &str = "/api/session/groups";
pub const SESSION_MESSAGES_PATTERN: &str = "/api/session/:sessionID/message";
pub const SESSION_MESSAGE_PATTERN: &str = "/api/session/:sessionID/message/:messageID";
pub const SESSION_DELETE_PATTERN: &str = "/api/session/:sessionID";
pub const SESSION_TRACE_PATTERN: &str = "/api/session/:sessionID/trace";
pub fn session_get(id: &str) -> String {
    format!("/api/session/{id}")
}
pub fn session_switch_agent(id: &str) -> String {
    format!("/api/session/{id}/agent")
}
pub fn session_switch_model(id: &str) -> String {
    format!("/api/session/{id}/model")
}
pub fn session_prompt(id: &str) -> String {
    format!("/api/session/{id}/prompt")
}
pub fn session_compact(id: &str) -> String {
    format!("/api/session/{id}/compact")
}
pub fn session_wait(id: &str) -> String {
    format!("/api/session/{id}/wait")
}
pub fn session_revert_stage(id: &str) -> String {
    format!("/api/session/{id}/revert/stage")
}
pub fn session_revert_clear(id: &str) -> String {
    format!("/api/session/{id}/revert/clear")
}
pub fn session_revert_commit(id: &str) -> String {
    format!("/api/session/{id}/revert/commit")
}
pub fn session_context(id: &str) -> String {
    format!("/api/session/{id}/context")
}
pub fn session_history(id: &str) -> String {
    format!("/api/session/{id}/history")
}
pub fn session_events(id: &str) -> String {
    format!("/api/session/{id}/event")
}
pub fn session_interrupt(id: &str) -> String {
    format!("/api/session/{id}/interrupt")
}
pub fn session_pause(id: &str) -> String {
    format!("/api/session/{id}/pause")
}
pub fn session_resume(id: &str) -> String {
    format!("/api/session/{id}/resume")
}
pub fn session_freeze(id: &str) -> String {
    format!("/api/session/{id}/freeze")
}
pub fn session_terminate(id: &str) -> String {
    format!("/api/session/{id}/terminate")
}
pub fn session_message(id: &str, msg_id: &str) -> String {
    format!("/api/session/{id}/message/{msg_id}")
}
pub fn session_messages(id: &str) -> String {
    format!("/api/session/{id}/message")
}
pub fn session_permission_create(id: &str) -> String {
    format!("/api/session/{id}/permission")
}
pub fn session_permission_list(id: &str) -> String {
    format!("/api/session/{id}/permission")
}
pub fn session_permission_get(id: &str, req_id: &str) -> String {
    format!("/api/session/{id}/permission/{req_id}")
}
pub fn session_permission_reply(id: &str, req_id: &str) -> String {
    format!("/api/session/{id}/permission/{req_id}/reply")
}
pub fn session_question_list(id: &str) -> String {
    format!("/api/session/{id}/question")
}
pub fn session_question_reply(id: &str, req_id: &str) -> String {
    format!("/api/session/{id}/question/{req_id}/reply")
}
pub fn session_question_reject(id: &str, req_id: &str) -> String {
    format!("/api/session/{id}/question/{req_id}/reject")
}

// ---- Model ----
pub const MODEL_LIST: &str = "/api/model";
pub const MODEL_LIST_PATTERN: &str = "/api/model";

// ---- Cost ----
pub const COST_SUMMARY: &str = "/api/cost/summary";
pub const COST_SUMMARY_PATTERN: &str = "/api/cost/summary";

// ---- Audit Log ----
pub const AUDIT_LOG: &str = "/api/audit-log";
pub const AUDIT_LOG_PATTERN: &str = "/api/audit-log";

// ---- Provider ----
pub const PROVIDER_LIST: &str = "/api/provider";
pub const PROVIDER_LIST_PATTERN: &str = "/api/provider";
pub const PROVIDER_GET_PATTERN: &str = "/api/provider/:providerID";
pub fn provider_get(id: &str) -> String {
    format!("/api/provider/{id}")
}

// ---- Integration ----
pub const INTEGRATION_LIST: &str = "/api/integration";
pub const INTEGRATION_LIST_PATTERN: &str = "/api/integration";
pub const INTEGRATION_GET_PATTERN: &str = "/api/integration/:integrationID";
pub const INTEGRATION_CONNECT_KEY_PATTERN: &str = "/api/integration/:integrationID/connect/key";
pub const INTEGRATION_CONNECT_OAUTH_PATTERN: &str = "/api/integration/:integrationID/connect/oauth";
pub const INTEGRATION_ATTEMPT_STATUS_PATTERN: &str = "/api/integration/attempt/:attemptID";
pub const INTEGRATION_ATTEMPT_COMPLETE_PATTERN: &str = "/api/integration/attempt/:attemptID/complete";
pub fn integration_get(id: &str) -> String {
    format!("/api/integration/{id}")
}
pub fn integration_connect_key(id: &str) -> String {
    format!("/api/integration/{id}/connect/key")
}
pub fn integration_connect_oauth(id: &str) -> String {
    format!("/api/integration/{id}/connect/oauth")
}
pub fn integration_attempt_status(attempt_id: &str) -> String {
    format!("/api/integration/attempt/{attempt_id}")
}
pub fn integration_attempt_complete(attempt_id: &str) -> String {
    format!("/api/integration/attempt/{attempt_id}/complete")
}
pub fn integration_attempt_cancel(attempt_id: &str) -> String {
    format!("/api/integration/attempt/{attempt_id}")
}

// ---- Credential ----
pub const CREDENTIAL_UPDATE_PATTERN: &str = "/api/credential/:credentialID";
pub fn credential_update(id: &str) -> String {
    format!("/api/credential/{id}")
}
pub fn credential_remove(id: &str) -> String {
    format!("/api/credential/{id}")
}

// ---- Permission ----
pub const PERMISSION_REQUEST_LIST: &str = "/api/permission/request";
pub const PERMISSION_SAVED_LIST: &str = "/api/permission/saved";
pub const PERMISSION_REQUEST_LIST_PATTERN: &str = "/api/permission/request";
pub const PERMISSION_SAVED_LIST_PATTERN: &str = "/api/permission/saved";
pub const PERMISSION_SESSION_CREATE_PATTERN: &str = "/api/session/:sessionID/permission";
pub const PERMISSION_SESSION_LIST_PATTERN: &str = "/api/session/:sessionID/permission";
pub const PERMISSION_SESSION_GET_PATTERN: &str = "/api/session/:sessionID/permission/:requestID";
pub const PERMISSION_SESSION_REPLY_PATTERN: &str = "/api/session/:sessionID/permission/:requestID/reply";
pub fn permission_saved_remove(id: &str) -> String {
    format!("/api/permission/saved/{id}")
}

// ---- Question ----
pub const QUESTION_REQUEST_LIST: &str = "/api/question/request";
pub const QUESTION_REQUEST_LIST_PATTERN: &str = "/api/question/request";
pub const QUESTION_SESSION_LIST_PATTERN: &str = "/api/session/:sessionID/question";
pub const QUESTION_SESSION_REPLY_PATTERN: &str = "/api/session/:sessionID/question/:requestID/reply";
pub const QUESTION_SESSION_REJECT_PATTERN: &str = "/api/session/:sessionID/question/:requestID/reject";

// ---- File System ----
pub const FS_READ: &str = "/api/fs/read";
pub const FS_LIST: &str = "/api/fs/list";
pub const FS_FIND: &str = "/api/fs/find";
pub const FS_READ_PATTERN: &str = "/api/fs/read/*path";
pub const FS_LIST_PATTERN: &str = "/api/fs/list";
pub const FS_FIND_PATTERN: &str = "/api/fs/find";

// ---- Command ----
pub const COMMAND_LIST: &str = "/api/command";
pub const COMMAND_LIST_PATTERN: &str = "/api/command";

// ---- Skill ----
pub const SKILL_LIST: &str = "/api/skill";
pub const SKILL_LIST_PATTERN: &str = "/api/skill";

// ---- Reference ----
pub const REFERENCE_LIST: &str = "/api/reference";
pub const REFERENCE_LIST_PATTERN: &str = "/api/reference";

// ---- PTY ----
pub const PTY_LIST: &str = "/api/pty";
pub const PTY_CREATE: &str = "/api/pty";
pub const PTY_LIST_PATTERN: &str = "/api/pty";
pub const PTY_CREATE_PATTERN: &str = "/api/pty";
pub const PTY_GET_PATTERN: &str = "/api/pty/:ptyID";
pub const PTY_UPDATE_PATTERN: &str = "/api/pty/:ptyID";
pub const PTY_CONNECT_TOKEN_PATTERN: &str = "/api/pty/:ptyID/connect-token";
pub const PTY_CONNECT_PATTERN: &str = "/api/pty/:ptyID/connect";
pub fn pty_get(id: &str) -> String {
    format!("/api/pty/{id}")
}
pub fn pty_update(id: &str) -> String {
    format!("/api/pty/{id}")
}
pub fn pty_remove(id: &str) -> String {
    format!("/api/pty/{id}")
}
pub fn pty_connect_token(id: &str) -> String {
    format!("/api/pty/{id}/connect-token")
}
pub fn pty_connect(id: &str) -> String {
    format!("/api/pty/{id}/connect")
}

// ---- Event (SSE) ----
pub const EVENT_SUBSCRIBE: &str = "/api/event";
pub const EVENT_SUBSCRIBE_PATTERN: &str = "/api/event";

// ---- Project Copy ----
pub const PROJECT_COPY_CREATE_PATTERN: &str = "/experimental/project/:projectID/copy";
pub const PROJECT_COPY_REFRESH_PATTERN: &str = "/experimental/project/:projectID/copy/refresh";
pub fn project_copy_create(project_id: &str) -> String {
    format!("/experimental/project/{project_id}/copy")
}
pub fn project_copy_remove(project_id: &str) -> String {
    format!("/experimental/project/{project_id}/copy")
}
pub fn project_copy_refresh(project_id: &str) -> String {
    format!("/experimental/project/{project_id}/copy/refresh")
}
