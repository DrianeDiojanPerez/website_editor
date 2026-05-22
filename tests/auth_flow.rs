//! End-to-end integration tests that drive the full Axum router
//! (handler → service → repository → SQLx) against an in-memory SQLite.
//!
//! Requests are sent through the router in-process via
//! `tower::ServiceExt::oneshot` — no real socket, no real port, no flakiness
//! from binding addresses. Each test gets its own fresh in-memory database.
//!
//! Run with `cargo test`. Requires the Ed25519 key files under `keys/` to
//! exist (run `just gen-keys` once).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use website_editor::api::handler::configure_handlers;
use website_editor::api::routes::router;
use website_editor::packages::lib::jwt::new_token_manager;
use website_editor::packages::repository::new_store;
use website_editor::packages::store::sqlite::{new_sqlite_db, SqliteConfig};

// Build a fresh app instance backed by an in-memory database. Migrations run
// during `new_sqlite_db`, so every test starts from a clean schema.
async fn make_app() -> axum::Router {
    // `.env` carries the JWT_*_KEY paths. dotenv is a no-op if already loaded.
    let _ = dotenvy::dotenv();

    let pool = new_sqlite_db(SqliteConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 1,
    })
    .await
    .expect("in-memory sqlite + migrations");

    let store = new_store(pool);
    let token_manager =
        new_token_manager().expect("Ed25519 keys must exist — run `just gen-keys`");
    let handler = configure_handlers(store, token_manager);
    router(handler)
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response is JSON")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get_with_auth(uri: &str, bearer: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("authorization", format!("Bearer {bearer}"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn health_endpoint_returns_name_and_version() {
    let app = make_app().await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["name"], "website_editor");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn protected_route_without_token_returns_401() {
    let app = make_app().await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn signup_with_invalid_payload_returns_structured_field_errors() {
    let app = make_app().await;

    let resp = app
        .oneshot(post(
            "/api/v1/auth/signup",
            json!({"username": "ab", "email": "nope", "password": "x"}),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = body_json(resp).await;
    assert_eq!(body["error"]["code"], 2005);
    assert_eq!(body["error"]["message"], "validation failed");

    // Every failing field should appear in the `fields` map.
    let fields = &body["error"]["fields"];
    assert!(fields["username"].is_array(), "missing `username` field errors");
    assert!(fields["email"].is_array(), "missing `email` field errors");
    assert!(fields["password"].is_array(), "missing `password` field errors");
}

#[tokio::test]
async fn signup_then_use_access_token_to_hit_protected_route() {
    let app = make_app().await;

    // 1. signup
    let resp = app
        .clone()
        .oneshot(post(
            "/api/v1/auth/signup",
            json!({
                "username": "alice",
                "email": "alice@example.com",
                "password": "longenough",
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = body_json(resp).await;
    let access = body["data"]["token"]["access_token"]
        .as_str()
        .expect("access_token present")
        .to_string();
    let refresh = body["data"]["token"]["refresh_token"]
        .as_str()
        .expect("refresh_token present")
        .to_string();
    assert!(!access.is_empty());
    assert!(!refresh.is_empty());

    // 2. protected route succeeds with the token
    let resp = app
        .clone()
        .oneshot(get_with_auth("/api/v1/users", &access))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. refresh token rotation: original refresh works once, then is dead
    let resp = app
        .clone()
        .oneshot(post(
            "/api/v1/auth/refresh",
            json!({"refresh_token": refresh}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .oneshot(post(
            "/api/v1/auth/refresh",
            json!({"refresh_token": refresh}),
        ))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "rotated refresh token should be unusable"
    );
}

#[tokio::test]
async fn admin_created_user_must_change_password_before_using_other_routes() {
    let app = make_app().await;

    // Bootstrap an "admin" via signup (signup doesn't force a reset).
    let resp = app
        .clone()
        .oneshot(post(
            "/api/v1/auth/signup",
            json!({
                "username": "admin",
                "email": "admin@example.com",
                "password": "admin1234",
            }),
        ))
        .await
        .unwrap();
    let admin_access = body_json(resp).await["data"]["token"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Admin creates Bob — must_change_password gets flipped on.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("authorization", format!("Bearer {admin_access}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "bobby",
                        "email": "bob@example.com",
                        "password": "temp1234",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let bob_body = body_json(resp).await;
    assert_eq!(bob_body["data"]["must_change_password"], true);

    // Bob logs in.
    let resp = app
        .clone()
        .oneshot(post(
            "/api/v1/auth/login",
            json!({"email": "bob@example.com", "password": "temp1234"}),
        ))
        .await
        .unwrap();
    let bob_access = body_json(resp).await["data"]["token"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Protected route is gated → 428 Precondition Required.
    let resp = app
        .clone()
        .oneshot(get_with_auth("/api/v1/users", &bob_access))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::PRECONDITION_REQUIRED);

    // change-password endpoint IS reachable while gated.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/change-password")
                .header("authorization", format!("Bearer {bob_access}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "current_password": "temp1234",
                        "new_password": "newpass123",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // After re-login, Bob can hit the protected route.
    let resp = app
        .clone()
        .oneshot(post(
            "/api/v1/auth/login",
            json!({"email": "bob@example.com", "password": "newpass123"}),
        ))
        .await
        .unwrap();
    let bob_access2 = body_json(resp).await["data"]["token"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = app
        .oneshot(get_with_auth("/api/v1/users", &bob_access2))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
