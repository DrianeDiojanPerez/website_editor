use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorStructure {
    pub code: i32,
    pub message: String,
    // Per-field errors for validation failures. Omitted from the JSON output
    // unless populated, so non-validation errors stay slim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorStructure>,
}

#[derive(Debug, Serialize)]
pub struct SuccessStructure<T: Serialize> {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
}

impl<T: Serialize> Response<T> {
    pub fn ok(data: T) -> Self {
        Self { data: Some(data), error: None }
    }

    pub fn err(code: i32, message: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(ErrorStructure {
                code,
                message: message.into(),
                fields: None,
            }),
        }
    }

    pub fn err_with_fields(
        code: i32,
        message: impl Into<String>,
        fields: HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            data: None,
            error: Some(ErrorStructure {
                code,
                message: message.into(),
                fields: Some(fields),
            }),
        }
    }
}
