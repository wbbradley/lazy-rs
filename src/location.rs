use nom_locate::LocatedSpan;
pub type LocationFilename = &'static str;

#[derive(Copy, Clone)]
pub struct Location {
    pub filename: LocationFilename,
    pub line: u32,
    pub col: u32,
}

impl Location {
    pub fn unknown() -> Self {
        Self {
            filename: "<unknown>",
            line: 0,
            col: 0,
        }
    }
}

impl From<&LocatedSpan<&str, LocationFilename>> for Location {
    fn from(span: &LocatedSpan<&str, LocationFilename>) -> Self {
        Self {
            filename: span.extra,
            line: span.location_line(),
            col: span.naive_get_utf8_column() as u32,
        }
    }
}
impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.col)
    }
}

impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.col)
    }
}
