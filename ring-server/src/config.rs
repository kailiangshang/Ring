use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub release_repo: String,
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let data_dir = PathBuf::from(
            std::env::var("RING_DATA_DIR").unwrap_or_else(|_| format!("{}/.ring", home)),
        );
        let port: u16 = std::env::var("RING_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7420);
        let database_url = std::env::var("RING_DATABASE_URL")
            .unwrap_or_else(|_| format!("sqlite:{}/data/ring.db?mode=rwc", data_dir.display()));
        let release_repo = std::env::var("RING_RELEASE_REPO")
            .unwrap_or_else(|_| "https://github.com/ring-project/ring".into());
        Config {
            port,
            database_url,
            data_dir,
            release_repo,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_port() {
        let config = Config::default();
        assert_eq!(config.port, 7420);
    }

    #[test]
    fn default_config_has_data_dir() {
        let config = Config::default();
        assert!(config.data_dir.to_string_lossy().contains(".ring"));
    }

    #[test]
    fn default_config_has_release_repo() {
        let config = Config::default();
        assert!(!config.release_repo.is_empty());
    }
}
