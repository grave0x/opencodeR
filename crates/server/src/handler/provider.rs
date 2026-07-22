use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_r_protocol::payload::DataResponse;
use opencode_r_protocol::error::ProviderNotFoundError;
use opencode_r_schema::provider::ProviderInfo;
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<ProviderInfo>>> {
    Json(DataResponse { data: state.catalog.list_providers() })
}

pub async fn get(
    State(state): State<SharedState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderInfo>, (StatusCode, Json<ProviderNotFoundError>)> {
    match state.catalog.get_provider(&provider_id) {
        Some(provider) => Ok(Json(provider)),
        None => {
            let pid = provider_id.clone();
            Err((
                StatusCode::NOT_FOUND,
                Json(ProviderNotFoundError {
                    provider_id,
                    message: format!("Provider not found: {}", pid),
                }),
            ))
        }
    }
}
