pub mod app;
pub mod article;
pub mod cli;
pub mod config;
pub mod error;
pub mod export;
pub mod json_util;
pub mod preview;
pub mod push;
pub mod render;
pub mod status;
pub mod system;

mod cover;
mod fetch;
mod footer;
mod humanize;
mod illustrate;
mod publish;
mod radar;
mod theme;
mod wechat;

pub use wechat::WechatClient;

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn temp_root(name: &str) -> io::Result<PathBuf> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("moonpub-{name}-{nanos}"));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    pub fn create_file(path: &Path, content: &str) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::app::run;
    use crate::cli::{Command, Options};
    use crate::test_helpers::temp_root;

    #[test]
    fn json_output_wraps_text() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("json-status")?;
        let options = Options {
            vault: root.clone(),
            command: Command::Status,
            json: true,
            config: None,
        };

        let output = run(&options)?;

        assert!(output.starts_with("{\"output\":\""));
        assert!(output.ends_with('}'));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
