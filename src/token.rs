use nom_locate::LocatedSpan;

use crate::location::{Location, LocationFilename};

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub location: Location,
}

impl From<LocatedSpan<&str, LocationFilename>> for Token {
    fn from(span: LocatedSpan<&str, LocationFilename>) -> Self {
        Self {
            text: span.fragment().to_string(),
            location: (&span).into(),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
