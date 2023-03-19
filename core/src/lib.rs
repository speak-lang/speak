pub mod error;
pub mod eval;
pub mod lexer;
pub mod log;
pub mod parser;
pub mod runtime;

#[macro_use]
extern crate lazy_static;

rust_i18n::i18n!("locales");

lazy_static! {
    static ref SPEAK: String = {
        match option_env!("SPEAK") {
            Some(x) => x.to_string(),
            None => "en".to_string(),
        }
    };
}
