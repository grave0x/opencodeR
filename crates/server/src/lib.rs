use axum::{
    Router,
    middleware as axum_mw,
    routing::{get, post, patch},
};
use opencode_r_protocol::route;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub mod handler;
pub mod middleware;
pub mod state;

pub type SharedState = Arc<state::AppState>;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        // Health
        .route(route::HEALTH_PATTERN, get(handler::health::get))
        // Location
        .route(route::LOCATION_PATTERN, get(handler::location::get))
        // Agent
        .route(route::AGENT_LIST_PATTERN, get(handler::agent::list))
        // Session
        .route(
            route::SESSION_LIST_PATTERN,
            get(handler::session::list).post(handler::session::create),
        )
        .route(route::SESSION_ACTIVE_PATTERN, get(handler::session::active))
        .route(route::SESSION_GET_PATTERN, get(handler::session::get))
        .route(route::SESSION_SWITCH_AGENT_PATTERN, post(handler::session::switch_agent))
        .route(route::SESSION_SWITCH_MODEL_PATTERN, post(handler::session::switch_model))
        .route(route::SESSION_PROMPT_PATTERN, post(handler::session::prompt))
        .route(route::SESSION_COMPACT_PATTERN, post(handler::session::compact))
        .route(route::SESSION_WAIT_PATTERN, post(handler::session::wait))
        .route(route::SESSION_REVERT_STAGE_PATTERN, post(handler::session::revert_stage))
        .route(route::SESSION_REVERT_CLEAR_PATTERN, post(handler::session::revert_clear))
        .route(route::SESSION_REVERT_COMMIT_PATTERN, post(handler::session::revert_commit))
        .route(route::SESSION_CONTEXT_PATTERN, get(handler::session::context))
        .route(route::SESSION_HISTORY_PATTERN, get(handler::session::history))
        .route(route::SESSION_EVENTS_PATTERN, get(handler::session::events))
        .route(route::SESSION_INTERRUPT_PATTERN, post(handler::session::interrupt))
        .route(route::SESSION_PAUSE_PATTERN, post(handler::session::pause))
        .route(route::SESSION_RESUME_PATTERN, post(handler::session::resume))
        .route(route::SESSION_FREEZE_PATTERN, post(handler::session::freeze))
        .route(route::SESSION_TERMINATE_PATTERN, post(handler::session::terminate))
        .route(route::SESSION_MESSAGES_PATTERN, get(handler::session::messages))
        .route(route::SESSION_MESSAGE_PATTERN, get(handler::session::message))
        .route(route::SESSION_TRACE_PATTERN, get(handler::session::trace))
        // Model
        .route(route::MODEL_LIST_PATTERN, get(handler::model::list))
        // Cost
        .route(route::COST_SUMMARY_PATTERN, get(handler::session::cost_summary))
        // Provider
        .route(route::PROVIDER_LIST_PATTERN, get(handler::provider::list))
        .route(route::PROVIDER_GET_PATTERN, get(handler::provider::get))
        // Integration
        .route(route::INTEGRATION_LIST_PATTERN, get(handler::integration::list))
        .route(route::INTEGRATION_GET_PATTERN, get(handler::integration::get))
        .route(route::INTEGRATION_CONNECT_KEY_PATTERN, post(handler::integration::connect_key))
        .route(
            route::INTEGRATION_CONNECT_OAUTH_PATTERN,
            post(handler::integration::connect_oauth),
        )
        .route(
            route::INTEGRATION_ATTEMPT_STATUS_PATTERN,
            get(handler::integration::attempt_status),
        )
        .route(
            route::INTEGRATION_ATTEMPT_COMPLETE_PATTERN,
            post(handler::integration::attempt_complete),
        )
        // Credential
        .route(route::CREDENTIAL_UPDATE_PATTERN, patch(handler::credential::update))
        // Permission
        .route(
            route::PERMISSION_REQUEST_LIST_PATTERN,
            get(handler::permission::request_list),
        )
        .route(route::PERMISSION_SAVED_LIST_PATTERN, get(handler::permission::saved_list))
        .route(
            route::PERMISSION_SESSION_CREATE_PATTERN,
            get(handler::permission::session_list).post(handler::permission::session_create),
        )
        .route(route::PERMISSION_SESSION_GET_PATTERN, get(handler::permission::session_get))
        .route(
            route::PERMISSION_SESSION_REPLY_PATTERN,
            post(handler::permission::session_reply),
        )
        // Question
        .route(
            route::QUESTION_REQUEST_LIST_PATTERN,
            get(handler::question::request_list),
        )
        .route(route::QUESTION_SESSION_LIST_PATTERN, get(handler::question::session_list))
        .route(
            route::QUESTION_SESSION_REPLY_PATTERN,
            post(handler::question::session_reply),
        )
        .route(
            route::QUESTION_SESSION_REJECT_PATTERN,
            post(handler::question::session_reject),
        )
        // File System
        .route(route::FS_READ_PATTERN, get(handler::fs::read))
        .route(route::FS_LIST_PATTERN, get(handler::fs::list))
        .route(route::FS_FIND_PATTERN, get(handler::fs::find))
        // Command
        .route(route::COMMAND_LIST_PATTERN, get(handler::command::list))
        // Skill
        .route(route::SKILL_LIST_PATTERN, get(handler::skill::list))
        // Reference
        .route(route::REFERENCE_LIST_PATTERN, get(handler::reference::list))
        // PTY
        .route(
            route::PTY_LIST_PATTERN,
            get(handler::pty::list).post(handler::pty::create),
        )
        .route(
            route::PTY_GET_PATTERN,
            get(handler::pty::get).put(handler::pty::update),
        )
        .route(route::PTY_CONNECT_TOKEN_PATTERN, post(handler::pty::connect_token))
        .route(route::PTY_CONNECT_PATTERN, get(handler::pty::connect))
        // Event
        .route(route::EVENT_SUBSCRIBE_PATTERN, get(handler::event::subscribe))
        // Project Copy
        .route(route::PROJECT_COPY_CREATE_PATTERN, post(handler::project_copy::create))
        .route(route::PROJECT_COPY_REFRESH_PATTERN, post(handler::project_copy::refresh))
        // Middleware
        .layer(axum_mw::from_fn(middleware::auth::auth_middleware))
        .layer(axum_mw::from_fn(middleware::access_log::access_log_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
