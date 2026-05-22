use std::path::Path;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub iat: i64,
    pub exp: i64,
    // `serde(default)` so older tokens issued before this claim existed still
    // deserialize cleanly (treated as `false`).
    #[serde(default)]
    pub must_change_password: bool,
}

struct KeyPair {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl KeyPair {
    fn load(private_path: &str, public_path: &str) -> anyhow::Result<Self> {
        let private = std::fs::read(Path::new(private_path)).map_err(|e| {
            anyhow::anyhow!("failed to read private key `{private_path}`: {e}")
        })?;
        let public = std::fs::read(Path::new(public_path)).map_err(|e| {
            anyhow::anyhow!("failed to read public key `{public_path}`: {e}")
        })?;
        Ok(Self {
            encoding: EncodingKey::from_ed_pem(&private)?,
            decoding: DecodingKey::from_ed_pem(&public)?,
        })
    }
}

// Manages signing and verification for both access and refresh tokens.
// Access  tokens are short-lived JWTs (EdDSA / Ed25519).
// Refresh tokens are opaque random strings; only their SHA-256 hash is stored
// in the database, so a DB leak doesn't reveal active tokens.
pub struct TokenManager {
    access: KeyPair,
    refresh: KeyPair,
    access_minutes: i64,
    refresh_days: i64,
}

impl TokenManager {
    pub fn from_env() -> anyhow::Result<Self> {
        let access = KeyPair::load(
            &env::get_string("JWT_ACCESS_PRIVATE_KEY")?,
            &env::get_string("JWT_ACCESS_PUBLIC_KEY")?,
        )?;
        let refresh = KeyPair::load(
            &env::get_string("JWT_REFRESH_PRIVATE_KEY")?,
            &env::get_string("JWT_REFRESH_PUBLIC_KEY")?,
        )?;
        let access_minutes = std::env::var("JWT_ACCESS_EXPIRATION_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);
        let refresh_days = std::env::var("JWT_REFRESH_EXPIRATION_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        Ok(Self { access, refresh, access_minutes, refresh_days })
    }

    pub fn access_expires_in_seconds(&self) -> i64 {
        self.access_minutes * 60
    }

    pub fn refresh_expires_in_seconds(&self) -> i64 {
        self.refresh_days * 24 * 60 * 60
    }

    pub fn refresh_days(&self) -> i64 {
        self.refresh_days
    }

    pub fn issue_access(
        &self,
        user_id: i64,
        username: &str,
        must_change_password: bool,
    ) -> anyhow::Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(self.access_minutes)).timestamp(),
            must_change_password,
        };
        Ok(encode(
            &Header::new(Algorithm::EdDSA),
            &claims,
            &self.access.encoding,
        )?)
    }

    pub fn decode_access(&self, token: &str) -> anyhow::Result<Claims> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.validate_exp = true;
        let data = decode::<Claims>(token, &self.access.decoding, &validation)?;
        Ok(data.claims)
    }

    // Refresh token issuance is keyed off the refresh keypair too — we don't
    // actually need to verify a JWT signature on refresh (the DB lookup is
    // authoritative), but holding a keypair keeps the door open for switching
    // to signed refresh JWTs later without an API change.
    #[allow(dead_code)]
    pub fn issue_signed_refresh(&self, user_id: i64, username: &str) -> anyhow::Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::days(self.refresh_days)).timestamp(),
            must_change_password: false,
        };
        Ok(encode(
            &Header::new(Algorithm::EdDSA),
            &claims,
            &self.refresh.encoding,
        )?)
    }
}

pub fn new_token_manager() -> anyhow::Result<Arc<TokenManager>> {
    Ok(Arc::new(TokenManager::from_env()?))
}

// 32 cryptographically-random bytes, hex-encoded → 64-char opaque refresh token.
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_hex(s: &str) -> String {
    let hash = Sha256::digest(s.as_bytes());
    hash.iter().map(|b| format!("{b:02x}")).collect()
}
