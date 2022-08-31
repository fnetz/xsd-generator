/// Fundamental facet (pt. 2, §4.2)
///
/// The `{value}` property is the only item in each of the enum's variant's data
#[derive(Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum OrderedValue {
    False,
    Partial,
    Total,
}

#[derive(Copy, Clone, Debug)]
pub enum CardinalityValue {
    Finite,
    CountablyInfinite,
}
