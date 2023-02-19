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
    fn string(&self) -> String {
        match self {
            ErrorReason::Syntax => "Syntax error".to_string(),
            ErrorReason::Runtime => "Runtime error".to_string(),
            ErrorReason::System => "System error".to_string(),
            ErrorReason::Assert => "Assertion error".to_string(),
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
