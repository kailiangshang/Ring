use axum::routing::{delete, get, post};
use axum::Router;

use crate::handlers::ai;
use crate::handlers::blueprint;
use crate::handlers::conversation;
use crate::handlers::graph;
use crate::handlers::install;
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

    let conversation_routes = Router::new()
        .route("/", get(conversation::list).post(conversation::create))
        .route("/{convId}", get(conversation::get))
        .route(
            "/{convId}/messages",
            get(conversation::get_messages).post(conversation::send_message),
        );

    let blueprint_routes = Router::new()
        .route("/templates", get(blueprint::list_templates))
        .route("/chat", post(blueprint::blueprint_chat))
        .route("/preview", post(blueprint::preview_blueprint))
        .route("/confirm", post(blueprint::confirm_blueprint));

    let graph_routes = Router::new()
        .route("/", get(graph::list_graphs))
        .route("/{graphId}", get(graph::get_graph))
        .route("/{graphId}/nodes", post(graph::create_node))
        .route(
            "/{graphId}/nodes/{nodeId}",
            get(graph::get_node)
                .put(graph::update_node)
                .delete(graph::delete_node),
        )
        .route(
            "/{graphId}/nodes/{nodeId}/content",
            get(graph::get_node_content),
        )
        .route("/{graphId}/edges", post(graph::create_edge))
        .route("/{graphId}/edges/{edgeId}", delete(graph::delete_edge));

    Router::new()
        .nest("/api/v1/setup", setup_routes)
        .nest("/api/v1/rings", ring_routes)
        .nest("/api/v1/rings/{ringId}/conversations", conversation_routes)
        .nest("/api/v1/rings/{ringId}/blueprint", blueprint_routes)
        .nest("/api/v1/rings/{ringId}/graphs", graph_routes)
        .route("/api/v1/super-ring/chat", post(ai::super_ring_chat))
        .route("/join", get(install::join_page))
        .with_state(state)
}
