rust_i18n::i18n!("locales");

// rust-i18n 4 returns Cow<str>; the interpreter's messages are owned Strings.
macro_rules! tr {
    ($($args:tt)*) => {
        ::rust_i18n::t!($($args)*).to_string()
    };
}

pub mod error;
pub mod eval;
pub mod lexer;
pub mod log;
pub mod parser;
pub mod runtime;

/// The words that open an indented block (`if`, `for`) in the given locale.
pub fn block_keywords(locale: &str) -> Vec<String> {
    vec![
        tr!("literals.if", locale = locale),
        tr!("literals.for", locale = locale),
    ]
}

/// Looks up a REPL string (`repl.<key>`) in the given locale; `arg` fills `%{a}`.
pub fn repl_text(locale: &str, key: &str, arg: &str) -> String {
    let key = format!("repl.{}", key);
    tr!(&key, locale = locale, a = arg)
}
