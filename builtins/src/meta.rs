use std::{borrow::Cow, fmt};

#[derive(Debug)]
pub enum Error {
    ValueNotInEnumeration(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ValueNotInEnumeration(ref value) => {
                write!(f, "Value {value:?} is not in enumeration")
            }
        }
    }
}

impl std::error::Error for Error {}

pub enum Whitespace {
    Preserve,
    Replace,
    Collapse,
}

pub fn normalized_value(value: &str, whitespace: Whitespace) -> Cow<str> {
    match whitespace {
        Whitespace::Preserve => Cow::Borrowed(value),
        Whitespace::Replace => Cow::Owned(
            value
                .replace('\t', " ")
                .replace('\n', " ")
                .replace('\r', " "),
        ),
        Whitespace::Collapse => value.split_whitespace().collect(), // FIXME: Not correct
    }
}
