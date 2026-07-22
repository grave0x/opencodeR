use axum::{extract::{Path as AxumPath, Query, State}, http::StatusCode, response::IntoResponse, Json};
use opencode_protocol::query::{FindQuery, ListQuery};
use opencode_protocol::payload::DataResponse;
use opencode_core::{FsListQuery, FsFindQuery, FsReadQuery};
use opencode_schema::schema::RelativePath;
use crate::SharedState;

pub async fn read(
    State(state): State<SharedState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let clean_path = path.trim_start_matches('/').to_string();
    let state_clone = state.clone();

    // Leetopt: move blocking I/O off the async worker thread
    let result = tokio::task::spawn_blocking(move || {
        state_clone.filesystem.read(FsReadQuery {
            path: RelativePath(clean_path),
        })
    }).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"message": format!("Task failed: {}", e)})),
    ))?;

    match result {
        Ok(content) => {
            let mime = if content.mime == "text/x-rust" { "text/plain; charset=utf-8" } else { &content.mime };
            let body = axum::body::Body::from(content.content);
            let response = axum::response::Response::builder()
                .header("Content-Type", mime)
                .body(body)
                .unwrap();
            Ok(response)
        }
        Err(e) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"message": e})))),
    }
}

pub async fn list(
    State(state): State<SharedState>,
    Query(query): Query<ListQuery>,
) -> Json<DataResponse<Vec<serde_json::Value>>> {
    Json(DataResponse {
        data: state.filesystem.list(FsListQuery { path: query.path }),
    })
}

pub async fn find(
    State(state): State<SharedState>,
    Query(query): Query<FindQuery>,
) -> Json<DataResponse<Vec<serde_json::Value>>> {
    Json(DataResponse {
        data: state.filesystem.find(FsFindQuery {
            query: query.query,
            r#type: query.r#type,
            limit: query.limit,
        }),
    })
}
