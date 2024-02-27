use std::{error::Error, fmt::Display};

pub struct HttpError {
    #[allow(dead_code)] msg: Option<String>
}

impl HttpError {
    pub fn new() -> HttpError {
        HttpError { msg: Option::None }
    }
    pub fn from_str(msg: &str) -> HttpError {
        HttpError { msg: Some(msg.to_string()) }
    }
}

impl<T: Error> From<T> for HttpError {
    fn from(value: T) -> Self {
        HttpError { msg: Some(value.to_string()) }
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.msg {
                Some(msg) => write!(f, "HttpError {{\n\tmsg: {}\n}}", msg),
                None => write!(f, "HttpError {{}}")
            }
        }
}
