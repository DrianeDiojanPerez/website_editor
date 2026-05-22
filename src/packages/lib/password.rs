// Bcrypt is CPU-bound and takes ~250ms at DEFAULT_COST — wrap in spawn_blocking
// so we don't stall the async runtime.

pub async fn hash(password: String) -> anyhow::Result<String> {
    let hashed = tokio::task::spawn_blocking(move || {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    })
    .await??;
    Ok(hashed)
}

pub async fn verify(password: String, hashed: String) -> bool {
    tokio::task::spawn_blocking(move || bcrypt::verify(password, &hashed))
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or(false)
}
