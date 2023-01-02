use super::xstypes::Set;

/// Fundamental facet (pt. 2, §4.2)
///
/// The `{value}` property is the only item in each of the enum's variant's data
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FundamentalFacet {
    /// Schema Component: ordered, a kind of Fundamental Facet (pt. 2, §4.2.1)
    Ordered(OrderedValue),
    /// Schema Component: bounded, a kind of Fundamental Facet (pt. 2, §4.2.2)
    Bounded(bool),
    /// Schema Component: cardinality, a kind of Fundamental Facet (pt. 2, §4.2.3)
    Cardinality(CardinalityValue),
    /// Schema Component: numeric, a kind of Fundamental Facet (pt. 2, §4.2.4)
    Numeric(bool),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OrderedValue {
    False,
    Partial,
    Total,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CardinalityValue {
    Finite,
    CountablyInfinite,
}

#[derive(Clone, Debug)]
/// Abstraction for `Set<FundamentalFacet>`
pub struct FundamentalFacetSet(Set<FundamentalFacet>);

impl FundamentalFacetSet {
    pub fn empty() -> Self {
        Self(Set::new())
    }

    pub fn new(inner: Set<FundamentalFacet>) -> Self {
        Self(inner)
    }

    pub fn ordered(&self) -> Option<OrderedValue> {
        self.0.iter().find_map(|facet| match facet {
            FundamentalFacet::Ordered(ordered) => Some(*ordered),
            _ => None,
        })
    }

    pub fn bounded(&self) -> Option<bool> {
        self.0.iter().find_map(|facet| match facet {
            FundamentalFacet::Bounded(bounded) => Some(*bounded),
            _ => None,
        })
    }

    pub fn cardinality(&self) -> Option<CardinalityValue> {
        self.0.iter().find_map(|facet| match facet {
            FundamentalFacet::Cardinality(cardinality) => Some(*cardinality),
            _ => None,
        })
    }

    pub fn numeric(&self) -> Option<bool> {
        self.0.iter().find_map(|facet| match facet {
            FundamentalFacet::Numeric(numeric) => Some(*numeric),
            _ => None,
        })
    }
}
