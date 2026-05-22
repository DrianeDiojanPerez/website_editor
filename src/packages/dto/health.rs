use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthDto {
    pub name: &'static str,
    pub version: &'static str,
}

impl HealthDto {
    pub fn current() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
