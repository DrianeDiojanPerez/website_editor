use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::packages::model::item::Item;

#[derive(Debug, Deserialize)]
pub struct NewItemDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ItemDto {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Item> for ItemDto {
    fn from(m: Item) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
