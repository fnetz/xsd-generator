use super::complex_type_def::{self, ComplexTypeDefinition};
use super::components::{IntermediateComponentContainer, Ref, Resolution};
use super::constraining_facet::{ConstrainingFacet, WhiteSpace, WhiteSpaceValue};
use super::fundamental_facet::{CardinalityValue, FundamentalFacet, OrderedValue};
use super::simple_type_def::{self, SimpleTypeDefinition};
use super::xstypes::QName;
use super::{attribute_decl, AttributeDeclaration, Sequence, Set, TypeDefinition};

// Namespaces used by the specification (pt. 1, §1.3.1)
pub const XS_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";
pub const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";
pub const VC_NAMESPACE: &str = " http://www.w3.org/2007/XMLSchema-versioning";

fn gen_primitive_type_def(
    components: &mut IntermediateComponentContainer,
    base_type: Ref<TypeDefinition>,
    name: &str,
) -> Ref<TypeDefinition> {
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
    let ws_facet = components.create(ConstrainingFacet::WhiteSpace(WhiteSpace::new(
        ws_value, ws_fixed,
    )));

    let simple_type_def = components.reserve();
    components.populate(
        simple_type_def,
        SimpleTypeDefinition {
            name: Some(name.into()),
            target_namespace: Some(XS_NAMESPACE.into()),
            base_type_definition: Some(base_type),
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

    components.create(TypeDefinition::Simple(simple_type_def))
}

fn gen_ordinary_type_def(
    components: &mut IntermediateComponentContainer,
    base_type: Ref<TypeDefinition>,
    name: &str,
    variety: simple_type_def::Variety,
    facets: Set<Ref<ConstrainingFacet>>,
    fundamental_facets: Set<FundamentalFacet>,
    item_type_def: Option<Ref<SimpleTypeDefinition>>,
) -> Ref<TypeDefinition> {
    let simple_type_def = components.create(SimpleTypeDefinition {
        name: Some(name.into()),
        target_namespace: Some(XS_NAMESPACE.into()),
        base_type_definition: Some(base_type),
        final_: Set::new(),
        variety: Some(variety),
        primitive_type_definition: match variety {
            simple_type_def::Variety::Atomic => {
                match base_type.get_intermediate(components).unwrap() {
                    TypeDefinition::Simple(ref_) => {
                        ref_.get_intermediate(components)
                            .unwrap()
                            .primitive_type_definition
                    }
                    TypeDefinition::Complex(_) => unimplemented!(),
                }
            }
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
    });

    components.create(TypeDefinition::Simple(simple_type_def))
}

pub fn register_builtins(components: &mut IntermediateComponentContainer) {
    register_builtin_types(components);
    register_builtin_attribute_decls(components);
}

fn register_builtin_types(components: &mut IntermediateComponentContainer) {
    let xs_any_type = components.create(ComplexTypeDefinition {
        annotations: Sequence::new(),
        name: Some("anyType".into()),
        target_namespace: Some(XS_NAMESPACE.into()),
        base_type_definition: None,
        final_: Set::new(),
        context: None, // TODO
        derivation_method: None,
        abstract_: false,
        attribute_uses: Set::new(),
        attribute_wildcard: None,
        content_type: complex_type_def::ContentType {
            variety: complex_type_def::ContentTypeVariety::Empty,
            particle: None,
            open_content: None,
            simple_type_definition: None,
        },
        prohibited_substitutions: Set::new(),
        assertions: Sequence::new(),
    });
    let xs_any_type_def = components.create(TypeDefinition::Complex(xs_any_type));
    components.register_type(xs_any_type_def);

    // == Part 2 §4.1.6 Built-in Simple Type Definitions ==

    // anySimpleType
    let xs_any_simple_type = components.create(SimpleTypeDefinition {
        name: Some("anySimpleType".into()),
        target_namespace: Some(XS_NAMESPACE.into()),
        final_: Set::new(),
        context: None,
        base_type_definition: Some(xs_any_type_def),
        facets: Set::new(),
        fundamental_facets: Set::new(),
        variety: None,
        primitive_type_definition: None,
        item_type_definition: None,
        member_type_definitions: None,
        annotations: Sequence::new(),
    });
    let xs_any_simple_type_def = components.create(TypeDefinition::Simple(xs_any_simple_type));
    components.register_type(xs_any_simple_type_def);

    // anyAtomicType
    let xs_any_atomic_type = components.create(SimpleTypeDefinition {
        name: Some("anyAtomicType".into()),
        target_namespace: Some(XS_NAMESPACE.into()),
        final_: Set::new(),
        context: None,
        base_type_definition: Some(xs_any_simple_type_def),
        facets: Set::new(),
        fundamental_facets: Set::new(),
        variety: Some(simple_type_def::Variety::Atomic),
        primitive_type_definition: None,
        item_type_definition: None,
        member_type_definitions: None,
        annotations: Sequence::new(),
    });
    let xs_any_atomic_type_def = components.create(TypeDefinition::Simple(xs_any_atomic_type));
    components.register_type(xs_any_atomic_type_def);

    // primitive data types

    let xs_string = gen_primitive_type_def(components, xs_any_atomic_type_def, "string");
    components.register_type(xs_string);

    let xs_boolean = gen_primitive_type_def(components, xs_any_atomic_type_def, "boolean");
    components.register_type(xs_boolean);

    let xs_float = gen_primitive_type_def(components, xs_any_atomic_type_def, "float");
    components.register_type(xs_float);

    let xs_double = gen_primitive_type_def(components, xs_any_atomic_type_def, "double");
    components.register_type(xs_double);

    let xs_decimal = gen_primitive_type_def(components, xs_any_atomic_type_def, "decimal");
    components.register_type(xs_decimal);

    let xs_date_time = gen_primitive_type_def(components, xs_any_atomic_type_def, "dateTime");
    components.register_type(xs_date_time);

    let xs_duration = gen_primitive_type_def(components, xs_any_atomic_type_def, "duration");
    components.register_type(xs_duration);

    let xs_time = gen_primitive_type_def(components, xs_any_atomic_type_def, "time");
    components.register_type(xs_time);

    let xs_date = gen_primitive_type_def(components, xs_any_atomic_type_def, "date");
    components.register_type(xs_date);

    let xs_g_month = gen_primitive_type_def(components, xs_any_atomic_type_def, "gMonth");
    components.register_type(xs_g_month);

    let xs_g_month_day = gen_primitive_type_def(components, xs_any_atomic_type_def, "gMonthDay");
    components.register_type(xs_g_month_day);

    let xs_g_day = gen_primitive_type_def(components, xs_any_atomic_type_def, "gDay");
    components.register_type(xs_g_day);

    let xs_g_year = gen_primitive_type_def(components, xs_any_atomic_type_def, "gYear");
    components.register_type(xs_g_year);

    let xs_g_year_month = gen_primitive_type_def(components, xs_any_atomic_type_def, "gYearMonth");
    components.register_type(xs_g_year_month);

    let xs_hex_binary = gen_primitive_type_def(components, xs_any_atomic_type_def, "hexBinary");
    components.register_type(xs_hex_binary);

    let xs_base64_binary =
        gen_primitive_type_def(components, xs_any_atomic_type_def, "base64Binary");
    components.register_type(xs_base64_binary);

    let xs_any_uri = gen_primitive_type_def(components, xs_any_atomic_type_def, "anyURI");
    components.register_type(xs_any_uri);

    let xs_qname = gen_primitive_type_def(components, xs_any_atomic_type_def, "QName");
    components.register_type(xs_qname);

    let xs_notation = gen_primitive_type_def(components, xs_any_atomic_type_def, "NOTATION");
    components.register_type(xs_notation);

    // ordinary data types

    let xs_integer = gen_ordinary_type_def(
        components,
        xs_decimal,
        "integer",
        simple_type_def::Variety::Atomic,
        Set::new(), // TODO P2 §3.4.13.3
        Set::new(), // TODO
        None,
    );
    components.register_type(xs_integer);

    let xs_non_negative_integer = gen_ordinary_type_def(
        components,
        xs_integer,
        "nonNegativeInteger",
        simple_type_def::Variety::Atomic,
        Set::new(), // TODO P2 §3.4.20.3
        Set::new(), // TODO
        None,
    );
    components.register_type(xs_non_negative_integer);

    let xs_positive_integer = gen_ordinary_type_def(
        components,
        xs_non_negative_integer,
        "positiveInteger",
        simple_type_def::Variety::Atomic,
        Set::new(), // TODO P2 §3.4.25.3
        Set::new(), // TODO
        None,
    );
    components.register_type(xs_positive_integer);
}

fn register_builtin_attribute_decls(components: &mut IntermediateComponentContainer) {
    let qname = components.resolve_simple_type_def(
        &QName(XS_NAMESPACE.into(), "QName".into()),
        Resolution::Immediate,
    );
    let boolean = components.resolve_simple_type_def(
        &QName(XS_NAMESPACE.into(), "boolean".into()),
        Resolution::Immediate,
    );
    let any_uri = components.resolve_simple_type_def(
        &QName(XS_NAMESPACE.into(), "anyURI".into()),
        Resolution::Immediate,
    );
    let any_simple_type = components.resolve_type_def(
        &QName(XS_NAMESPACE.into(), "anySimpleType".into()),
        Resolution::Immediate,
    );

    // Built-in Attribute Declarations according to pt. 1, §3.2.7
    // The {inheritable} property is not specified by the 1.1 spec;
    // assuming `false` for now.

    let xsi_type = components.create(AttributeDeclaration {
        name: "type".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: qname,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
    });
    components.register_attribute_decl(xsi_type);

    let xsi_nil = components.create(AttributeDeclaration {
        name: "nil".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: boolean,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
    });
    components.register_attribute_decl(xsi_nil);

    let schema_location_simple_type = components.create(SimpleTypeDefinition {
        name: None,
        target_namespace: Some(XSI_NAMESPACE.into()),
        base_type_definition: Some(any_simple_type),
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
    let xsi_schema_location = components.create(AttributeDeclaration {
        name: "schemaLocation".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: schema_location_simple_type,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
    });
    components.register_attribute_decl(xsi_schema_location);

    let xsi_no_namespace_schema_location = components.create(AttributeDeclaration {
        name: "noNamespaceSchemaLocation".into(),
        target_namespace: Some(XSI_NAMESPACE.into()),
        type_definition: any_uri,
        scope: attribute_decl::Scope::new_global(),
        value_constraint: None,
        annotations: Sequence::new(),
        inheritable: false,
    });
    components.register_attribute_decl(xsi_no_namespace_schema_location);
}
