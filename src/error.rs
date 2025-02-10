#![allow(dead_code)]
use nom_language::error::VerboseError;

use crate::{
    id::{IdError, IdErrorTrait},
    runtime::error::RuntimeError,
};

macro_rules! error {
    ($($arg:tt)*) => {
        PitaError::new(format!($($arg)*), std::panic::Location::caller())
    };
}
pub(crate) use error;

#[derive(Debug)]
pub struct PitaError(String, &'static std::panic::Location<'static>);

impl PitaError {
    pub fn new(msg: String, location: &'static std::panic::Location<'static>) -> Self {
        Self(msg, location)
    }
}
impl std::fmt::Display for PitaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: cvr error: {}", self.1, self.0)
    }
}

impl<C: IdErrorTrait> From<IdError<C>> for PitaError {
    #[track_caller]
    fn from(e: IdError<C>) -> Self {
        Self(format!("id error: {e}"), std::panic::Location::caller())
    }
}
impl From<crate::value::CtorIdError> for PitaError {
    #[track_caller]
    fn from(e: crate::value::CtorIdError) -> Self {
        Self(
            format!("constructor error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl From<RuntimeError> for PitaError {
    #[track_caller]
    fn from(e: RuntimeError) -> Self {
        Self(
            format!("runtime error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl From<std::io::Error> for PitaError {
    #[track_caller]
    fn from(e: std::io::Error) -> Self {
        Self(format!("io error: {e}"), std::panic::Location::caller())
    }
}
impl From<nom::Err<VerboseError<&str>>> for PitaError {
    #[track_caller]
    fn from(e: nom::Err<VerboseError<&str>>) -> Self {
        Self(
            format!("parsing error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl<T: std::fmt::Debug> From<nom::Err<nom::error::Error<T>>> for PitaError {
    #[track_caller]
    fn from(e: nom::Err<nom::error::Error<T>>) -> Self {
        Self(format!("nom error: {}", e), std::panic::Location::caller())
    }
}
impl From<std::num::ParseIntError> for PitaError {
    #[track_caller]
    fn from(e: std::num::ParseIntError) -> Self {
        Self(
            format!("number parsing error: {e}"),
            std::panic::Location::caller(),
        )
    }
}
impl From<String> for PitaError {
    #[track_caller]
    fn from(e: String) -> Self {
        Self(e, std::panic::Location::caller())
    }
}
impl From<&'static str> for PitaError {
    #[track_caller]
    fn from(e: &'static str) -> Self {
        Self(e.to_string(), std::panic::Location::caller())
    }
}
