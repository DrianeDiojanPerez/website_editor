use async_trait::async_trait;
use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::packages::codes;
use crate::packages::service::{err_from_validation, ServiceError};

// Drop-in replacement for axum's `Json<T>` extractor that:
//   1. Deserializes the request body into `T` (returns 400 with a JSON error
//      message if the body is malformed).
//   2. Runs `T::validate()` and returns a 400 with `{field: [messages]}` if
//      any field-level rules fail.
//
// Handlers consume it the same way: `ValidatedJson(dto): ValidatedJson<MyDto>`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ServiceError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|e| {
            ServiceError::new(
                format!("invalid JSON body: {e}"),
                codes::ERR_VALIDATION,
            )
        })?;

        value.validate().map_err(err_from_validation)?;

        Ok(ValidatedJson(value))
    }
}
