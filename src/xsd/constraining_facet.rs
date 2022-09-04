use roxmltree::Node;

use super::{
    components::Component,
    xstypes::{Sequence, Set},
    Annotation, Assertion, MappingContext, Ref,
};

/// Constraining facet (pt. 2, §4.3)
#[derive(Clone, Debug)]
pub enum ConstrainingFacet {
    /// Schema Component: length, a kind of Constraining Facet (pt. 2, §4.3.1)
    Length(Length),
    /// Schema Component: minLength, a kind of Constraining Facet (pt. 2, §4.3.2)
    MinLength(Length),
    /// Schema Component: maxLength, a kind of Constraining Facet (pt. 2, §4.3.3)
    MaxLength(Length),
    /// Schema Component: pattern, a kind of Constraining Facet (pt. 2, §4.3.4)
    Pattern(Pattern),
    /// Schema Component: enumeration, a kind of Constraining Facet (pt. 2, §4.3.5)
    Enumeration(Enumeration),
    /// Schema Component: whiteSpace, a kind of Constraining Facet (pt. 2, §4.3.6)
    WhiteSpace(WhiteSpace),
    /// Schema Component: maxInclusive, a kind of Constraining Facet (pt. 2, §4.3.7)
    MaxInclusive(MinMax),
    /// Schema Component: maxExclusive, a kind of Constraining Facet (pt. 2, §4.3.8)
    MaxExclusive(MinMax),
    /// Schema Component: minExclusive, a kind of Constraining Facet (pt. 2, §4.3.9)
    MinExclusive(MinMax),
    /// Schema Component: minInclusive, a kind of Constraining Facet (pt. 2, §4.3.10)
    MinInclusive(MinMax),
    /// Schema Component: totalDigits, a kind of Constraining Facet (pt. 2, §4.3.11)
    TotalDigits(TotalDigits),
    /// Schema Component: fractionDigits, a kind of Constraining Facet (pt. 2, §4.3.12)
    FractionDigits(FractionDigits),
    /// Schema Component: assertions, a kind of Constraining Facet (pt. 2, §4.3.13)
    Assertions(Assertions),
    /// Schema Component: explicitTimezone, a kind of Constraining Facet (pt. 2, §4.3.14)
    ExplicitTimezone(ExplicitTimezone),
}

/// Common type for the length, minLength and maxLength Constraining Facets
#[derive(Clone, Debug)]
pub struct Length {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: u64,
    pub fixed: bool,
}

/// Schema Component: pattern, a kind of Constraining Facet (pt. 2, §4.3.4)
#[derive(Clone, Debug)]
pub struct Pattern {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: Set<String>, // TODO regex data type; non-empty set
}

/// Schema Component: enumeration, a kind of Constraining Facet (pt. 2, §4.3.5)
#[derive(Clone, Debug)]
pub struct Enumeration {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: Set<String>, // TODO data type?
}

/// Schema Component: whiteSpace, a kind of Constraining Facet (pt. 2, §4.3.6)
#[derive(Clone, Debug)]
pub struct WhiteSpace {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: WhiteSpaceValue,
    pub fixed: bool,
}

#[derive(Clone, Debug)]
pub enum WhiteSpaceValue {
    Preserve,
    Replace,
    Collapse,
}

/// Common type for the maxInclusive, maxExclusive, minExclusive, minInclusive Constraining Facets
#[derive(Clone, Debug)]
pub struct MinMax {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: String, // TODO data type?
    pub fixed: bool,
}

/// Schema Component: totalDigits, a kind of Constraining Facet (pt. 2, §4.3.11)
#[derive(Clone, Debug)]
pub struct TotalDigits {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: u64, // TODO positiveInteger
    pub fixed: bool,
}

/// Schema Component: fractionDigits, a kind of Constraining Facet (pt. 2, §4.3.12)
#[derive(Clone, Debug)]
pub struct FractionDigits {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: u64,
    pub fixed: bool,
}

/// Schema Component: assertions, a kind of Constraining Facet (pt. 2, §4.3.13)
#[derive(Clone, Debug)]
pub struct Assertions {
    pub annotations: Sequence<Ref<Annotation>>,
    pub assertions: Sequence<Ref<Assertion>>,
}

/// Schema Component: explicitTimezone, a kind of Constraining Facet (pt. 2, §4.3.14)
#[derive(Clone, Debug)]
pub struct ExplicitTimezone {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: ExplicitTimezoneValue,
    pub fixed: bool,
}

#[derive(Clone, Debug)]
pub enum ExplicitTimezoneValue {
    Required,
    Prohibited,
    Optional,
}

impl ConstrainingFacet {
    /// Returns `None` in case the constraining facet is not supported by the processor
    pub(super) fn map_from_xml(_context: &mut MappingContext, _facet: Node) -> Option<Ref<Self>> {
        todo!()
    }

    pub fn annotations(&self) -> &[Ref<Annotation>] {
        match self {
            ConstrainingFacet::Length(c) => &c.annotations,
            ConstrainingFacet::MinLength(c) => &c.annotations,
            ConstrainingFacet::MaxLength(c) => &c.annotations,
            ConstrainingFacet::Pattern(c) => &c.annotations,
            ConstrainingFacet::Enumeration(c) => &c.annotations,
            ConstrainingFacet::WhiteSpace(c) => &c.annotations,
            ConstrainingFacet::MaxInclusive(c) => &c.annotations,
            ConstrainingFacet::MaxExclusive(c) => &c.annotations,
            ConstrainingFacet::MinExclusive(c) => &c.annotations,
            ConstrainingFacet::MinInclusive(c) => &c.annotations,
            ConstrainingFacet::TotalDigits(c) => &c.annotations,
            ConstrainingFacet::FractionDigits(c) => &c.annotations,
            ConstrainingFacet::Assertions(c) => &c.annotations,
            ConstrainingFacet::ExplicitTimezone(c) => &c.annotations,
        }
    }
}

impl WhiteSpace {
    pub fn new(value: WhiteSpaceValue, fixed: bool) -> Self {
        Self {
            annotations: Sequence::new(),
            value,
            fixed,
        }
    }
}

impl Component for ConstrainingFacet {
    const DISPLAY_NAME: &'static str = "ConstrainingFacet";
}
