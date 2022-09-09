use lazy_static::lazy_static;

use super::complex_type_def::{self, ComplexTypeDefinition};
use super::components::Ref;
use super::constraining_facet::{ConstrainingFacet, WhiteSpace, WhiteSpaceValue};
use super::fundamental_facet::{CardinalityValue, FundamentalFacet, OrderedValue};
use super::mapping_context::MappingContext;
use super::model_group::Compositor;
use super::particle::MaxOccurs;
use super::simple_type_def::{self, SimpleTypeDefinition};
use super::wildcard::{NamespaceConstraint, NamespaceConstraintVariety, ProcessContents};
use super::xstypes::QName;
use super::{
    attribute_decl, AttributeDeclaration, ModelGroup, Particle, Sequence, Set, Term,
    TypeDefinition, Wildcard,
};

// Namespaces used by the specification (pt. 1, §1.3.1)
pub const XS_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";
pub const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";
pub const VC_NAMESPACE: &str = " http://www.w3.org/2007/XMLSchema-versioning";

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

fn gen_ordinary_type_def(
    context: &mut MappingContext,
    base_type: TypeDefinition,
    name: &str,
    variety: simple_type_def::Variety,
    facets: Set<Ref<ConstrainingFacet>>,
    fundamental_facets: Set<FundamentalFacet>,
    item_type_def: Option<Ref<SimpleTypeDefinition>>,
) -> TypeDefinition {
    let simple_type_def = context.reserve();
    let type_def = TypeDefinition::Simple(simple_type_def);
    context.insert(
        simple_type_def,
        SimpleTypeDefinition {
            name: Some(name.into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            base_type_definition: base_type,
            final_: Set::new(),
            variety: Some(variety),
            primitive_type_definition: match variety {
                simple_type_def::Variety::Atomic => match base_type {
                    TypeDefinition::Simple(ref_) => {
                        ref_.get(context.components()).primitive_type_definition
                    }
                    TypeDefinition::Complex(_) => unimplemented!(),
                },
                _ => None,
            },
            facets,
            fundamental_facets,
            context: None,
            item_type_definition: match variety {
                simple_type_def::Variety::Atomic => None,
                _ => Some(item_type_def.unwrap()),
            },
            member_type_definitions: None,
            annotations: Sequence::new(),
        },
    );

    type_def
}

pub(super) fn register_builtins(context: &mut MappingContext) {
    register_xs_any_type(context);
    register_special_types(context);
    register_xs_error(context);
    register_builtin_primitive_types(context);
    register_builtin_ordinary_types(context);

    register_builtin_attribute_decls(context);
}

/// Registers the only built-in complex type, `xs:anyType` (§3.4.7 Built-in Complex Type Definition)
fn register_xs_any_type(context: &mut MappingContext) {
    // The inner particle of ·xs:anyType· contains a wildcard which matches any element:
    let inner_particle_term = context.create(Wildcard {
        namespace_constraint: NamespaceConstraint {
            variety: NamespaceConstraintVariety::Any,
            namespaces: Set::new(),
            disallowed_names: Set::new(),
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
            disallowed_names: Set::new(),
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
            content_type: complex_type_def::ContentType {
                variety: complex_type_def::ContentTypeVariety::Mixed,
                particle: Some(outer_particle),
                simple_type_definition: None,
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
        },
    );
    context.register(TypeDefinition::Complex(xs_any_type));
}

/// Registers the special built-in datatypes, `xs:anySimpleType` and ` xs:anyAtomicType`
/// (see Specification pt. 2, §3.2 Special Built-in Datatypes; and pt. 2, §4.1.6 Built-in Simple
/// Type Definitions)
fn register_special_types(context: &mut MappingContext) {
    let xs_any_type = context.resolve(&XS_ANY_TYPE_NAME);

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
            facets: Set::new(),
            fundamental_facets: Set::new(),
            variety: None,
            primitive_type_definition: None,
            item_type_definition: None,
            member_type_definitions: None,
            annotations: Sequence::new(),
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
            facets: Set::new(),
            fundamental_facets: Set::new(),
            variety: Some(simple_type_def::Variety::Atomic),
            primitive_type_definition: None,
            item_type_definition: None,
            member_type_definitions: None,
            annotations: Sequence::new(),
        },
    );
    context.register(xs_any_atomic_type_def);
}

/// Registers the built-in `xs:error` type (see Specification pt. 1, §3.16.7.3)
fn register_xs_error(context: &mut MappingContext) {
    let xs_any_simple_type_def = context.resolve(&XS_ANY_SIMPLE_TYPE_NAME);

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
        facets: Set::new(),
        fundamental_facets: Set::new(),
        variety: Some(simple_type_def::Variety::Union),
        primitive_type_definition: None,
        item_type_definition: None,
        member_type_definitions: Some(Sequence::new()),
        annotations: Sequence::new(),
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
fn register_builtin_primitive_types(context: &mut MappingContext) {
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

    let xs_any_atomic_type_def = context.resolve(&XS_ANY_ATOMIC_TYPE_NAME);

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
                facets: vec![whitespace],
                fundamental_facets,
                context: None,
                item_type_definition: None,
                member_type_definitions: None,
                annotations: Sequence::new(),
            },
        );
        context.register(simple_type_def);
    }
}

fn register_builtin_ordinary_types(context: &mut MappingContext) {
    let xs_decimal = context.resolve(&XS_DECIMAL_NAME);
    let xs_string = context.resolve(&XS_STRING_NAME);

    let xs_integer = gen_ordinary_type_def(
        context,
        xs_decimal,
        "integer",
        simple_type_def::Variety::Atomic,
        Set::new(), // TODO P2 §3.4.13.3
        Set::new(), // TODO
        None,
    );
    context.register(xs_integer);

    let xs_non_negative_integer = gen_ordinary_type_def(
        context,
        xs_integer,
        "nonNegativeInteger",
        simple_type_def::Variety::Atomic,
        Set::new(), // TODO P2 §3.4.20.3
        Set::new(), // TODO
        None,
    );
    context.register(xs_non_negative_integer);

    let xs_positive_integer = gen_ordinary_type_def(
        context,
        xs_non_negative_integer,
        "positiveInteger",
        simple_type_def::Variety::Atomic,
        Set::new(), // TODO P2 §3.4.25.3
        Set::new(), // TODO
        None,
    );
    context.register(xs_positive_integer);

    let xs_normalized_string = gen_ordinary_type_def(
        context,
        xs_string,
        "normalizedString",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_normalized_string);

    let xs_token = gen_ordinary_type_def(
        context,
        xs_normalized_string,
        "token",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_token);

    let xs_nmtoken = gen_ordinary_type_def(
        context,
        xs_token,
        "NMTOKEN",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_nmtoken);

    let xs_name = gen_ordinary_type_def(
        context,
        xs_token,
        "Name",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_name);

    let xs_ncname = gen_ordinary_type_def(
        context,
        xs_name,
        "NCName",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_ncname);

    let xs_id = gen_ordinary_type_def(
        context,
        xs_ncname,
        "ID",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_id);

    let xs_language = gen_ordinary_type_def(
        context,
        xs_token,
        "language",
        simple_type_def::Variety::Atomic, // TODO
        Set::new(),                       // TODO
        Set::new(),                       // TODO
        None,
    );
    context.register(xs_language);
}

fn register_builtin_attribute_decls(context: &mut MappingContext) {
    let qname = context.resolve(&XS_QNAME_NAME);
    let boolean = context.resolve(&XS_BOOLEAN_NAME);
    let any_uri = context.resolve(&XS_ANY_URI_NAME);
    let any_simple_type = context.resolve(&XS_ANY_SIMPLE_TYPE_NAME);

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
    });
    context.register(xsi_nil);

    let schema_location_simple_type = context.create(SimpleTypeDefinition {
        name: None,
        target_namespace: Some(XSI_NAMESPACE.into()),
        base_type_definition: any_simple_type,
        facets: Set::new(), // TODO spec says absent?
        variety: Some(simple_type_def::Variety::List),
        item_type_definition: Some(any_uri),
        annotations: Sequence::new(),

        final_: Set::new(),
        context: None,
        fundamental_facets: Set::new(), // TODO
        primitive_type_definition: None,
        member_type_definitions: None,
    });
    let xsi_schema_location = context.create(AttributeDeclaration {
        name: "schemaLocation".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: schema_location_simple_type,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
    });
    context.register(xsi_schema_location);

    let xsi_no_namespace_schema_location = context.create(AttributeDeclaration {
        name: "noNamespaceSchemaLocation".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: any_uri,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
    });
    context.register(xsi_no_namespace_schema_location);
}

pub fn is_builtin_name(name: &QName) -> bool {
    is_builtin_type_name(name) || is_builtin_attribute_decl_name(name)
}

pub fn is_builtin_type_name(name: &QName) -> bool {
    if name.namespace_name.as_deref() != Some(XS_NAMESPACE) {
        return false;
    }

    const BUILTIN_TYPE_NAMES: [&str; 33] = [
        "anyType",
        "anySimpleType",
        "anyAtomicType",
        "error",
        "string",
        "boolean",
        "float",
        "double",
        "decimal",
        "dateTime",
        "duration",
        "time",
        "date",
        "gMonth",
        "gMonthDay",
        "gDay",
        "gYear",
        "gYearMonth",
        "hexBinary",
        "base64Binary",
        "anyURI",
        "QName",
        "NOTATION",
        "integer",
        "nonNegativeInteger",
        "positiveInteger",
        "normalizedString",
        "token",
        "NMTOKEN",
        "Name",
        "NCName",
        "ID",
        "language",
    ];
    BUILTIN_TYPE_NAMES.contains(&name.local_name.as_str())
}

pub fn is_builtin_attribute_decl_name(name: &QName) -> bool {
    if name.namespace_name.as_deref() != Some(XSI_NAMESPACE) {
        return false;
    }

    const BUILTIN_ATTRIBUTE_NAMES: [&str; 4] =
        ["type", "nil", "schemaLocation", "noNamespaceSchemaLocation"];

    BUILTIN_ATTRIBUTE_NAMES.contains(&name.local_name.as_str())
}
