#[macro_use]
extern crate lazy_static;
mod core;
use crate::core::{
    log::{log_interactive, log_safe_err},
    runtime::Context,
};
use clap::{Parser, Subcommand};
use std::io::{self, BufReader};

/// The `Speak` CLI Interpreter
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct SpeakCLI {
    // CLI ARGUEMENTS
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

    match speak_cli.command {
        Commands::Run { file_path } => match ctx.exec_path(file_path) {
            Ok(val) => log_interactive(&format!("{}\n", val.string())),
            Err(err) => log_safe_err(&err.reason, &err.message),
        },
        Commands::Repl => loop {
            let mut input = String::new();
            log_interactive("\n> ");

            match io::stdin().read_line(&mut input) {
                Ok(_) => match ctx.exec(BufReader::new(input.as_bytes())) {
                    Ok(val) => {
                        log_interactive(&val.string());
                    }
                    Err(err) => {
                        log_safe_err(&err.reason, &err.message);
                    }
                },
                Err(err) => {
                    log_safe_err(&core::error::ErrorReason::System, &err.to_string());
                }
            }
        },
    }
}
