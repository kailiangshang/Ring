use axum::http::HeaderValue;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[cfg(debug_assertions)]
use tower_http::services::ServeDir;

#[cfg(not(debug_assertions))]
use axum::extract::Request;
#[cfg(not(debug_assertions))]
use axum::response::{IntoResponse, Response};
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
#[cfg(not(debug_assertions))]
static UI_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");

use crate::state::AppState;

mod archive;
mod blueprint;
mod chat;
mod config;
mod export;
mod graph;
mod group_docs;
mod health;
mod invite;
mod join_page;
mod members;
mod mode;
mod notification;
mod prompts;
mod rings;
mod self_data;
mod session;
mod setup;
mod skills;
mod super_chat;
mod upload;
mod ws;

#[cfg(not(debug_assertions))]
fn serve_embedded(path: &str) -> Response {
    let file_path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    if let Some(file) = UI_DIR.get_file(file_path) {
        let mime = mime_guess::from_path(file_path).first_or_octet_stream();
        let body = file.contents();
        (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
            body.to_vec(),
        )
            .into_response()
    } else if !file_path.contains('.')
        && !file_path.starts_with("assets/")
        && !file_path.ends_with(".js")
        && !file_path.ends_with(".css")
    {
        if let Some(index) = UI_DIR.get_file("index.html") {
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "text/html")],
                index.contents().to_vec(),
            )
                .into_response()
        } else {
            axum::http::StatusCode::NOT_FOUND.into_response()
        }
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

#[cfg(not(debug_assertions))]
async fn embedded_ui_handler(req: Request) -> Response {
    serve_embedded(req.uri().path())
}

pub fn build_router(state: AppState) -> Router {
    let localhost = [
        "http://localhost:5173",
        "http://localhost:7420",
        "http://127.0.0.1:5173",
        "http://127.0.0.1:7420",
    ]
    .iter()
    .map(|o| o.parse::<HeaderValue>().unwrap())
    .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(localhost)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::HeaderName::from_static("x-ring-token"),
        ]);

    let api = Router::new()
        .route("/health", get(health::health_check))
        .route("/prompts", get(prompts::list_prompts))
        .route("/ws", get(ws::ws_handler))
        .route("/setup/status", get(setup::get_status))
        .route("/setup/recover", get(setup::recover_token))
        .route("/setup", post(setup::submit_setup).put(setup::update_setup))
        .route("/rings", get(rings::list_rings).post(rings::create_ring))
        .route("/rings/{ring_id}", get(rings::get_ring).delete(rings::delete_ring))
        .route(
            "/rings/{ring_id}/members",
            get(members::list_members).post(members::add_member),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}/role",
            put(members::update_role),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}/grant-session",
            post(members::grant_session),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}/revoke-session",
            post(members::revoke_session),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}",
            delete(members::remove_member),
        )
        .route(
            "/config/llm",
            get(config::get_llm_config).put(config::update_llm_config),
        )
        .route("/config/llm/test", post(config::test_llm_config))
        .route("/config/gitlab/test", post(config::test_gitlab_config))
        .route(
            "/config/privacy_filters",
            get(config::get_privacy_filters).put(config::update_privacy_filters),
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
            "/self/identity",
            get(self_data::get_identity).put(self_data::update_identity),
        )
        .route(
            "/self/style",
            get(self_data::get_style).put(self_data::update_style),
        )
        .route("/self/metrics", get(self_data::get_metrics))
        .route("/self/metrics/heartbeat", post(self_data::heartbeat))
        .route(
            "/self/personality",
            get(self_data::get_personality).put(self_data::update_personality),
        )
        .route(
            "/self/privacy",
            get(self_data::get_privacy).put(self_data::update_privacy),
        )
        .route("/self/export", get(self_data::export_data))
        .route("/self/reset", post(self_data::reset_data))
        .route("/self/memory", get(self_data::list_memories))
        .route(
            "/self/memory/{name}",
            get(self_data::get_memory)
                .put(self_data::update_memory)
                .delete(self_data::delete_memory),
        )
        .route(
            "/rings/{ring_id}/graph",
            get(graph::get_graph).post(graph::create_node_handler),
        )
        .route(
            "/rings/{ring_id}/graphs",
            get(graph::list_graphs_handler).post(graph::create_graph_handler),
        )
        .route(
            "/rings/{ring_id}/graphs/{graph_id}",
            delete(graph::delete_graph_handler),
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
            "/rings/{ring_id}/sessions/{session_id}/transfer-ownership",
            post(session::transfer_ownership),
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
        .route(
            "/rings/{ring_id}/archive/quick",
            post(archive::quick_archive_handler),
        )
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
            "/rings/{ring_id}/archives/{archive_id}/diff",
            get(archive::get_archive_diff),
        )
        .route(
            "/rings/{ring_id}/archive-queue",
            get(archive::archive_queue),
        )
        .route("/rings/{ring_id}/repo/status", get(archive::repo_status))
        .route("/rings/{ring_id}/repo/init", post(archive::init_repo))
        .route("/rings/{ring_id}/repo/git-log", get(archive::git_log))
        .route("/rings/{ring_id}/repo/revert", post(archive::git_revert))
        .route("/rings/{ring_id}/sync/bundle", get(archive::sync_bundle))
        .route("/rings/sync/import", post(archive::sync_import))
        .route(
            "/rings/{ring_id}/blueprint",
            get(blueprint::get_blueprint_handler),
        )
        .route(
            "/rings/{ring_id}/blueprint/from-template",
            post(blueprint::preview_template),
        )
        .route(
            "/rings/{ring_id}/blueprint/confirm",
            post(blueprint::confirm_blueprint_handler),
        )
        .route(
            "/rings/{ring_id}/blueprint/chat",
            post(blueprint::blueprint_chat),
        )
        .route(
            "/rings/{ring_id}/blueprint/chat/history",
            get(blueprint::blueprint_history),
        )
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
        .route("/join/info", get(invite::join_info))
        .route("/join", post(invite::join_ring))
        .route("/join/local", post(invite::local_join_handler))
        .route("/join/apply", post(invite::apply_join))
        .route("/join/apply/status", get(invite::apply_status))
        .route(
            "/rings/{ring_id}/join-requests",
            get(invite::list_join_requests_handler),
        )
        .route(
            "/rings/{ring_id}/join-requests/{request_id}/approve",
            post(invite::approve_request),
        )
        .route(
            "/rings/{ring_id}/join-requests/{request_id}/reject",
            post(invite::reject_request),
        )
        .route("/notifications", get(notification::list_notifications))
        .route(
            "/notifications/unread-count",
            get(notification::get_unread_count),
        )
        .route(
            "/notifications/{notification_id}/read",
            post(notification::mark_as_read),
        )
        .route(
            "/notifications/read-all",
            post(notification::mark_all_as_read),
        )
        .route(
            "/notifications/{notification_id}",
            delete(notification::delete_notification),
        )
        .route(
            "/rings/{ring_id}/export/chat",
            get(export::export_ring_chat),
        )
        .route(
            "/rings/{ring_id}/export/chat-pdf",
            get(export::export_ring_chat_pdf),
        )
        .route(
            "/rings/{ring_id}/export/graph",
            get(export::export_ring_graph),
        )
        .route(
            "/rings/{ring_id}/export/backup",
            get(export::export_ring_backup),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/export",
            get(export::export_session_messages),
        )
        .route("/self/export/chat", get(export::export_self_chat))
        .route("/super/export/chat", get(export::export_super_chat))
        .route(
            "/rings/{ring_id}/export/report",
            get(export::export_ai_report),
        )
        .route(
            "/rings/{ring_id}/export/node",
            get(export::export_node_markdown),
        )
        .route(
            "/super/cross-ring/query",
            post(super_chat::cross_ring_query_handler),
        )
        .route(
            "/super/cross-ring/analysis",
            post(super_chat::cross_ring_analysis_handler),
        )
        .route("/rings/{ring_id}/upload", post(upload::upload_ring_file))
        .route("/super/upload", post(upload::upload_super_file))
        .route("/upload/parse", post(upload::parse_file))
        .route(
            "/rings/{ring_id}/sessions/{session_id}/material-prep/upload",
            post(upload::upload_session_file),
        )
        .with_state(state.clone());

    let app = Router::new()
        .nest("/api", api)
        .route("/ring/join", get(join_page::join_page_handler))
        .with_state(state);

    #[cfg(debug_assertions)]
    let static_dir = std::env::var("RING_STATIC_DIR").unwrap_or_else(|_| "../ui/dist".into());
    #[cfg(debug_assertions)]
    let app =
        app.fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true));

    #[cfg(not(debug_assertions))]
    let app = app.fallback(axum::routing::any(embedded_ui_handler));

    app.layer(cors).layer(TraceLayer::new_for_http())
}
