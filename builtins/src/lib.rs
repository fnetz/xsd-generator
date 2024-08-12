pub mod meta;
use regex::Regex;
use std::string::String as StdString;

fn is_name_start_char(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == ':'
        || ('\u{C0}'..='\u{D6}').contains(&c)
        || ('\u{D8}'..='\u{F6}').contains(&c)
        || ('\u{F8}'..='\u{2FF}').contains(&c)
        || ('\u{370}'..='\u{37D}').contains(&c)
        || ('\u{37F}'..='\u{1FFF}').contains(&c)
        || ('\u{200C}'..='\u{200D}').contains(&c)
        || ('\u{2070}'..='\u{218F}').contains(&c)
        || ('\u{2C00}'..='\u{2FEF}').contains(&c)
        || ('\u{3001}'..='\u{D7FF}').contains(&c)
        || ('\u{F900}'..='\u{FDCF}').contains(&c)
        || ('\u{FDF0}'..='\u{FFFD}').contains(&c)
        || ('\u{10000}'..='\u{EFFFF}').contains(&c)
}

fn is_name_char(c: char) -> bool {
    is_name_start_char(c)
        || c.is_ascii_digit()
        || c == '-'
        || c == '.'
        || c == '\u{B7}'
        || ('\u{0300}'..='\u{036F}').contains(&c)
        || ('\u{203F}'..='\u{2040}').contains(&c)
}

// TODO: SimpleType impl for any* types
#[derive(Debug)]
pub struct AnyType;

#[derive(Debug)]
pub struct AnySimpleType;

#[derive(Debug)]
pub struct AnyAtomicType;

#[derive(Debug)]
pub struct Error;

impl meta::SimpleType for Error {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        // Special case of empty union instantiation, see Pt. 1, §3.16.7.3, Note
        Err(meta::Error::ErrorTypeCanNotBeInstantiated)
    }
}

#[derive(Debug)]
pub struct Decimal(pub StdString);

impl meta::SimpleType for Decimal {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        // NOTE: Not actually a pattern facet, but specified in lexical mapping
        const PATTERN: &str = r"^(\+|-)?([0-9]+(\.[0-9]*)?|\.[0-9]+)$";
        // TODO: Cache regex
        let pattern = Regex::new(PATTERN).unwrap();
        if !pattern.is_match(literal) {
            return Err(meta::Error::NotPatternValid {
                pattern: PATTERN.to_string(),
                value: literal.to_string(),
            });
        }
        Ok(Self(literal.to_string()))
    }
}

#[derive(Debug)]
pub struct DateTime(pub StdString);

impl meta::SimpleType for DateTime {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Duration(pub StdString);

impl meta::SimpleType for Duration {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Time(pub StdString);

impl meta::SimpleType for Time {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Date(pub StdString);

impl meta::SimpleType for Date {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        // TODO
        Ok(Self(literal.to_string()))
    }
}

#[derive(Debug)]
pub struct GMonth(pub StdString);

impl meta::SimpleType for GMonth {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct GMonthDay(pub StdString);

impl meta::SimpleType for GMonthDay {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct GDay(pub StdString);

impl meta::SimpleType for GDay {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct GYear(pub StdString);

impl meta::SimpleType for GYear {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct GYearMonth(pub StdString);

impl meta::SimpleType for GYearMonth {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct HexBinary(pub StdString);

impl meta::SimpleType for HexBinary {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Base64Binary(pub StdString);

impl meta::SimpleType for Base64Binary {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct AnyURI(pub StdString);

impl meta::SimpleType for AnyURI {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        // TODO
        Ok(Self(literal.to_string()))
    }
}

#[derive(Debug)]
pub struct QName(pub StdString);

impl meta::SimpleType for QName {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Notation(pub StdString);

impl meta::SimpleType for Notation {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct NormalizedString(pub StdString);

impl meta::SimpleType for NormalizedString {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Replace);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Token(pub StdString);

impl meta::SimpleType for Token {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Language(pub StdString);

impl meta::SimpleType for Language {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        // pattern: [a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*
        let pattern_valid = literal.split('-').enumerate().all(|(i, part)| {
            let content_valid = if i == 0 {
                part.chars().all(|c| c.is_ascii_alphabetic())
            } else {
                part.chars().all(|c| c.is_ascii_alphanumeric())
            };
            content_valid && (1..=8).contains(&part.len())
        });
        if !pattern_valid {
            return Err(meta::Error::NotPatternValid {
                pattern: "[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*".to_string(),
                value: literal.to_string(),
            });
        }
        Ok(Self(literal.to_string()))
    }
}

#[derive(Debug)]
pub struct NmToken(pub StdString);

impl meta::SimpleType for NmToken {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        // pattern: \c+ (where \c means "the set of name characters, those ·matched· by NameChar")
        let pattern_valid = !literal.is_empty() && literal.chars().all(is_name_char);
        if !pattern_valid {
            return Err(meta::Error::NotPatternValid {
                pattern: r"\c+".to_string(),
                value: literal.to_string(),
            });
        }
        Ok(Self(literal.to_string()))
    }
}

#[derive(Debug)]
pub struct NmTokens(pub Vec<NmToken>);

impl meta::SimpleType for NmTokens {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Name(pub StdString);

impl meta::SimpleType for Name {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        // pattern: \i\c*
        let pattern_valid = !literal.is_empty()
            && literal.chars().enumerate().all(|(i, c)| {
                if i == 0 {
                    is_name_start_char(c)
                } else {
                    is_name_char(c)
                }
            });
        if !pattern_valid {
            return Err(meta::Error::NotPatternValid {
                pattern: r"\i\c*".to_string(),
                value: literal.to_string(),
            });
        }
        Ok(Self(literal.to_string()))
    }
}

#[derive(Debug)]
pub struct NcName(pub StdString);

impl meta::SimpleType for NcName {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Id(pub StdString);

impl meta::SimpleType for Id {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct IdRef(pub StdString);

impl meta::SimpleType for IdRef {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct IdRefs(pub Vec<IdRef>);

impl meta::SimpleType for IdRefs {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Entity(pub StdString);

impl meta::SimpleType for Entity {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Entities(pub Vec<Entity>);

impl meta::SimpleType for Entities {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Integer(pub StdString);

impl meta::SimpleType for Integer {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct NonPositiveInteger(pub StdString);

impl meta::SimpleType for NonPositiveInteger {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct NegativeInteger(pub StdString);

impl meta::SimpleType for NegativeInteger {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct NonNegativeInteger(pub StdString);

impl meta::SimpleType for NonNegativeInteger {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct PositiveInteger(pub StdString);

impl meta::SimpleType for PositiveInteger {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct YearMonthDuration(pub StdString);

impl meta::SimpleType for YearMonthDuration {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct DayTimeDuration(pub StdString);

impl meta::SimpleType for DayTimeDuration {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct DateTimeStamp(pub StdString);

impl meta::SimpleType for DateTimeStamp {
    const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(meta::Whitespace::Collapse);
    fn from_literal(_literal: &str) -> Result<Self, meta::Error> {
        todo!()
    }
}

// Built-in types defined using native rust types, only used for literal mapping
pub struct String(());

impl String {
    pub fn from_literal(literal: &str) -> Result<StdString, meta::Error> {
        Ok(literal.to_string())
    }
}
