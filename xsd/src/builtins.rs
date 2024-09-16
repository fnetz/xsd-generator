use lazy_static::lazy_static;

use super::complex_type_def::{self, ComplexTypeDefinition};
use super::constraining_facet::{
    ConstrainingFacet, ConstrainingFacets, ExplicitTimezone, ExplicitTimezoneValue, FractionDigits,
    Length, MinMax, Pattern, WhiteSpace, WhiteSpaceValue,
};
use super::fundamental_facet::{
    CardinalityValue, FundamentalFacet, FundamentalFacetSet, OrderedValue,
};
use super::mapping_context::RootContext;
use super::model_group::Compositor;
use super::particle::MaxOccurs;
use super::simple_type_def::{self, SimpleTypeDefinition};
use super::wildcard::{
    DisallowedNameSet, NamespaceConstraint, NamespaceConstraintVariety, ProcessContents,
};
use super::xstypes::QName;
use super::{
    attribute_decl, AttributeDeclaration, ModelGroup, Particle, Sequence, Set, Term,
    TypeDefinition, Wildcard,
};

// Namespaces used by the specification (pt. 1, §1.3.1)
pub const XS_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";
pub const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

lazy_static! {
    /// The `xs:anyType` qualified name
    pub static ref XS_ANY_TYPE_NAME: QName = QName::with_namespace(XS_NAMESPACE, "anyType");
    /// The `xs:anySimpleType` qualified name
    pub static ref XS_ANY_SIMPLE_TYPE_NAME: QName = QName::with_namespace(XS_NAMESPACE, "anySimpleType");
    /// The `xs:anyAtomicType` qualified name
    pub static ref XS_ANY_ATOMIC_TYPE_NAME: QName = QName::with_namespace(XS_NAMESPACE, "anyAtomicType");
    pub static ref XS_QNAME_NAME: QName = QName::with_namespace(XS_NAMESPACE, "QName");
    pub static ref XS_ANY_URI_NAME: QName = QName::with_namespace(XS_NAMESPACE, "anyURI");
    pub static ref XS_BOOLEAN_NAME: QName = QName::with_namespace(XS_NAMESPACE, "boolean");
    pub static ref XS_DECIMAL_NAME: QName = QName::with_namespace(XS_NAMESPACE, "decimal");
    pub static ref XS_STRING_NAME: QName = QName::with_namespace(XS_NAMESPACE, "string");
}

pub(super) fn register_builtins(context: &mut RootContext) {
    register_xs_any_type(context);
    register_special_types(context);
    register_xs_error(context);
    register_builtin_primitive_types(context);
    register_builtin_ordinary_types(context);

    register_builtin_attribute_decls(context);
}

/// Registers the only built-in complex type, `xs:anyType` (§3.4.7 Built-in Complex Type Definition)
fn register_xs_any_type(context: &mut RootContext) {
    // The inner particle of ·xs:anyType· contains a wildcard which matches any element:
    let inner_particle_term = context.create(Wildcard {
        namespace_constraint: NamespaceConstraint {
            variety: NamespaceConstraintVariety::Any,
            namespaces: Set::new(),
            disallowed_names: DisallowedNameSet::default(),
        },
        process_contents: ProcessContents::Lax,
        annotations: Sequence::new(),
    });

    let inner_particle = context.create(Particle {
        min_occurs: 0,
        max_occurs: MaxOccurs::Unbounded,
        term: Term::Wildcard(inner_particle_term),
        annotations: Sequence::new(),
    });

    // The outer particle of ·xs:anyType· contains a sequence with a single term:
    let outer_particle_term = context.create(ModelGroup {
        compositor: Compositor::Sequence,
        particles: vec![inner_particle],
        annotations: Sequence::new(),
    });

    let outer_particle = context.create(Particle {
        min_occurs: 1,
        max_occurs: MaxOccurs::Count(1),
        term: Term::ModelGroup(outer_particle_term),
        annotations: Sequence::new(),
    });

    let wildcard = context.create(Wildcard {
        namespace_constraint: NamespaceConstraint {
            variety: NamespaceConstraintVariety::Any,
            namespaces: Set::new(),
            disallowed_names: DisallowedNameSet::default(),
        },
        process_contents: ProcessContents::Lax,
        annotations: Sequence::new(),
    });

    let xs_any_type = context.reserve();
    context.insert(
        xs_any_type,
        ComplexTypeDefinition {
            name: Some("anyType".into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            base_type_definition: TypeDefinition::Complex(xs_any_type),
            derivation_method: Some(complex_type_def::DerivationMethod::Restriction),
            content_type: complex_type_def::ContentType::Mixed {
                particle: outer_particle,
                open_content: None,
            },
            attribute_uses: Set::new(),
            attribute_wildcard: Some(wildcard),
            final_: Set::new(),
            context: None,
            prohibited_substitutions: Set::new(),
            assertions: Sequence::new(),
            abstract_: false,
            annotations: Sequence::new(),
            is_builtin: true,
        },
    );
    context.register(TypeDefinition::Complex(xs_any_type));
}

/// Registers the special built-in datatypes, `xs:anySimpleType` and ` xs:anyAtomicType`
/// (see Specification pt. 2, §3.2 Special Built-in Datatypes; and pt. 2, §4.1.6 Built-in Simple
/// Type Definitions)
fn register_special_types(context: &mut RootContext) {
    let xs_any_type = context
        .resolve(&XS_ANY_TYPE_NAME)
        .expect("xs:anyType should be registered before xs:anySimpleType and xs:anyAtomicType");

    // anySimpleType (pt. 2, §3.2.1)
    let xs_any_simple_type = context.reserve();
    let xs_any_simple_type_def = TypeDefinition::Simple(xs_any_simple_type);
    context.insert(
        xs_any_simple_type,
        SimpleTypeDefinition {
            name: Some("anySimpleType".into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            final_: Set::new(),
            context: None,
            base_type_definition: TypeDefinition::Complex(xs_any_type),
            facets: ConstrainingFacets::new(),
            fundamental_facets: FundamentalFacetSet::empty(),
            variety: None,
            primitive_type_definition: None,
            item_type_definition: None,
            member_type_definitions: None,
            annotations: Sequence::new(),
            is_builtin: true,
        },
    );
    context.register(xs_any_simple_type_def);

    // anyAtomicType (pt. 2, §3.2.2)
    let xs_any_atomic_type = context.reserve();
    let xs_any_atomic_type_def = TypeDefinition::Simple(xs_any_atomic_type);
    context.insert(
        xs_any_atomic_type,
        SimpleTypeDefinition {
            name: Some("anyAtomicType".into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            final_: Set::new(),
            context: None,
            base_type_definition: xs_any_simple_type_def,
            facets: ConstrainingFacets::new(),
            fundamental_facets: FundamentalFacetSet::empty(),
            variety: Some(simple_type_def::Variety::Atomic),
            primitive_type_definition: None,
            item_type_definition: None,
            member_type_definitions: None,
            annotations: Sequence::new(),
            is_builtin: true,
        },
    );
    context.register(xs_any_atomic_type_def);
}

/// Registers the built-in `xs:error` type (see Specification pt. 1, §3.16.7.3)
fn register_xs_error(context: &mut RootContext) {
    let xs_any_simple_type_def = context
        .resolve(&XS_ANY_SIMPLE_TYPE_NAME)
        .expect("xs:anySimpleType should be registered before xs:error");

    let xs_error = context.create(SimpleTypeDefinition {
        name: Some("error".into()),
        target_namespace: Some(XS_NAMESPACE.into()),
        final_: [
            simple_type_def::DerivationMethod::Extension,
            simple_type_def::DerivationMethod::Restriction,
            simple_type_def::DerivationMethod::List,
            simple_type_def::DerivationMethod::Union,
        ]
        .into_iter()
        .collect(),
        context: None,
        base_type_definition: xs_any_simple_type_def,
        facets: ConstrainingFacets::new(),
        fundamental_facets: FundamentalFacetSet::empty(),
        variety: Some(simple_type_def::Variety::Union),
        primitive_type_definition: None,
        item_type_definition: None,
        member_type_definitions: Some(Sequence::new()),
        annotations: Sequence::new(),
        is_builtin: true,
    });
    context.register(TypeDefinition::Simple(xs_error));
}

struct PrimitiveInfo {
    name: &'static str,
    ordered: OrderedValue,
    bounded: bool,
    cardinality: CardinalityValue,
    numeric: bool,
}

impl PrimitiveInfo {
    const fn new(
        name: &'static str,
        ordered: OrderedValue,
        bounded: bool,
        cardinality: CardinalityValue,
        numeric: bool,
    ) -> Self {
        Self {
            name,
            ordered,
            bounded,
            cardinality,
            numeric,
        }
    }
}

/// Registers the 19 builtin primitive types given by the Specification, according to
/// pt. 1, §3.16.7.4
fn register_builtin_primitive_types(context: &mut RootContext) {
    // The list of primitive type names, along with their fundamental facets (from Table F.1):
    // (name, ordered, bounded, cardinality, numeric)
    use CardinalityValue::*;
    use OrderedValue::*;
    const PRIMITIVE_TYPES: [PrimitiveInfo; 19] = [
        PrimitiveInfo::new("string", False, false, CountablyInfinite, false),
        PrimitiveInfo::new("boolean", False, false, Finite, false),
        PrimitiveInfo::new("float", Partial, true, Finite, true),
        PrimitiveInfo::new("double", Partial, true, Finite, true),
        PrimitiveInfo::new("decimal", Total, false, CountablyInfinite, true),
        PrimitiveInfo::new("dateTime", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("duration", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("time", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("date", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("gMonth", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("gMonthDay", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("gDay", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("gYear", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("gYearMonth", Partial, false, CountablyInfinite, false),
        PrimitiveInfo::new("hexBinary", False, false, CountablyInfinite, false),
        PrimitiveInfo::new("base64Binary", False, false, CountablyInfinite, false),
        PrimitiveInfo::new("anyURI", False, false, CountablyInfinite, false),
        PrimitiveInfo::new("QName", False, false, CountablyInfinite, false),
        PrimitiveInfo::new("NOTATION", False, false, CountablyInfinite, false),
    ];

    let xs_any_atomic_type_def = context
        .resolve(&XS_ANY_ATOMIC_TYPE_NAME)
        .expect("xs:anyAtomicType should be registered before the primitive types");

    for PrimitiveInfo {
        name,
        ordered,
        bounded,
        cardinality,
        numeric,
    } in PRIMITIVE_TYPES
    {
        let fundamental_facets = [
            FundamentalFacet::Ordered(ordered),
            FundamentalFacet::Bounded(bounded),
            FundamentalFacet::Cardinality(cardinality),
            FundamentalFacet::Numeric(numeric),
        ]
        .into_iter()
        .collect();
        let fundamental_facets = FundamentalFacetSet::new(fundamental_facets);

        // Constraining facets (whiteSpace)
        let (ws_value, ws_fixed) = match name {
            "string" => (WhiteSpaceValue::Preserve, false),
            _ => (WhiteSpaceValue::Collapse, true),
        };
        let whitespace = context.create(ConstrainingFacet::WhiteSpace(WhiteSpace::new(
            ws_value, ws_fixed,
        )));

        let simple_type_def = context.reserve();
        context.insert(
            simple_type_def,
            SimpleTypeDefinition {
                name: Some(name.into()),
                target_namespace: Some(XS_NAMESPACE.into()),
                base_type_definition: xs_any_atomic_type_def,
                final_: Set::new(),
                variety: Some(simple_type_def::Variety::Atomic),
                primitive_type_definition: Some(simple_type_def),
                facets: ConstrainingFacets::from(vec![whitespace]),
                fundamental_facets,
                context: None,
                item_type_definition: None,
                member_type_definitions: None,
                annotations: Sequence::new(),
                is_builtin: true,
            },
        );
        context.register(simple_type_def);
    }
}

enum OrdinaryVariety {
    Atomic,
    /// List with given item type definition name
    List(&'static str),
}

#[derive(Copy, Clone)]
enum FixedValue {
    NotFixed,
    Fixed,
}

#[derive(Copy, Clone)]
enum OrdinaryFacet {
    ExplicitTimezone(ExplicitTimezoneValue, FixedValue),
    FractionDigits(u64, FixedValue),
    /// Maximum value, represented as string
    MaxInclusive(&'static str),
    MinInclusive(&'static str),
    MinLength(u64),
    Pattern(&'static [&'static str]),
    WhiteSpace(WhiteSpaceValue, FixedValue),
}

impl FixedValue {
    fn as_bool(self) -> bool {
        match self {
            FixedValue::NotFixed => false,
            FixedValue::Fixed => true,
        }
    }
}

impl OrdinaryFacet {
    fn to_constraining_facet(self) -> ConstrainingFacet {
        match self {
            OrdinaryFacet::ExplicitTimezone(value, fixed) => {
                ConstrainingFacet::ExplicitTimezone(ExplicitTimezone {
                    value,
                    fixed: fixed.as_bool(),
                    annotations: Sequence::new(),
                })
            }
            OrdinaryFacet::FractionDigits(value, fixed) => {
                ConstrainingFacet::FractionDigits(FractionDigits {
                    value,
                    fixed: fixed.as_bool(),
                    annotations: Sequence::new(),
                })
            }
            OrdinaryFacet::MaxInclusive(value) => ConstrainingFacet::MaxInclusive(MinMax {
                value: value.into(),
                fixed: false,
                annotations: Sequence::new(),
            }),
            OrdinaryFacet::MinInclusive(value) => ConstrainingFacet::MinInclusive(MinMax {
                value: value.into(),
                fixed: false,
                annotations: Sequence::new(),
            }),
            OrdinaryFacet::MinLength(value) => ConstrainingFacet::MinLength(Length {
                value,
                fixed: false,
                annotations: Sequence::new(),
            }),
            OrdinaryFacet::Pattern(value) => ConstrainingFacet::Pattern(Pattern {
                value: value.iter().map(|&f| f.to_string()).collect(),
                annotations: Sequence::new(),
            }),
            OrdinaryFacet::WhiteSpace(value, fixed) => ConstrainingFacet::WhiteSpace(WhiteSpace {
                value,
                fixed: fixed.as_bool(),
                annotations: Sequence::new(),
            }),
        }
    }
}

enum OrdinaryBaseType {
    Named(&'static str),
    AnonIndirectList(&'static str),
}

struct OrdinaryInfo {
    name: &'static str,
    base_type: OrdinaryBaseType,
    variety: OrdinaryVariety,
    ordered: OrderedValue,
    bounded: bool,
    cardinality: CardinalityValue,
    numeric: bool,
    facets: &'static [OrdinaryFacet],
    // TODO annotations
}

fn register_builtin_ordinary_types(context: &mut RootContext) {
    use CardinalityValue::*;
    use FixedValue::*;
    use OrderedValue::*;
    const ORDINARY_TYPES: [OrdinaryInfo; 28] = [
        OrdinaryInfo {
            name: "normalizedString",
            base_type: OrdinaryBaseType::Named("string"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[OrdinaryFacet::WhiteSpace(
                WhiteSpaceValue::Replace,
                NotFixed,
            )],
        },
        OrdinaryInfo {
            name: "token",
            base_type: OrdinaryBaseType::Named("normalizedString"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[OrdinaryFacet::WhiteSpace(
                WhiteSpaceValue::Collapse,
                NotFixed,
            )],
        },
        OrdinaryInfo {
            name: "language",
            base_type: OrdinaryBaseType::Named("token"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&["[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "NMTOKEN",
            base_type: OrdinaryBaseType::Named("token"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&[r"\c+"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "NMTOKENS",
            base_type: OrdinaryBaseType::AnonIndirectList("NMTOKEN"),
            variety: OrdinaryVariety::List("NMTOKEN"),
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::MinLength(1),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "Name",
            base_type: OrdinaryBaseType::Named("token"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&[r"\i\c*"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "NCName",
            base_type: OrdinaryBaseType::Named("Name"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&[r"\i\c*", r"[\i-[:]][\c-[:]]*"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "ID",
            base_type: OrdinaryBaseType::Named("NCName"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&[r"\i\c*", r"[\i-[:]][\c-[:]]*"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "IDREF",
            base_type: OrdinaryBaseType::Named("NCName"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&[r"\i\c*", r"[\i-[:]][\c-[:]]*"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "IDREFS",
            base_type: OrdinaryBaseType::AnonIndirectList("IDREF"),
            variety: OrdinaryVariety::List("IDREF"),
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::MinLength(1),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "ENTITY",
            base_type: OrdinaryBaseType::Named("NCName"),
            variety: OrdinaryVariety::Atomic,
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::Pattern(&[r"\i\c*", r"[\i-[:]][\c-[:]]*"]),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "ENTITIES",
            base_type: OrdinaryBaseType::AnonIndirectList("ENTITY"),
            variety: OrdinaryVariety::List("ENTITY"),
            ordered: False,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::MinLength(1),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, NotFixed),
            ],
        },
        OrdinaryInfo {
            name: "integer",
            base_type: OrdinaryBaseType::Named("decimal"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
            ],
        },
        OrdinaryInfo {
            name: "nonPositiveInteger",
            base_type: OrdinaryBaseType::Named("integer"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("0"),
            ],
        },
        OrdinaryInfo {
            name: "negativeInteger",
            base_type: OrdinaryBaseType::Named("nonPositiveInteger"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]), // TODO pattern seems wrong (spec issue?)
                OrdinaryFacet::MaxInclusive("-1"),
            ],
        },
        OrdinaryInfo {
            name: "long",
            base_type: OrdinaryBaseType::Named("integer"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("9223372036854775807"),
                OrdinaryFacet::MinInclusive("-9223372036854775808"),
            ],
        },
        OrdinaryInfo {
            name: "int",
            base_type: OrdinaryBaseType::Named("long"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("2147483647"),
                OrdinaryFacet::MinInclusive("-2147483648"),
            ],
        },
        OrdinaryInfo {
            name: "short",
            base_type: OrdinaryBaseType::Named("int"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("32767"),
                OrdinaryFacet::MinInclusive("-32768"),
            ],
        },
        OrdinaryInfo {
            name: "byte",
            base_type: OrdinaryBaseType::Named("short"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("127"),
                OrdinaryFacet::MinInclusive("-128"),
            ],
        },
        OrdinaryInfo {
            name: "nonNegativeInteger",
            base_type: OrdinaryBaseType::Named("integer"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MinInclusive("0"),
            ],
        },
        OrdinaryInfo {
            name: "unsignedLong",
            base_type: OrdinaryBaseType::Named("nonNegativeInteger"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("18446744073709551615"),
                OrdinaryFacet::MinInclusive("0"),
            ],
        },
        OrdinaryInfo {
            name: "unsignedInt",
            base_type: OrdinaryBaseType::Named("unsignedLong"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("4294967295"),
                OrdinaryFacet::MinInclusive("0"),
            ],
        },
        OrdinaryInfo {
            name: "unsignedShort",
            base_type: OrdinaryBaseType::Named("unsignedInt"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("65535"),
                OrdinaryFacet::MinInclusive("0"),
            ],
        },
        OrdinaryInfo {
            name: "unsignedByte",
            base_type: OrdinaryBaseType::Named("unsignedShort"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: true,
            cardinality: Finite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MaxInclusive("255"),
                OrdinaryFacet::MinInclusive("0"),
            ],
        },
        OrdinaryInfo {
            name: "positiveInteger",
            base_type: OrdinaryBaseType::Named("nonNegativeInteger"),
            variety: OrdinaryVariety::Atomic,
            ordered: Total,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: true,
            facets: &[
                OrdinaryFacet::FractionDigits(0, Fixed),
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&[r"[\-+]?[0-9]+"]),
                OrdinaryFacet::MinInclusive("1"),
            ],
        },
        OrdinaryInfo {
            name: "yearMonthDuration",
            base_type: OrdinaryBaseType::Named("duration"),
            variety: OrdinaryVariety::Atomic,
            ordered: Partial,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&["[^DT]*"]),
            ],
        },
        OrdinaryInfo {
            name: "dayTimeDuration",
            base_type: OrdinaryBaseType::Named("duration"),
            variety: OrdinaryVariety::Atomic,
            ordered: Partial,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::Pattern(&["[^YM]*(T.*)?"]),
            ],
        },
        OrdinaryInfo {
            name: "dateTimeStamp",
            base_type: OrdinaryBaseType::Named("dateTime"),
            variety: OrdinaryVariety::Atomic,
            ordered: Partial,
            bounded: false,
            cardinality: CountablyInfinite,
            numeric: false,
            facets: &[
                OrdinaryFacet::WhiteSpace(WhiteSpaceValue::Collapse, Fixed),
                OrdinaryFacet::ExplicitTimezone(ExplicitTimezoneValue::Required, Fixed),
            ],
        },
    ];

    for OrdinaryInfo {
        name,
        base_type,
        variety,
        ordered,
        bounded,
        cardinality,
        numeric,
        facets,
    } in ORDINARY_TYPES
    {
        let simple_type_ref = context.reserve();

        let base_type = match base_type {
            OrdinaryBaseType::Named(base_type_name) => {
                let base_type_name = QName::with_namespace(XS_NAMESPACE, base_type_name);
                context.resolve(&base_type_name).unwrap_or_else(|| {
                    panic!("Base type {:?} for {} not found", base_type_name, name)
                })
            }
            OrdinaryBaseType::AnonIndirectList(item_type_name) => {
                let item_type_name = QName::with_namespace(XS_NAMESPACE, item_type_name);
                let item_type = context.resolve(&item_type_name).unwrap_or_else(|| {
                    panic!("Item type {:?} for {} not found", item_type_name, name)
                });

                let xs_any_simple_type = context
                    .resolve(&XS_ANY_SIMPLE_TYPE_NAME)
                    .expect("xs:anySimpleType should be registered before the ordinary types");
                context.create(SimpleTypeDefinition {
                    annotations: Sequence::new(),
                    name: None,
                    target_namespace: Some(XS_NAMESPACE.into()),
                    final_: Set::new(),
                    context: Some(simple_type_def::Context::SimpleType(simple_type_ref)),
                    base_type_definition: xs_any_simple_type,
                    facets: ConstrainingFacets::new(),
                    fundamental_facets: FundamentalFacetSet::empty(),
                    variety: Some(simple_type_def::Variety::List),
                    primitive_type_definition: None,
                    item_type_definition: Some(item_type),
                    member_type_definitions: None,
                    is_builtin: true,
                })
            }
        };

        let fundamental_facets = [
            FundamentalFacet::Ordered(ordered),
            FundamentalFacet::Bounded(bounded),
            FundamentalFacet::Cardinality(cardinality),
            FundamentalFacet::Numeric(numeric),
        ]
        .into_iter()
        .collect();
        let fundamental_facets = FundamentalFacetSet::new(fundamental_facets);

        let (variety, item_type) = match variety {
            OrdinaryVariety::Atomic => (simple_type_def::Variety::Atomic, None),
            OrdinaryVariety::List(item_type_name) => {
                let item_type_name = QName::with_namespace(XS_NAMESPACE, item_type_name);
                (
                    simple_type_def::Variety::List,
                    Some(
                        context
                            .resolve(&item_type_name)
                            .expect("Item type not found"),
                    ),
                )
            }
        };

        let facets = ConstrainingFacets::from(
            facets
                .iter()
                .map(|f| context.create(f.to_constraining_facet()))
                .collect::<Vec<_>>(),
        );

        context.insert(
            simple_type_ref,
            SimpleTypeDefinition {
                name: Some(name.into()),
                target_namespace: Some(XS_NAMESPACE.into()),
                base_type_definition: TypeDefinition::Simple(base_type),
                final_: Set::new(),
                variety: Some(variety),
                primitive_type_definition: match variety {
                    simple_type_def::Variety::Atomic => {
                        base_type
                            .get(context.components())
                            .primitive_type_definition
                    }
                    _ => None,
                },
                facets,
                fundamental_facets,
                context: None,
                item_type_definition: match variety {
                    simple_type_def::Variety::Atomic => None,
                    _ => Some(item_type.unwrap()),
                },
                member_type_definitions: None,
                annotations: Sequence::new(),
                is_builtin: true,
            },
        );
        context.register(simple_type_ref);
    }
}

fn register_builtin_attribute_decls(context: &mut RootContext) {
    let qname = context
        .resolve(&XS_QNAME_NAME)
        .expect("xs:QName should be registered");
    let boolean = context
        .resolve(&XS_BOOLEAN_NAME)
        .expect("xs:boolean should be registered");
    let any_uri = context
        .resolve(&XS_ANY_URI_NAME)
        .expect("xs:anyURI should be registered");
    let any_simple_type = context
        .resolve(&XS_ANY_SIMPLE_TYPE_NAME)
        .expect("xs:anySimpleType should be registered");

    // Built-in Attribute Declarations according to pt. 1, §3.2.7
    // The {inheritable} property is not specified by the 1.1 spec;
    // assuming `false` for now.

    let xsi_type = context.create(AttributeDeclaration {
        name: "type".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: qname,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
        is_builtin: true,
    });
    context.register(xsi_type);

    let xsi_nil = context.create(AttributeDeclaration {
        name: "nil".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: boolean,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
        is_builtin: true,
    });
    context.register(xsi_nil);

    let xsi_schema_location = context.reserve();
    let schema_location_simple_type = context.create(SimpleTypeDefinition {
        name: None,
        target_namespace: Some(XSI_NAMESPACE.into()),
        base_type_definition: any_simple_type,
        facets: ConstrainingFacets::new(), // TODO spec says absent?
        variety: Some(simple_type_def::Variety::List),
        item_type_definition: Some(any_uri),
        annotations: Sequence::new(),
        final_: Set::new(),
        context: Some(simple_type_def::Context::Attribute(xsi_schema_location)),
        fundamental_facets: FundamentalFacetSet::empty(),
        primitive_type_definition: None,
        member_type_definitions: None,
        is_builtin: true,
    });
    context.insert(
        xsi_schema_location,
        AttributeDeclaration {
            name: "schemaLocation".into(),
            target_namespace: Some(XSI_NAMESPACE.into()),
            type_definition: schema_location_simple_type,
            scope: attribute_decl::Scope::new_global(),
            value_constraint: None,
            annotations: Sequence::new(),
            inheritable: false,
            is_builtin: true,
        },
    );
    context.register(xsi_schema_location);

    let xsi_no_namespace_schema_location = context.create(AttributeDeclaration {
        name: "noNamespaceSchemaLocation".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: any_uri,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
        is_builtin: true,
    });
    context.register(xsi_no_namespace_schema_location);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuiltinOverwriteAction;

    #[test]
    fn registers_builtins_without_crashing() {
        let mut root_context = RootContext::new(BuiltinOverwriteAction::Deny, &[]);

        register_builtins(&mut root_context);
    }
}
