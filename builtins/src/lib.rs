#[derive(Debug)]
pub struct AnyType;
#[derive(Debug)]
pub struct AnySimpleType;
#[derive(Debug)]
pub struct AnyAtomicType;
#[derive(Debug)]
pub struct Error;
#[derive(Debug)]
pub struct Decimal(pub String);
#[derive(Debug)]
pub struct DateTime(pub String);
#[derive(Debug)]
pub struct Duration(pub String);
#[derive(Debug)]
pub struct Time(pub String);
#[derive(Debug)]
pub struct Date(pub String);
#[derive(Debug)]
pub struct GMonth(pub String);
#[derive(Debug)]
pub struct GMonthDay(pub String);
#[derive(Debug)]
pub struct GDay(pub String);
#[derive(Debug)]
pub struct GYear(pub String);
#[derive(Debug)]
pub struct GYearMonth(pub String);
#[derive(Debug)]
pub struct HexBinary(pub String);
#[derive(Debug)]
pub struct Base64Binary(pub String);
#[derive(Debug)]
pub struct AnyURI(pub String);
#[derive(Debug)]
pub struct QName(pub String);
#[derive(Debug)]
pub struct Notation(pub String);
#[derive(Debug)]
pub struct NormalizedString(pub String);
#[derive(Debug)]
pub struct Token(pub String);
#[derive(Debug)]
pub struct Language(pub String);
#[derive(Debug)]
pub struct NmToken(pub String);
#[derive(Debug)]
pub struct NmTokens(pub Vec<NmToken>);
#[derive(Debug)]
pub struct Name(pub String);
#[derive(Debug)]
pub struct NcName(pub String);
#[derive(Debug)]
pub struct Id(pub String);
#[derive(Debug)]
pub struct IdRef(pub String);
#[derive(Debug)]
pub struct IdRefs(pub Vec<IdRef>);
#[derive(Debug)]
pub struct Entity(pub String);
#[derive(Debug)]
pub struct Entities(pub Vec<Entity>);
#[derive(Debug)]
pub struct Integer(pub String);
#[derive(Debug)]
pub struct NonPositiveInteger(pub String);
#[derive(Debug)]
pub struct NegativeInteger(pub String);
#[derive(Debug)]
pub struct NonNegativeInteger(pub String);
#[derive(Debug)]
pub struct PositiveInteger(pub String);
#[derive(Debug)]
pub struct YearMonthDuration(pub String);
#[derive(Debug)]
pub struct DayTimeDuration(pub String);
#[derive(Debug)]
pub struct DateTimeStamp(pub String);
