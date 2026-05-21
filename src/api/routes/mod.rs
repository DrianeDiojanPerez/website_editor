use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use super::handler::{
    project as project_handler, project_member as project_member_handler,
    project_version as project_version_handler, user as user_handler, Handler,
};

pub fn router(handler: Arc<Handler>) -> Router {
    let v1 = Router::new()
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
        // Project versions (immutable snapshots)
        .route(
            "/projects/:project_id/versions",
            get(project_version_handler::list_project_versions)
                .post(project_version_handler::snapshot_project_version),
        )
        .route(
            "/projects/:project_id/versions/:id",
            get(project_version_handler::get_project_version),
        );

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1", v1)
        .with_state(handler)
        .layer(TraceLayer::new_for_http())
}
