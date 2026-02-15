use std::process::ExitCode;

use bear::config::Config;

fn main() -> ExitCode {
    bear::claude_code_client::logger::init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = bear::ui::run(config) {
        eprintln!("Error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
