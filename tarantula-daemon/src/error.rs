use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub line: u32,
    pub module: String,
    pub msg: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}] => {}", self.module, self.line, self.msg)
    }
}
