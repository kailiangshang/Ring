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
        let data_dir = PathBuf::from(home).join(".ring");
        Config {
            port: 7420,
            database_url: format!("sqlite:{}/data/ring.db", data_dir.display()),
            data_dir,
            release_repo: "https://github.com/ring-project/ring".into(),
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
