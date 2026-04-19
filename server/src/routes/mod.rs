use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

mod archive;
mod chat;
mod config;
mod graph;
mod group_docs;
mod health;
mod invite;
mod members;
mod mode;
mod rings;
mod session;
mod setup;
mod skills;
mod super_chat;
mod ws;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health::health_check))
        .route("/ws", get(ws::ws_handler))
        .route("/setup/status", get(setup::get_status))
        .route("/setup", post(setup::submit_setup).put(setup::update_setup))
        .route("/rings", get(rings::list_rings).post(rings::create_ring))
        .route("/rings/{ring_id}", get(rings::get_ring))
        .route(
            "/rings/{ring_id}/members",
            get(members::list_members).post(members::add_member),
        )
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
        .route("/rings/{ring_id}/chat", post(chat::ring_chat))
        .route("/rings/{ring_id}/chat/history", get(chat::ring_history))
        .route("/self/chat", post(chat::self_chat))
        .route("/self/chat/history", get(chat::self_history))
        .route(
            "/rings/{ring_id}/graph",
            get(graph::get_graph).post(graph::create_node_handler),
        )
        .route(
            "/rings/{ring_id}/graph/nodes/{node_id}",
            put(graph::update_node).delete(graph::delete_node),
        )
        .route(
            "/rings/{ring_id}/graph/edges",
            post(graph::create_edge_handler),
        )
        .route(
            "/rings/{ring_id}/graph/edges/{edge_id}",
            delete(graph::delete_edge),
        )
        .route(
            "/rings/{ring_id}/sessions",
            get(session::list_sessions).post(session::create_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}",
            get(session::get_session).delete(session::delete_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/close",
            post(session::close_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/reopen",
            post(session::reopen_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/participants",
            post(session::invite_participants),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/participants/{target_id}",
            delete(session::remove_participant),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/archive-toggle",
            put(session::archive_toggle),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/messages",
            get(session::get_messages),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/start",
            post(session::start_session_handler),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/summarize",
            post(session::summarize_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/material-prep",
            get(session::get_material_prep),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/material-prep/highlights",
            post(session::highlight_material_handler),
        )
        .route("/rings/{ring_id}/archive", post(archive::trigger_archive))
        .route("/rings/{ring_id}/archives", get(archive::list_archives))
        .route(
            "/rings/{ring_id}/archives/{archive_id}",
            get(archive::get_archive),
        )
        .route(
            "/rings/{ring_id}/archives/{archive_id}/review",
            post(archive::review_archive),
        )
        .route(
            "/rings/{ring_id}/archive-queue",
            get(archive::archive_queue),
        )
        .route("/rings/{ring_id}/repo/status", get(archive::repo_status))
        .route("/rings/{ring_id}/repo/init", post(archive::init_repo))
        .route("/super/chat", post(super_chat::super_chat_handler))
        .route("/super/chat/history", get(super_chat::super_history))
        .route(
            "/super/system-prompt",
            get(super_chat::get_system_prompt).put(super_chat::update_system_prompt),
        )
        .route(
            "/super/preferences",
            get(super_chat::get_preferences).put(super_chat::update_preferences),
        )
        .route("/skills", get(skills::list_skills))
        .route("/skills/install", post(skills::install_skill_handler))
        .route(
            "/skills/{name}",
            get(skills::get_skill_detail).delete(skills::remove_skill),
        )
        .route(
            "/rings/{ring_id}/invite-tokens",
            post(invite::create_invite_token).get(invite::list_invite_tokens),
        )
        .route(
            "/rings/{ring_id}/invite-tokens/{token}",
            delete(invite::revoke_invite_token),
        )
        .with_state(state);

    let static_dir = std::env::var("RING_STATIC_DIR").unwrap_or_else(|_| "../ui/dist".into());

    Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
