use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "you must be logged in before doing this operation")]
    LoginRequired,
    #[fail(display = "io error")]
    IoError,
    #[fail(display = "login error")]
    LoginError,
    #[fail(display = "received invalid response")]
    InvalidResponse,
    #[fail(display = "network error")]
    NetworkError,
}

/* ----------- failure boilerplate ----------- */
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.inner)?;
        writeln!(f, "{:#?}", self.cause())
    }
}

impl Error {
    /*pub const fn new(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }*/
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

//conversion

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        e.context(ErrorKind::IoError).into()
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        e.context(ErrorKind::IoError).into()
    }
}
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        e.context(ErrorKind::NetworkError).into()
    }
}
impl From<html_extractor::Error> for Error {
    fn from(e: html_extractor::Error) -> Self {
        e.context(ErrorKind::InvalidResponse).into()
    }
}
