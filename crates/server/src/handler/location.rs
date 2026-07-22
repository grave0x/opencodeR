use axum::Json;
use opencode_schema::location::{LocationInfo, LocationKind, LocationRef};

pub async fn get() -> Json<LocationInfo> {
    Json(LocationInfo {
        id: LocationRef("local".into()),
        name: Some("Local".into()),
        workspace_id: None,
        kind: LocationKind::Local,
    })
}
