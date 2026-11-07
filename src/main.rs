use std::path::PathBuf;

use moonpub::app::run;
use moonpub::cli::Options;

fn main() -> anyhow::Result<()> {
    load_dotenv();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match execute(&args) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            Ok(())
        }
        Err(error) => {
            // Only callers that explicitly opted into JSON get a machine-readable
            // failure. Text mode keeps stdout empty and stderr human-readable.
            if args.iter().any(|arg| arg == "--json") {
                println!("{}", moonpub::protocol::error_json(&args, &error));
            }
            Err(error)
        }
    }
}

/// Parse and run in one place, so parse failures and runtime failures both go
/// through the same error-reporting path.
fn execute(args: &[String]) -> anyhow::Result<String> {
    let options = Options::parse(args.iter().cloned())?;
    Ok(run(&options)?)
}

/// Load .env files from cwd, ~/.moonpub.env, and XDG config dir.
/// Never overwrites existing env vars — the user's shell always wins.
fn load_dotenv() {
    let mut candidates = vec![PathBuf::from(".env")];
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(PathBuf::from(&home).join(".moonpub.env"));
    }
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim();
                    let val = val.trim().trim_matches('"').trim();
                    if std::env::var(key).is_err() {
                        // SAFETY: we have not spawned any threads yet, this runs before CLI parsing
                        unsafe {
                            std::env::set_var(key, val);
                        }
                    }
                }
            }
        }
    }
}
