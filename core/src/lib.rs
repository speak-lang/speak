pub mod error;
pub mod eval;
pub mod lexer;
pub mod log;
pub mod parser;
pub mod runtime;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "language")]
use self::error::{Err, ErrorReason};
use serde_derive::Deserialize;
#[cfg(feature = "language")]
use std::{env, fs};
#[cfg(feature = "language")]
static LANGUAGE_CONF_FILE: &str = "assets/languages.toml";

lazy_static! {
    static ref LANGUAGE_CONF: LanguageConf = {
        #[cfg(feature = "language")]
        {
            match option_env!("LANG") {
                Some(x) => make_for_lang(x),
                None => LanguageConf::default(),
            }
        }
        #[cfg(not(feature = "language"))]
        {
            LanguageConf::default()
        }
    };
}

#[derive(Deserialize)]
pub struct LanguageConf {
    // Keywords.
    number_: String,
    string_: String,
    bool_: String,
    true_: String,
    false_: String,
    if_: String,
    is_: String,

    // Builtin functions.
    print: String,
    sprint: String,
    println: String,
    len: String,
}

impl Default for LanguageConf {
    fn default() -> Self {
        Self {
            number_: "number".to_string(),
            bool_: "bool".to_string(),
            string_: "string".to_string(),
            true_: "true".to_string(),
            false_: "false".to_string(),
            if_: "if".to_string(),
            is_: "is".to_string(),
            print: "print".to_string(),
            sprint: "sprint".to_string(),
            println: "println".to_string(),
            len: "len".to_string(),
        }
    }
}

#[cfg(feature = "language")]
fn make_for_lang(lang: &str) -> LanguageConf {
    match lang {
        "english" => LanguageConf::default(),
        _ => get_language_config(lang).unwrap(), // OK, explodes at compile time
    }
}

#[cfg(feature = "language")]
fn get_language_config(language: &str) -> Result<LanguageConf, Err> {
    let languages = fs::read(
        {
            env::current_dir()
                .expect("there must be a wd")
                .join(LANGUAGE_CONF_FILE)
        }
        .to_str()
        .expect("the path should resolve to a string"),
    )
    .expect("the file is part of the interpreter assets");

    match toml::from_slice::<toml::Value>(&languages) {
        Ok(lang_conf) => match lang_conf {
            toml::Value::Table(tables) => {
                for (table, contents) in tables {
                    if table == language {
                        return match contents.try_into::<LanguageConf>() {
                            Ok(conf) => Ok(conf),
                            Err(err) => Err(Err {
                                message: format!("unable to parse language mappings: {err}"),
                                reason: ErrorReason::Assert,
                            }),
                        };
                    }
                }
                Err(Err {
                    message: format!(
                        "language {language} could not be found the configuration file"
                    ),
                    reason: ErrorReason::System,
                })
            }
            _ => Err(Err {
                message: "toml format expected table with lanaguage name".to_string(),
                reason: ErrorReason::Assert,
            }),
        },
        Err(err) => Err(Err {
            message: format!("unable to parse languages config file: {err}"),
            reason: ErrorReason::System,
        }),
    }
}
