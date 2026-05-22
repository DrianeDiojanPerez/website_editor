use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorStructure {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorStructure>,
}

impl<T: Serialize> Response<T> {
    pub fn ok(data: T) -> Self {
        Self { data: Some(data), error: None }
    }

    pub fn err(code: i32, message: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(ErrorStructure { code, message: message.into() }),
        }
    }
}
