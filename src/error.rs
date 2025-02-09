#![allow(dead_code)]
use nom_language::error::VerboseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(String, &'static std::panic::Location<'static>);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: cvr error: {}", self.1, self.0)
    }
}

impl From<crate::value::IdError> for Error {
    #[track_caller]
    fn from(e: crate::value::IdError) -> Self {
        Self(format!("id error: {e}"), std::panic::Location::caller())
    }
}
impl From<crate::value::CtorIdError> for Error {
    #[track_caller]
    fn from(e: crate::value::CtorIdError) -> Self {
        Self(
            format!("constructor error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl From<std::io::Error> for Error {
    #[track_caller]
    fn from(e: std::io::Error) -> Self {
        Self(format!("io error: {e}"), std::panic::Location::caller())
    }
}
impl From<nom::Err<VerboseError<&str>>> for Error {
    #[track_caller]
    fn from(e: nom::Err<VerboseError<&str>>) -> Self {
        Self(
            format!("parsing error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl From<nom::Err<nom::error::Error<&str>>> for Error {
    #[track_caller]
    fn from(e: nom::Err<nom::error::Error<&str>>) -> Self {
        Self(format!("nom error: {}", e), std::panic::Location::caller())
    }
}
impl From<std::num::ParseIntError> for Error {
    #[track_caller]
    fn from(e: std::num::ParseIntError) -> Self {
        Self(
            format!("number parsing error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl From<String> for Error {
    #[track_caller]
    fn from(e: String) -> Self {
        Self(e, std::panic::Location::caller())
    }
}
impl From<&'static str> for Error {
    #[track_caller]
    fn from(e: &'static str) -> Self {
        Self(e.to_string(), std::panic::Location::caller())
    }
}
