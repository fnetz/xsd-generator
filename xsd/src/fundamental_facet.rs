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

#[derive(Copy, Clone, Debug)]
struct FundamentalFacets {
    ordered: OrderedValue,
    bounded: bool,
    cardinality: CardinalityValue,
    numeric: bool,
}

#[derive(Copy, Clone, Debug, Default)]
/// Abstraction for `Set<FundamentalFacet>`
pub struct FundamentalFacetSet(Option<FundamentalFacets>);

impl FundamentalFacetSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_values(
        ordered: OrderedValue,
        bounded: bool,
        cardinality: CardinalityValue,
        numeric: bool,
    ) -> Self {
        Self(Some(FundamentalFacets {
            ordered,
            bounded,
            cardinality,
            numeric,
        }))
    }

    // pub fn new(inner: Set<FundamentalFacet>) -> Self {
    //     Self {
    //         ordered: inner.iter().find_map(|facet| match facet {
    //             FundamentalFacet::Ordered(ordered) => Some(*ordered),
    //             _ => None,
    //         }),
    //         bounded: inner.iter().find_map(|facet| match facet {
    //             FundamentalFacet::Bounded(bounded) => Some(*bounded),
    //             _ => None,
    //         }),
    //         cardinality: inner.iter().find_map(|facet| match facet {
    //             FundamentalFacet::Cardinality(cardinality) => Some(*cardinality),
    //             _ => None,
    //         }),
    //         numeric: inner.iter().find_map(|facet| match facet {
    //             FundamentalFacet::Numeric(numeric) => Some(*numeric),
    //             _ => None,
    //         }),
    //     }
    // }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn ordered(&self) -> Option<OrderedValue> {
        self.0.map(|facets| facets.ordered)
    }

    pub fn bounded(&self) -> Option<bool> {
        self.0.map(|facets| facets.bounded)
    }

    pub fn cardinality(&self) -> Option<CardinalityValue> {
        self.0.map(|facets| facets.cardinality)
    }

    pub fn numeric(&self) -> Option<bool> {
        self.0.map(|facets| facets.numeric)
    }
}
