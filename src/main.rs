use clap::{Parser, Subcommand};
use rustyline::{DefaultEditor, error::ReadlineError};
use speak_core::{
    log::{log_interactive, log_safe_err},
    runtime::Context,
};
use std::{env, io::BufReader};

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
        Commands::Repl => repl(&mut ctx, &speak),
    }
}

/// True while `input` needs more lines: an unclosed bracket, paren or string,
/// or a block (a `name: args -> type` first line, or one starting with a block keyword
/// such as `if` or `for`) not yet ended by a blank line.
fn is_incomplete(input: &str, block_keywords: &[String]) -> bool {
    let (mut depth, mut in_str, mut prev) = (0i32, false, ' ');
    for line in input.lines() {
        for c in line.chars() {
            match c {
                '"' => in_str = !in_str,
                '/' if !in_str && prev == '/' => break,
                '(' | '[' | '{' if !in_str => depth += 1,
                ')' | ']' | '}' if !in_str => depth -= 1,
                _ => {}
            }
            prev = c;
        }
        prev = ' ';
    }
    if in_str || depth > 0 {
        return true;
    }
    let first = input.lines().next().unwrap_or("");
    let is_fn =
        !first.starts_with(char::is_whitespace) && first.contains(':') && first.contains("->");
    let is_block = first.split_whitespace().next().is_some_and(|w| {
        !first.starts_with(char::is_whitespace) && block_keywords.iter().any(|k| k == w)
    });
    (is_fn || is_block) && !input.ends_with("\n\n")
}

fn repl(ctx: &mut Context, speak: &str) {
    let t = |key: &str, arg: &str| speak_core::repl_text(speak, key, arg);
    let exit = t("exit", "");
    let block_keywords = speak_core::block_keywords(speak);
    let history = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(|h| std::path::PathBuf::from(h).join(".speak_history"));

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(err) => return log_safe_err(&speak_core::error::ErrorReason::System, &err.to_string()),
    };
    if let Some(h) = &history {
        let _ = rl.load_history(h);
    }
    log_interactive(&format!("{}\n{}\n", t("welcome", ""), t("hint", &exit)));

    let mut input = String::new();
    loop {
        let prompt = t(
            if input.is_empty() {
                "prompt"
            } else {
                "continuation"
            },
            "",
        );
        match rl.readline(&prompt) {
            Ok(line) => {
                if input.is_empty() && line.trim().eq_ignore_ascii_case(&exit) {
                    break;
                }
                input.push_str(&line);
                input.push('\n');
                if is_incomplete(&input, &block_keywords) {
                    continue;
                }
                let _ = rl.add_history_entry(input.trim_end());
                match ctx.exec(speak, BufReader::new(input.as_bytes())) {
                    Ok((val, _, _)) => log_interactive(&format!("{}\n", val.string())),
                    Err(err) => log_safe_err(&err.reason, &err.message),
                }
                input.clear();
            }
            Err(ReadlineError::Interrupted) => input.clear(),
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                log_safe_err(&speak_core::error::ErrorReason::System, &err.to_string());
                break;
            }
        }
    }
    if let Some(h) = &history {
        let _ = rl.save_history(h); // unwritable history is not worth bothering a student
    }
}

#[cfg(test)]
mod tests {
    use super::is_incomplete;

    #[test]
    fn multiline_detection() {
        let kw = ["if".to_string(), "for".to_string()];
        let is_incomplete = |s| is_incomplete(s, &kw);
        assert!(!is_incomplete("1 + 1\n"));
        assert!(is_incomplete("f: (1,\n"));
        assert!(is_incomplete("[1, 2\n"));
        assert!(is_incomplete("println \"abc\n"));
        assert!(!is_incomplete("x is [1, 2]\n"));
        assert!(!is_incomplete("x is \"(\" // (\n"));
        assert!(is_incomplete("f: n number -> number\n"));
        assert!(is_incomplete("f: n number -> number\n    n + 1\n"));
        assert!(!is_incomplete("f: n number -> number\n    n + 1\n\n"));
        assert!(is_incomplete("for i in 1..3\n"));
        assert!(is_incomplete("if x = 1\n    println x\n"));
        assert!(!is_incomplete("if x = 1\n    println x\n\n"));
        assert!(!is_incomplete("iffy is 1\n"));
    }
}
