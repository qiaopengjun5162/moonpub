mod ai;
mod ai_workflow;
pub mod app;
mod app_article_commands;
mod app_draft_follow_up;
mod app_publish_commands;
mod app_support;
pub mod article;
pub mod bundle;
pub mod cli;
pub mod config;
pub mod draft;
pub mod error;
pub mod evidence;
pub mod export;
pub mod init;
pub mod intake;
pub mod json_util;
mod layout_audit;
pub mod markdown;
pub mod plugin;
pub mod preflight;
pub mod preview;
pub mod protocol;
pub mod push;
mod push_browser;
pub mod release_check;
pub mod render;
pub mod ship;
pub mod status;
pub mod system;

mod cdp;
mod cover;
mod fetch;
mod footer;
mod humanize;
mod illustrate;
mod publish;
mod publish_steps;
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
    fn status_json_returns_structured_payload() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("json-status")?;
        let options = Options {
            articles: root.clone(),
            command: Command::Status,
            json: true,
            config: None,
        };

        let output = run(&options)?;

        assert!(output.starts_with("{\"command\":\"status\",\"stages\":["));
        assert!(output.contains(r#""stage":"drafts""#));
        assert!(output.contains(r#""stage":"ready""#));
        assert!(output.contains(r#""stage":"published""#));
        assert!(output.contains(r#""next_command":"moonpub new \"你的第一篇文章\"""#));
        assert!(
            output
                .contains(r#""next_step":"create your first article draft to start the workflow""#)
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workspace_json_returns_structured_payload() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("json-workspace")?;
        let options = Options {
            articles: root.clone(),
            command: Command::Workspace,
            json: true,
            config: None,
        };

        let output = run(&options)?;

        assert!(output.starts_with("{\"command\":\"workspace\""));
        assert!(output.contains(r#""workspace_kind":"local-publishing-core""#));
        assert!(output.contains(r#""entry_path":"existing-markdown""#));
        assert!(output.contains(r#""next_command":"moonpub new \"你的第一篇文章\"""#));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
