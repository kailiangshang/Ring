use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::handlers::ai;
use crate::handlers::archive;
use crate::handlers::blueprint;
use crate::handlers::conversation;
use crate::handlers::git;
use crate::handlers::graph;
use crate::handlers::install;
use crate::handlers::member;
use crate::handlers::notification;
use crate::handlers::ring;
use crate::handlers::search;
use crate::handlers::session;
use crate::handlers::settings;
use crate::handlers::setup;
use crate::handlers::ws;
use crate::middleware::auth::auth_middleware;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let setup_routes = Router::new()
        .route("/status", get(setup::get_status))
        .route("/username", post(setup::set_username))
        .route("/llm", post(setup::set_llm))
        .route("/gitlab", post(setup::set_gitlab))
        .route("/complete", post(setup::complete));

    let ring_routes = Router::new()
        .route("/join", post(member::join_ring))
        .route("/", get(ring::list_rings).post(ring::create_ring))
        .route(
            "/{ringId}",
            get(ring::get_ring)
                .put(ring::update_ring)
                .delete(ring::delete_ring),
        );

    let member_routes = Router::new()
        .route("/", get(member::list_members))
        .route("/invites", post(member::generate_invite))
        .route("/{memberId}/role", put(member::update_role))
        .route("/{memberId}", delete(member::remove_member));

    let session_routes = Router::new()
        .route(
            "/",
            post(session::create_session).get(session::list_sessions),
        )
        .route(
            "/{sessionId}",
            get(session::get_session).delete(session::delete_session),
        )
        .route("/{sessionId}/close", post(session::close_session))
        .route("/{sessionId}/leave", post(session::leave_session))
        .route("/{sessionId}/archive-toggle", put(session::toggle_archive))
        .route("/{sessionId}/invite", post(session::invite_member))
        .route("/{sessionId}/messages", get(session::get_messages).post(session::send_message));

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

    let search_routes = Router::new().route("/", post(search::search_nodes));

    let archive_routes = Router::new()
        .route("/", post(archive::archive))
        .route("/queue", get(archive::get_queue))
        .route("/{archiveId}/confirm", post(archive::confirm_archive));

    let git_routes = Router::new()
        .route("/prs", get(git::list_prs))
        .route("/prs/{prId}/merge", post(git::merge_pr))
        .route("/prs/{prId}/reject", post(git::reject_pr))
        .route("/commits", get(git::get_commit_log));

    let notification_routes = Router::new()
        .route("/", get(notification::list_notifications))
        .route("/{notificationId}", post(notification::mark_read));

    let settings_routes = Router::new().route(
        "/",
        get(settings::get_settings).put(settings::update_settings),
    );

    let protected = Router::new()
        .nest("/api/v1/rings", ring_routes)
        .nest("/api/v1/rings/{ringId}/members", member_routes)
        .nest("/api/v1/rings/{ringId}/sessions", session_routes)
        .nest("/api/v1/rings/{ringId}/conversations", conversation_routes)
        .nest("/api/v1/rings/{ringId}/blueprint", blueprint_routes)
        .nest("/api/v1/rings/{ringId}/graphs", graph_routes)
        .nest("/api/v1/rings/{ringId}/search", search_routes)
        .nest("/api/v1/rings/{ringId}/archive", archive_routes)
        .nest("/api/v1/rings/{ringId}/git", git_routes)
        .nest("/api/v1/notifications", notification_routes)
        .nest("/api/v1/settings", settings_routes)
        .route("/api/v1/super-ring/chat", post(ai::super_ring_chat))
        .route("/api/v1/ws/{ringId}", get(ws::ws_handler))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/api/v1/setup", setup_routes)
        .route("/join", get(install::join_page))
        .merge(protected)
        .with_state(state)
        .layer(CorsLayer::permissive())
}
