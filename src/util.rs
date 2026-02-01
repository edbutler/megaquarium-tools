// pattern: Functional Core

use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
struct BasicError {
    pub message: String,
}

pub fn error<S: Into<String>>(msg: S) -> Box<dyn Error> {
    let err = BasicError { message: msg.into() };
    Box::new(err)
}

impl fmt::Display for BasicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BasicError {}

macro_rules! as_str_display {
    ($t:ident) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}
pub(crate) use as_str_display;
