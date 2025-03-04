use std::{borrow::Cow, fmt};

#[derive(Debug)]
pub enum Error {
    ValueNotInEnumeration(String),
    NoValidBranch,
    NotPatternValid { pattern: String, value: String },
    ErrorTypeCanNotBeInstantiated,
    ElementInSimpleContentType,
    NotBoolean,
}

impl Error {
    fn constraint_name(&self) -> Option<&str> {
        match self {
            Self::ElementInSimpleContentType => Some("cvc-complex-type.1.2"),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ValueNotInEnumeration(ref value) => {
                write!(f, "Value {value:?} is not in enumeration")
            }
            Self::NoValidBranch => write!(f, "No valid branch"),
            Self::NotPatternValid {
                ref pattern,
                ref value,
            } => write!(f, "Value {value:?} does not match pattern {pattern:?}"),
            Self::ErrorTypeCanNotBeInstantiated => write!(f, "xs:error can not be instantiated"),
            Self::ElementInSimpleContentType => write!(
                f,
                "Element information item in complex type with simple content"
            ),
            Self::NotBoolean => write!(f, "Value is not a boolean"),
        }?;
        if let Some(constraint_name) = self.constraint_name() {
            write!(f, " ({} violation)", constraint_name)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub enum Whitespace {
    Preserve,
    Replace,
    Collapse,
}

pub fn normalized_value(value: &str, whitespace: Whitespace) -> Cow<str> {
    // TODO: Don't allocate if not necessary
    match whitespace {
        Whitespace::Preserve => Cow::Borrowed(value),
        Whitespace::Replace => Cow::Owned(value.replace(['\t', '\n', '\r'], " ")),
        Whitespace::Collapse => value
            .split([' ', '\t', '\n', '\r'])
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .into(),
    }
}

pub trait SimpleType: Sized {
    const FACET_WHITE_SPACE: Option<Whitespace>;

    /// Maps a literal from lexical space to value space.
    /// If the literal can not be mapped (i.e. is not part of the type's lexical space), then an
    /// error is returned.
    ///
    /// Before calling this function, the value must be normalized (e.g. using
    /// [`normalized_value`]). Any pre-lexical facet (i.e. whiteSpace) must be applied according to
    /// the type's rules.
    fn from_literal(value: &str) -> Result<Self, Error>;

    /// Maps a string to this type. The string is first normalized, and then converted using
    /// [`Self::from_literal`].
    fn from_string(value: &str) -> Result<Self, Error> {
        let normalized = Self::FACET_WHITE_SPACE
            .map(|white_space| normalized_value(value, white_space))
            .unwrap_or_else(|| Cow::Borrowed(value));
        Self::from_literal(&normalized)
    }
}

pub trait ComplexType: Sized {
    type Node<'a>;

    fn from_node(node: &Self::Node<'_>) -> Result<Self, Error>;
}
