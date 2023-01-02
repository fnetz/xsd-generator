use std::fmt;

#[derive(Debug)]
pub enum XsdError {
    NamePrefixNotResolved(String),
}

impl fmt::Display for XsdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamePrefixNotResolved(prefix) => {
                write!(f, "Failed to resolve prefix {prefix:?} to a namespace URI")
            }
        }
    }
}

impl std::error::Error for XsdError {}
