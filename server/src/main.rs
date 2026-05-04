use clap::Parser;
use ring_server::routes::build_router;
use ring_server::services::self_data;
use ring_server::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;

#[derive(Parser)]
#[command(name = "ring")]
#[command(about = "Ring - 群组知识协作空间")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// 监听端口
    #[arg(short, long, default_value = "7420")]
    port: u16,

    #[arg(short, long)]
    daemon: bool,

    #[arg(long)]
    pid_file: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter("ring_server=debug,tower_http=debug")
        .init();

    let data_dir = dirs_data_dir();

    print_banner(cli.port, &data_dir);

    if cli.daemon {
        #[cfg(unix)]
        {
            let _ = std::fs::create_dir_all(&data_dir);
            let pid_path = cli
                .pid_file
                .clone()
                .unwrap_or_else(|| format!("{}/ring.pid", data_dir));
            let exe = std::env::current_exe().expect("failed to get current executable");
            let args: Vec<String> = std::env::args()
                .skip(1)
                .filter(|a| a != "-d" && a != "--daemon")
                .collect();
            let mut cmd = std::process::Command::new(exe);
            cmd.args(&args);
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
            let child = cmd.spawn().expect("failed to spawn daemon process");
            let pid = child.id();
            std::fs::write(&pid_path, pid.to_string()).expect("failed to write pid file");
            tracing::info!("daemon started with pid {pid}");
            println!("daemon started with pid {pid}");
            std::process::exit(0);
        }
        #[cfg(not(unix))]
        {
            println!("daemon mode not supported on Windows");
        }
    }

    if let Err(e) = tokio::fs::create_dir_all(&data_dir).await {
        tracing::error!("failed to create data dir: {e}");
        std::process::exit(1);
    }

    let db_url = format!("sqlite:{}/ring.db?mode=rwc", data_dir);
    let pool = match SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("failed to connect to SQLite: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::error!("failed to run migrations: {e}");
        std::process::exit(1);
    }

    let rings_dir = std::path::PathBuf::from(format!("{data_dir}/rings"));
    if let Err(e) = tokio::fs::create_dir_all(&rings_dir).await {
        tracing::error!("failed to create rings dir: {e}");
        std::process::exit(1);
    }

    let hub_dir = std::path::PathBuf::from(format!("{data_dir}/hub"));
    if let Err(e) = tokio::fs::create_dir_all(&hub_dir).await {
        tracing::error!("failed to create hub dir: {e}");
        std::process::exit(1);
    }

    let skills_dir = std::path::PathBuf::from(format!("{data_dir}/skills"));
    if let Err(e) = tokio::fs::create_dir_all(&skills_dir).await {
        tracing::error!("failed to create skills dir: {e}");
        std::process::exit(1);
    }

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

    let addr = format!("0.0.0.0:{}", cli.port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to bind to port {}: {}", cli.port, e);
            std::process::exit(1);
        }
    };

    let startup_msg = format!("Ring server listening on http://localhost:{}", cli.port);
    println!("{}", startup_msg);
    tracing::info!("{}", startup_msg);

    let shutdown = async {
        #[cfg(unix)]
        {
            let mut sigterm =
                match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("failed to install SIGTERM handler: {e}");
                        return;
                    }
                };
            let mut sigint =
                match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("failed to install SIGINT handler: {e}");
                        return;
                    }
                };
            tokio::select! {
                _ = sigterm.recv() => tracing::info!("received SIGTERM, shutting down gracefully"),
                _ = sigint.recv() => tracing::info!("received SIGINT, shutting down gracefully"),
            }
        }
        #[cfg(windows)]
        {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("received Ctrl+C, shutting down gracefully");
        }
    };

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
    {
        tracing::error!("server error: {e}");
        std::process::exit(1);
    }

    println!("Ring stopped. Goodbye.");
    tracing::info!("Ring stopped. Goodbye.");
}

fn dirs_data_dir() -> String {
    #[cfg(target_os = "windows")]
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.ring")
}

fn print_banner(port: u16, data_dir: &str) {
    let version = env!("CARGO_PKG_VERSION");
    let cyan = "\x1b[1;36m";
    let dim = "\x1b[2;37m";
    let bold = "\x1b[1;37m";
    let reset = "\x1b[0m";
    println!();
    println!("{cyan}  ┌─────────────────────────────────────────────────┐{reset}");
    println!("{cyan}  │{reset}                                                 {cyan}│{reset}");
    println!("{cyan}  │{reset}   {bold}╔═╗╔═╗╔╦╗╔═╗╦ ╦╔═╗╦═╗{reset}                     {cyan}│{reset}");
    println!("{cyan}  │{reset}   {bold}╠╦╝║╣  ║ ║╣ ║║║║ ╦╠╦╝{reset}                     {cyan}│{reset}");
    println!("{cyan}  │{reset}   {bold}╩╚═╚═╝ ╩ ╚═╝╚╩╝╚═╝╩╚═{reset}                     {cyan}│{reset}");
    println!("{cyan}  │{reset}                                                 {cyan}│{reset}");
    println!("{cyan}  │{reset}   {dim}Group Knowledge Workspace  v{version:<14}{reset}    {cyan}│{reset}");
    println!("{cyan}  │{reset}                                                 {cyan}│{reset}");
    println!("{cyan}  ├─────────────────────────────────────────────────┤{reset}");
    println!("{cyan}  │{reset}   {dim}➜{reset}  {cyan}Local:{reset}   http://localhost:{port:<18} {cyan}│{reset}");
    println!("{cyan}  │{reset}   {dim}➜{reset}  {cyan}Data:{reset}    {data_dir:<29}{cyan}│{reset}");
    println!("{cyan}  └─────────────────────────────────────────────────┘{reset}");
    println!();
    tracing::info!("Ring v{version} starting on http://localhost:{port}, data_dir: {data_dir}");
}
