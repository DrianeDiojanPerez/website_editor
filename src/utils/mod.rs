use chrono::{DateTime, Utc};

pub fn get_current_time() -> DateTime<Utc> {
    Utc::now()
}
