use std::process::ExitCode;

use workspace::config::Config;

fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = workspace::ui::run(config) {
        eprintln!("Error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
