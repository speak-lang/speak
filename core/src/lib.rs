pub mod error;
pub mod eval;
pub mod lexer;
pub mod log;
pub mod parser;
pub mod runtime;

#[macro_use]
extern crate lazy_static;

rust_i18n::i18n!("locales");

/// Looks up a REPL string (`repl.<key>`) in the given locale; `arg` fills `%{a}`.
pub fn repl_text(locale: &str, key: &str, arg: &str) -> String {
    let key = format!("repl.{}", key);
    rust_i18n::t!(&key, locale = locale, a = arg)
}
