use moonpub::{Command, Options, run};

fn main() -> anyhow::Result<()> {
    let options = Options::parse(std::env::args().skip(1))?;
    let output = run(&options)?;
    if !output.is_empty() {
        println!("{output}");
    }
    if matches!(options.command, Command::Help) {
        return Ok(());
    }
    Ok(())
}
