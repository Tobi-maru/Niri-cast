use std::process::Command;

#[derive(Debug, Default)]
pub struct SystemAdapter;

impl SystemAdapter {
    pub fn command_exists(&self, name: &str) -> bool {
        Command::new("which")
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn service_is_active(&self, service: &str) -> bool {
        Command::new("systemctl")
            .args(["--user", "is-active", service])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    pub fn path_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    pub fn any_path_exists(&self, paths: &[&str]) -> bool {
        paths.iter().any(|path| self.path_exists(path))
    }
}
