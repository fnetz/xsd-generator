/// Constraining facet (Part 2, §4.3)
#[derive(Clone, Debug)]
pub enum ConstrainingFacet {
    Length,
    MinLength,
    MaxLength,
    Pattern,
    Enumeration,
    WhiteSpace,
    MaxInclusive,
    MaxExclusive,
    MinInclusive,
    MinExclusive,
    TotalDigits,
    FractionDigits,
    Assertions,
    ExplicitTimezone,
}
