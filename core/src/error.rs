use std::sync::mpsc::SendError;

// ErrorReason enums represent possible errors that the Speak interpreter
// binding functions may return.
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorReason {
    // Unknown,
    Syntax,
    Runtime,
    System,
    Assert,
}

impl ErrorReason {
    pub fn string(&self) -> String {
        match self {
            ErrorReason::Syntax => tr!("errors.const.syntax"),
            ErrorReason::Runtime => tr!("errors.const.runtime"),
            ErrorReason::System => tr!("errors.const.system"),
            ErrorReason::Assert => tr!("errors.const.assert"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Err {
    pub reason: ErrorReason,
    pub message: String,
}

impl Err {
    pub fn string(&self) -> String {
        format!("{}: {}", self.reason.string(), self.message)
    }
}

impl From<std::io::Error> for Err {
    fn from(err: std::io::Error) -> Self {
        Err {
            reason: ErrorReason::System,
            message: err.to_string(),
        }
    }
}

impl<T> From<SendError<T>> for Err {
    fn from(err: SendError<T>) -> Self {
        Err {
            reason: ErrorReason::System,
            message: err.to_string(),
        }
    }
}
