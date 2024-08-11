pub mod meta;
use std::string::String as StdString;

#[derive(Debug)]
pub struct AnyType;
#[derive(Debug)]
pub struct AnySimpleType;
#[derive(Debug)]
pub struct AnyAtomicType;
#[derive(Debug)]
pub struct Error;
#[derive(Debug)]
pub struct Decimal(pub StdString);
#[derive(Debug)]
pub struct DateTime(pub StdString);
#[derive(Debug)]
pub struct Duration(pub StdString);
#[derive(Debug)]
pub struct Time(pub StdString);
#[derive(Debug)]
pub struct Date(pub StdString);
#[derive(Debug)]
pub struct GMonth(pub StdString);
#[derive(Debug)]
pub struct GMonthDay(pub StdString);
#[derive(Debug)]
pub struct GDay(pub StdString);
#[derive(Debug)]
pub struct GYear(pub StdString);
#[derive(Debug)]
pub struct GYearMonth(pub StdString);
#[derive(Debug)]
pub struct HexBinary(pub StdString);
#[derive(Debug)]
pub struct Base64Binary(pub StdString);
#[derive(Debug)]
pub struct AnyURI(pub StdString);
impl AnyURI {
    pub fn from_literal(literal: &str) -> Result<Self, meta::Error> {
        Ok(Self(literal.to_string()))
    }
}
#[derive(Debug)]
pub struct QName(pub StdString);
#[derive(Debug)]
pub struct Notation(pub StdString);
#[derive(Debug)]
pub struct NormalizedString(pub StdString);
#[derive(Debug)]
pub struct Token(pub StdString);
#[derive(Debug)]
pub struct Language(pub StdString);
#[derive(Debug)]
pub struct NmToken(pub StdString);
#[derive(Debug)]
pub struct NmTokens(pub Vec<NmToken>);
#[derive(Debug)]
pub struct Name(pub StdString);
#[derive(Debug)]
pub struct NcName(pub StdString);
#[derive(Debug)]
pub struct Id(pub StdString);
#[derive(Debug)]
pub struct IdRef(pub StdString);
#[derive(Debug)]
pub struct IdRefs(pub Vec<IdRef>);
#[derive(Debug)]
pub struct Entity(pub StdString);
#[derive(Debug)]
pub struct Entities(pub Vec<Entity>);
#[derive(Debug)]
pub struct Integer(pub StdString);
#[derive(Debug)]
pub struct NonPositiveInteger(pub StdString);
#[derive(Debug)]
pub struct NegativeInteger(pub StdString);
#[derive(Debug)]
pub struct NonNegativeInteger(pub StdString);
#[derive(Debug)]
pub struct PositiveInteger(pub StdString);
#[derive(Debug)]
pub struct YearMonthDuration(pub StdString);
#[derive(Debug)]
pub struct DayTimeDuration(pub StdString);
#[derive(Debug)]
pub struct DateTimeStamp(pub StdString);

// Built-in types defined using native rust types, only used for literal mapping
pub struct String(());
impl String {
    pub fn from_literal(literal: &str) -> Result<StdString, meta::Error> {
        Ok(literal.to_string())
    }
}
