use ring_server::routes::build_router;
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

    let state = AppState::new(pool, rings_dir, hub_dir);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7420")
        .await
        .expect("failed to bind to port 7420");

    tracing::info!("ring-server listening on http://localhost:7420");
    axum::serve(listener, app).await.expect("server error");
}

fn dirs_data_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.ring")
}
