use moonpub::{AppError, Command, Options, run};

fn main() {
    if let Err(error) = run_from_env() {
        eprintln!("moonpub: {error}");
        std::process::exit(1);
    }
}

fn run_from_env() -> Result<(), AppError> {
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
