use ring_server::routes::build_router;
use ring_server::services::self_data;
use ring_server::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("ring_server=debug,tower_http=debug")
        .init();

    let data_dir = dirs_data_dir();
    std::fs::create_dir_all(&data_dir).expect("failed to create data dir");

    let db_url = format!("sqlite:{}/ring.db?mode=rwc", data_dir);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let rings_dir = std::path::PathBuf::from(format!("{data_dir}/rings"));
    std::fs::create_dir_all(&rings_dir).expect("failed to create rings dir");

    let hub_dir = std::path::PathBuf::from(format!("{data_dir}/hub"));
    std::fs::create_dir_all(&hub_dir).expect("failed to create hub dir");

    let skills_dir = std::path::PathBuf::from(format!("{data_dir}/skills"));
    std::fs::create_dir_all(&skills_dir).expect("failed to create skills dir");

    let state = AppState::new(pool, rings_dir, hub_dir, skills_dir);
    let app = build_router(state.clone());

    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let buf = {
                    let mut guard = state_clone.dwell_buffer.lock().await;
                    std::mem::take(&mut *guard)
                };
                for (user_id, user_buf) in buf {
                    if !user_buf.is_empty() {
                        let self_dir = self_data::get_self_dir(&user_id);
                        let _ = self_data::flush_dwell_buffer(&self_dir, &user_buf);
                    }
                }
            }
        });
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7420")
        .await
        .expect("failed to bind to port 7420");

    tracing::info!("ring-server listening on http://localhost:7420");

    let shutdown = async {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("failed to install SIGINT handler");
        tokio::select! {
            _ = sigterm.recv() => tracing::info!("received SIGTERM, shutting down gracefully"),
            _ = sigint.recv() => tracing::info!("received SIGINT, shutting down gracefully"),
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .expect("server error");
}

fn dirs_data_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.ring")
}
