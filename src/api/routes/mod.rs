use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use super::handler::{
    auth as auth_handler, health as health_handler, project as project_handler,
    project_member as project_member_handler, project_version as project_version_handler,
    user as user_handler, Handler,
};
use super::middlewares::jwt::{require_jwt, require_password_set};

pub fn router(handler: Arc<Handler>) -> Router {
    // Public — no JWT required.
    let public = Router::new()
        .route("/auth/signup", post(auth_handler::signup))
        .route("/auth/login", post(auth_handler::login))
        .route("/auth/refresh", post(auth_handler::refresh))
        .route("/auth/logout", post(auth_handler::logout));

    // Authenticated-but-not-fully-onboarded: only `change-password` is
    // reachable while must_change_password is set, so it skips
    // `require_password_set` but still requires a valid access token.
    let change_password_only = Router::new()
        .route("/auth/change-password", post(auth_handler::change_password))
        .route_layer(axum::middleware::from_fn_with_state(
            handler.clone(),
            require_jwt,
        ));

    // Fully-authenticated routes — must have a valid token AND a settled password.
    let protected = Router::new()
        // Users
        .route(
            "/users",
            get(user_handler::get_users).post(user_handler::create_user),
        )
        .route(
            "/users/:id",
            get(user_handler::get_user)
                .patch(user_handler::update_user)
                .delete(user_handler::delete_user),
        )
        .route(
            "/users/:id/reset-password",
            post(user_handler::force_password_reset),
        )
        // Projects
        .route(
            "/projects",
            get(project_handler::get_projects).post(project_handler::create_project),
        )
        .route(
            "/projects/:id",
            get(project_handler::get_project)
                .patch(project_handler::update_project)
                .delete(project_handler::delete_project),
        )
        // Project members
        .route(
            "/projects/:project_id/members",
            get(project_member_handler::list_project_members)
                .post(project_member_handler::attach_project_member),
        )
        .route(
            "/projects/:project_id/members/:id",
            axum::routing::patch(project_member_handler::update_project_member)
                .delete(project_member_handler::detach_project_member),
        )
        // Project versions
        .route(
            "/projects/:project_id/versions",
            get(project_version_handler::list_project_versions)
                .post(project_version_handler::snapshot_project_version),
        )
        .route(
            "/projects/:project_id/versions/:id",
            get(project_version_handler::get_project_version),
        )
        // Stacked: JWT first, then password-set check (runs inside-out).
        .route_layer(axum::middleware::from_fn(require_password_set))
        .route_layer(axum::middleware::from_fn_with_state(
            handler.clone(),
            require_jwt,
        ));

    let v1 = Router::new()
        .merge(public)
        .merge(change_password_only)
        .merge(protected);

    Router::new()
        .route("/health", get(health_handler::get_health))
        .nest("/api/v1", v1)
        .with_state(handler)
        .layer(TraceLayer::new_for_http())
}
