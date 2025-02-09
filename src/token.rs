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
            location: Location {
                filename: span.extra,
                line: span.location_line(),
                col: span.naive_get_utf8_column() as u32,
            },
        }
    }
}
