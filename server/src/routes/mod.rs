use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

mod config;
mod group_docs;
mod health;
mod members;
mod mode;
mod rings;
mod setup;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health::health_check))
        .route("/setup/status", get(setup::get_status))
        .route("/setup", post(setup::submit_setup).put(setup::update_setup))
        .route("/rings", get(rings::list_rings).post(rings::create_ring))
        .route("/rings/{ring_id}", get(rings::get_ring))
        .route("/rings/{ring_id}/members", get(members::list_members))
        .route(
            "/rings/{ring_id}/members/{target_id}/role",
            put(members::update_role),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}",
            delete(members::remove_member),
        )
        .route(
            "/config/llm",
            get(config::get_llm_config).put(config::update_llm_config),
        )
        .route(
            "/rings/{ring_id}/mode",
            get(mode::get_mode).put(mode::update_mode),
        )
        .route(
            "/rings/{ring_id}/group-docs/{doc_name}",
            get(group_docs::get_group_doc).put(group_docs::update_group_doc),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new("ui/dist").append_index_html_on_directories(true))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
