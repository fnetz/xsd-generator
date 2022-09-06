use lazy_static::lazy_static;

use super::complex_type_def::{self, ComplexTypeDefinition};
use super::components::Ref;
use super::constraining_facet::{ConstrainingFacet, WhiteSpace, WhiteSpaceValue};
use super::fundamental_facet::{CardinalityValue, FundamentalFacet, OrderedValue};
use super::mapping_context::MappingContext;
use super::simple_type_def::{self, SimpleTypeDefinition};
use super::xstypes::QName;
use super::{attribute_decl, AttributeDeclaration, Sequence, Set, TypeDefinition};

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

}

fn gen_primitive_type_def(
    context: &mut MappingContext,
    base_type: TypeDefinition,
    name: &str,
) -> TypeDefinition {
    // Fundamental facets (from table F.1)
    use CardinalityValue::*;
    use OrderedValue::*;
    let (ordered, bounded, cardinality, numeric) = match name {
        "string" => (False, false, CountablyInfinite, false),
        "boolean" => (False, false, Finite, false),
        "float" => (Partial, true, Finite, true),
        "double" => (Partial, true, Finite, true),
        "decimal" => (Total, false, CountablyInfinite, true),
        "duration" => (Partial, false, CountablyInfinite, false),
        "dateTime" => (Partial, false, CountablyInfinite, false),
        "time" => (Partial, false, CountablyInfinite, false),
        "date" => (Partial, false, CountablyInfinite, false),
        "gYearMonth" => (Partial, false, CountablyInfinite, false),
        "gYear" => (Partial, false, CountablyInfinite, false),
        "gMonthDay" => (Partial, false, CountablyInfinite, false),
        "gDay" => (Partial, false, CountablyInfinite, false),
        "gMonth" => (Partial, false, CountablyInfinite, false),
        "hexBinary" => (False, false, CountablyInfinite, false),
        "base64Binary" => (False, false, CountablyInfinite, false),
        "anyURI" => (False, false, CountablyInfinite, false),
        "QName" => (False, false, CountablyInfinite, false),
        "NOTATION" => (False, false, CountablyInfinite, false),
        _ => unreachable!("Tried to generate primitive type def for non-primitive {name}"),
    };
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
    let ws_facet = context.create(ConstrainingFacet::WhiteSpace(WhiteSpace::new(
        ws_value, ws_fixed,
    )));

    let simple_type_def = context.reserve();
    let type_def = TypeDefinition::Simple(simple_type_def);
    context.insert(
        simple_type_def,
        SimpleTypeDefinition {
            name: Some(name.into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            base_type_definition: base_type,
            final_: Set::new(),
            variety: Some(simple_type_def::Variety::Atomic),
            primitive_type_definition: Some(simple_type_def),
            facets: vec![ws_facet],
            fundamental_facets,
            context: None,
            item_type_definition: None,
            member_type_definitions: None,
            annotations: Sequence::new(),
        },
    );

    type_def
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
            annotations: Sequence::new(), // TODO
        },
    );

    type_def
}

pub(super) fn register_builtins(context: &mut MappingContext) {
    register_builtin_types(context);
    register_builtin_attribute_decls(context);
}

fn register_builtin_types(context: &mut MappingContext) {
    let xs_any_type = context.reserve();
    let xs_any_type_def = TypeDefinition::Complex(xs_any_type);
    context.insert(
        xs_any_type,
        ComplexTypeDefinition {
            name: Some("anyType".into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            base_type_definition: xs_any_type_def,
            derivation_method: Some(complex_type_def::DerivationMethod::Restriction),
            content_type: complex_type_def::ContentType {
                variety: complex_type_def::ContentTypeVariety::Mixed,
                particle: None, // TODO §3.4.7
                open_content: None,
                simple_type_definition: None,
            },
            attribute_uses: Set::new(),
            attribute_wildcard: None, // TODO
            final_: Set::new(),
            context: None,
            prohibited_substitutions: Set::new(),
            assertions: Sequence::new(),
            abstract_: false,
            annotations: Sequence::new(),
        },
    );
    context.register(xs_any_type_def);

    // == Part 2 §4.1.6 Built-in Simple Type Definitions ==

    // anySimpleType
    let xs_any_simple_type = context.reserve();
    let xs_any_simple_type_def = TypeDefinition::Simple(xs_any_simple_type);
    context.insert(
        xs_any_simple_type,
        SimpleTypeDefinition {
            name: Some("anySimpleType".into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            final_: Set::new(),
            context: None,
            base_type_definition: xs_any_type_def,
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

    // anyAtomicType
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

    let xs_error = context.reserve();
    let xs_error_def = TypeDefinition::Simple(xs_error);
    context.insert(
        xs_error,
        SimpleTypeDefinition {
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
        },
    );
    context.register(xs_error_def);

    // primitive data types

    let xs_string = gen_primitive_type_def(context, xs_any_atomic_type_def, "string");
    context.register(xs_string);

    let xs_boolean = gen_primitive_type_def(context, xs_any_atomic_type_def, "boolean");
    context.register(xs_boolean);

    let xs_float = gen_primitive_type_def(context, xs_any_atomic_type_def, "float");
    context.register(xs_float);

    let xs_double = gen_primitive_type_def(context, xs_any_atomic_type_def, "double");
    context.register(xs_double);

    let xs_decimal = gen_primitive_type_def(context, xs_any_atomic_type_def, "decimal");
    context.register(xs_decimal);

    let xs_date_time = gen_primitive_type_def(context, xs_any_atomic_type_def, "dateTime");
    context.register(xs_date_time);

    let xs_duration = gen_primitive_type_def(context, xs_any_atomic_type_def, "duration");
    context.register(xs_duration);

    let xs_time = gen_primitive_type_def(context, xs_any_atomic_type_def, "time");
    context.register(xs_time);

    let xs_date = gen_primitive_type_def(context, xs_any_atomic_type_def, "date");
    context.register(xs_date);

    let xs_g_month = gen_primitive_type_def(context, xs_any_atomic_type_def, "gMonth");
    context.register(xs_g_month);

    let xs_g_month_day = gen_primitive_type_def(context, xs_any_atomic_type_def, "gMonthDay");
    context.register(xs_g_month_day);

    let xs_g_day = gen_primitive_type_def(context, xs_any_atomic_type_def, "gDay");
    context.register(xs_g_day);

    let xs_g_year = gen_primitive_type_def(context, xs_any_atomic_type_def, "gYear");
    context.register(xs_g_year);

    let xs_g_year_month = gen_primitive_type_def(context, xs_any_atomic_type_def, "gYearMonth");
    context.register(xs_g_year_month);

    let xs_hex_binary = gen_primitive_type_def(context, xs_any_atomic_type_def, "hexBinary");
    context.register(xs_hex_binary);

    let xs_base64_binary = gen_primitive_type_def(context, xs_any_atomic_type_def, "base64Binary");
    context.register(xs_base64_binary);

    let xs_any_uri = gen_primitive_type_def(context, xs_any_atomic_type_def, "anyURI");
    context.register(xs_any_uri);

    let xs_qname = gen_primitive_type_def(context, xs_any_atomic_type_def, "QName");
    context.register(xs_qname);

    let xs_notation = gen_primitive_type_def(context, xs_any_atomic_type_def, "NOTATION");
    context.register(xs_notation);

    // ordinary data types

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
