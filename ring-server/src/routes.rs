use axum::routing::{get, post};
use axum::Router;

use crate::handlers::ring;
use crate::handlers::setup;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let setup_routes = Router::new()
        .route("/status", get(setup::get_status))
        .route("/username", post(setup::set_username))
        .route("/llm", post(setup::set_llm))
        .route("/gitlab", post(setup::set_gitlab))
        .route("/complete", post(setup::complete));

    let ring_routes = Router::new()
        .route("/", get(ring::list_rings).post(ring::create_ring))
        .route(
            "/{ringId}",
            get(ring::get_ring)
                .put(ring::update_ring)
                .delete(ring::delete_ring),
        );

    Router::new()
        .nest("/api/v1/setup", setup_routes)
        .nest("/api/v1/rings", ring_routes)
        .with_state(state)
}
