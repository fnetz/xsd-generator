use crate::{
    Annotation, Assertion, MappingContext, Ref,
    components::{Component, ComponentTable},
    values::{ActualValue, actual_value},
    xstypes::{Sequence, Set},
};
use roxmltree::Node;
use std::fmt;

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

/// Container for constraining facets. Allows access to the individual facets, while hiding the
/// actual storage structure.
#[derive(Clone, Default)]
pub struct ConstrainingFacets {
    facets: Vec<Ref<ConstrainingFacet>>,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub value: Sequence<Ref<Assertion>>,
}

/// Schema Component: explicitTimezone, a kind of Constraining Facet (pt. 2, §4.3.14)
#[derive(Clone, Debug)]
pub struct ExplicitTimezone {
    pub annotations: Sequence<Ref<Annotation>>,
    pub value: ExplicitTimezoneValue,
    pub fixed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExplicitTimezoneValue {
    Required,
    Prohibited,
    Optional,
}

impl ConstrainingFacet {
    /// This function maps a list of facet elements to a list of [`ConstrainingFacet`]s.
    /// As there are elements (`<enumeration>`, `<pattern>`, `<assertion>`) where multiple
    /// occurrences are mapped to a single facet, the function needs knowledge of all the elements.
    /// Returns `None` in case one of the constraining facets is not supported by the processor
    pub(super) fn map_from_xml(
        context: &mut MappingContext,
        facets: &[Node],
        schema: Node,
    ) -> Option<Vec<Ref<Self>>> {
        // First, create separate groups for facets with potentially multiple elements
        let mut patterns = Vec::new();
        let mut enumerations = Vec::new();
        let mut assertions = Vec::new();
        let mut singular = Vec::new();

        for facet in facets.iter().copied() {
            let bin = match facet.tag_name().name() {
                "pattern" => &mut patterns,
                "enumeration" => &mut enumerations,
                "assertion" => &mut assertions,
                _ => &mut singular,
            };
            bin.push(facet);
        }

        let mut facets = Vec::new();

        if !patterns.is_empty() {
            // Let R be a regular expression given by the appropriate case among the following:
            let r = if patterns.len() == 1 {
                // If there is only one <pattern> among the [children] of a <restriction>, then the
                // actual value of its value [attribute]
                actual_value::<String>(patterns[0].attribute("value").unwrap(), patterns[0])
            } else {
                // otherwise the concatenation of the actual values of all the <pattern>
                // [children]'s value [attributes], in order, separated by '|', so forming a single
                // regular expression with multiple ·branches·.
                patterns
                    .iter()
                    .map(|&p| actual_value::<&str>(p.attribute("value").unwrap(), p))
                    .collect::<Vec<_>>()
                    .join("|")
            };

            // The value is then given by the appropriate case among the following:
            // 1 If the {base type definition} of the ·owner· has a pattern facet among its
            //   {facets}, then the union of that pattern facet's {value} and {·R·}
            // 2 otherwise just {·R·}
            // TODO case 1
            let value = vec![r];

            // The annotation mapping of the set containing all of the <pattern> elements among the
            // [children] of the <restriction> element information item, as defined in section XML
            // Representation of Annotation Schema Components of [XSD 1.1 Part 1: Structures].
            let annotations = Annotation::xml_element_set_annotation_mapping(context, &patterns);

            facets.push(context.create(ConstrainingFacet::Pattern(Pattern { value, annotations })));
        }

        if !enumerations.is_empty() {
            // {value} The appropriate case among the following:
            // 1 If there is only one <enumeration> among the [children] of a <restriction>,
            //   then a set with one member, the actual value of its value [attribute],
            //   interpreted as an instance of the {base type definition}.
            // 2 otherwise a set of the actual values of all the <enumeration> [children]'s value
            //   [attributes], interpreted as instances of the {base type definition}.
            let value = enumerations
                .iter()
                .map(|&e| e.attribute("value").unwrap().to_string())
                .collect::<Vec<_>>();

            // A (possibly empty) sequence of Annotation components, one for each <annotation>
            // among the [children] of the <enumeration>s among the [children] of a <restriction>,
            // in order.
            let annotations =
                Annotation::xml_element_set_annotation_mapping(context, &enumerations);

            facets.push(context.create(ConstrainingFacet::Enumeration(Enumeration {
                value,
                annotations,
            })));
        }

        if !assertions.is_empty() {
            // A sequence whose members are Assertions drawn from the following sources, in order:
            // 1 If the {base type definition} of the ·owner· has an assertions facet among its
            //   {facets}, then the Assertions which appear in the {value} of that assertions facet.
            // 2 Assertions corresponding to the <assertion> element information items among the
            //   [children] of <restriction>, if any, in document order.
            // TODO base type assertions
            let value = assertions
                .into_iter()
                .map(|assert| Assertion::map_from_xml(context, assert, schema))
                .collect::<Vec<_>>();

            // {annotations} The empty sequence.
            let annotations = Sequence::new();

            facets.push(context.create(ConstrainingFacet::Assertions(Assertions {
                value,
                annotations,
            })))
        }

        for facet in singular {
            let facet = match facet.tag_name().name() {
                "length" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::Length(Length {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "minLength" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::MinLength(Length {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "maxLength" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::MaxLength(Length {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "whiteSpace" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::WhiteSpace(WhiteSpace {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "maxInclusive" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::MaxInclusive(MinMax {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "maxExclusive" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::MaxExclusive(MinMax {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "minExclusive" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::MinExclusive(MinMax {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "minInclusive" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::MinInclusive(MinMax {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "totalDigits" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::TotalDigits(TotalDigits {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "fractionDigits" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::FractionDigits(FractionDigits {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                "explicitTimezone" => {
                    let (value, fixed, annotations) =
                        Self::map_value_fixed_annotations(context, facet);

                    context.create(Self::ExplicitTimezone(ExplicitTimezone {
                        value,
                        fixed,
                        annotations,
                    }))
                }
                _ => return None,
            };
            facets.push(facet);
        }

        Some(facets)
    }

    /// Shared code for mapping facets which have the `value` and `fixed` attribute (with default of
    /// fixed being `false`), as well as annotations.
    fn map_value_fixed_annotations<'a, V: ActualValue<'a>>(
        context: &mut MappingContext,
        facet: Node<'a, '_>,
    ) -> (V, bool, Vec<Ref<Annotation>>) {
        // {value} The actual value of the value [attribute]
        let value = actual_value::<V>(facet.attribute("value").unwrap(), facet);

        // {fixed}
        //   The actual value of the fixed [attribute], if present, otherwise false
        let fixed = facet
            .attribute("fixed")
            .map(|v| actual_value::<bool>(v, facet))
            .unwrap_or(false);

        // {annotations} The annotation mapping of the <...> element [...]
        let annotations = Annotation::xml_element_annotation_mapping(context, facet);

        (value, fixed, annotations)
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

    /// Checks whether `self` is of the same kind as `other`. This effectively only compares the
    /// _discriminant_, i.e. ignores the data.
    pub fn is_of_same_kind_as(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

macro_rules! access_methods {
    ($($name:ident => $variant:ident: $typ:ty),*) => {
        $(
            pub fn $name<'a>(&self, components: &'a impl ComponentTable) -> Option<&'a $typ> {
                self.facets.iter().find_map(|f| match f.get(components) {
                    ConstrainingFacet::$variant(c) => Some(c),
                    _ => None,
                })
            }
        )*
    };
}

impl ConstrainingFacets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Ref<ConstrainingFacet>> {
        self.facets.iter()
    }

    pub fn iter_resolved<'a, 'b: 'a>(
        &'a self,
        components: &'b impl ComponentTable,
    ) -> impl Iterator<Item = &'b ConstrainingFacet> + 'a {
        self.facets.iter().map(move |f| f.get(components))
    }

    access_methods! {
        length => Length: Length,
        min_length => MinLength: Length,
        max_length => MaxLength: Length,
        patterns => Pattern: Pattern,
        enumerations => Enumeration: Enumeration,
        white_space => WhiteSpace: WhiteSpace,
        max_inclusive => MaxInclusive: MinMax,
        max_exclusive => MaxExclusive: MinMax,
        min_exclusive => MinExclusive: MinMax,
        min_inclusive => MinInclusive: MinMax,
        total_digits => TotalDigits: TotalDigits,
        fraction_digits => FractionDigits: FractionDigits,
        assertions => Assertions: Assertions,
        explicit_timezone => ExplicitTimezone: ExplicitTimezone
    }
}

impl fmt::Debug for ConstrainingFacets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.facets, f)
    }
}

impl From<Vec<Ref<ConstrainingFacet>>> for ConstrainingFacets {
    fn from(facets: Vec<Ref<ConstrainingFacet>>) -> Self {
        Self { facets }
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

impl ActualValue<'_> for WhiteSpaceValue {
    fn convert(src: &'_ str, _parent: Node) -> Self {
        match src {
            "preserve" => Self::Preserve,
            "replace" => Self::Replace,
            "collapse" => Self::Collapse,
            _ => panic!("Invalid value for whiteSpace value"),
        }
    }
}

impl ActualValue<'_> for ExplicitTimezoneValue {
    fn convert(src: &'_ str, _parent: Node) -> Self {
        match src {
            "required" => Self::Required,
            "prohibited" => Self::Prohibited,
            "optional" => Self::Optional,
            _ => panic!("Invalid value for whiteSpace value"),
        }
    }
}

impl Component for ConstrainingFacet {
    const DISPLAY_NAME: &'static str = "ConstrainingFacet";
}
