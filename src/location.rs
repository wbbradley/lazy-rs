pub type LocationFilename = &'static str;

#[derive(Clone)]
pub struct Location {
    pub filename: LocationFilename,
    pub line: u32,
    pub col: u32,
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
