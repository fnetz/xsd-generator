use std::fmt;

use crate::xstypes::QName;

#[derive(Debug)]
pub enum XsdError {
    NamePrefixNotResolved(String),
    AbsentComponentValue,
    UnnamedTopLevelElement,
    UnknownTopLevelElement(String),
    UnresolvedReference(QName),
    UnresolvedBuiltin(&'static QName),
}

impl fmt::Display for XsdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamePrefixNotResolved(prefix) => {
                write!(f, "Failed to resolve prefix {prefix:?} to a namespace URI")
            }
            Self::AbsentComponentValue => {
                write!(f, "Component value is absent")
            }
            Self::UnnamedTopLevelElement => {
                write!(f, "Top-level element is unnamed")
            }
            Self::UnknownTopLevelElement(name) => {
                write!(f, "Unknown top-level element {name:?}")
            }
            Self::UnresolvedReference(name) => {
                write!(f, "Unresolved reference {name:?}")
            }
            Self::UnresolvedBuiltin(name) => {
                write!(f, "Unresolved builtin {name:?}")
            }
        }
    }
}

impl std::error::Error for XsdError {}
