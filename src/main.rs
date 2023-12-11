use clap::{Parser, Subcommand};
use core::{
    log::{log_interactive, log_safe_err},
    runtime::Context,
};
use std::{
    env,
    io::{self, BufReader},
};

static SPEAK: &str = "SPEAK";

/// The `Speak` CLI Interpreter.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct SpeakCLI {
    // CLI ARGUMENTS
    #[clap(subcommand)]
    command: Commands,

    /// Log all interpreter debug information.
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Runs the `Speak` file provided.
    Run { file_path: String },
    ///  Initializes an interactive repl session to start typing Speak expressions.
    Repl,
}

fn main() {
    let speak_cli = SpeakCLI::parse();
    let mut ctx = Context::new(&speak_cli.verbose);

    let mut speak = "en".to_string();
    if let Ok(speak_) = env::var(SPEAK) {
        speak = speak_;
    }

    match speak_cli.command {
        Commands::Run { file_path } => match ctx.exec_path(&speak, &file_path) {
            Ok(val) => log_interactive(&format!("{}\n", val.string())),
            Err(err) => log_safe_err(&err.reason, &err.message),
        },
        Commands::Repl => loop {
            let mut input = String::new();
            log_interactive("\n> ");

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    // TODO: implement locales for exit
                    if input.trim().eq_ignore_ascii_case("exit") {
                        return;
                    }

                    match ctx.exec(&speak, BufReader::new(input.as_bytes())) {
                        Ok((val, _, _)) => {
                            log_interactive(&val.string());
                        }
                        Err(err) => {
                            log_safe_err(&err.reason, &err.message);
                        }
                    }
                }
                Err(err) => {
                    log_safe_err(&core::error::ErrorReason::System, &err.to_string());
                }
            }
        },
    }
}
