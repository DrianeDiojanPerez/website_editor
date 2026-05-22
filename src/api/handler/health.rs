use axum::Json;

use crate::packages::dto::health::HealthDto;

#[tracing::instrument]
pub async fn get_health() -> Json<HealthDto> {
    Json(HealthDto::current())
}
