use super::{
    components::{Component, Named, NamedXml},
    mapping_context::TopLevelMappable,
    shared,
    values::{actual_value, normalized_value},
    xstypes::{AnyURI, NCName, QName, Sequence},
    Annotation, AttributeGroupDefinition, AttributeUse, ComplexTypeDefinition, MappingContext, Ref,
    SimpleTypeDefinition,
};

use roxmltree::Node;

/// Schema Component: Attribute Declaration, a kind of Annotated Component (§3.2)
#[derive(Clone, Debug)]
pub struct AttributeDeclaration {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub type_definition: Ref<SimpleTypeDefinition>,
    pub scope: Scope,
    pub value_constraint: Option<ValueConstraint>,
    pub inheritable: bool,
}

/// Property Record: Scope (§3.2)
pub type Scope = shared::Scope<ScopeParent>;

pub use shared::ScopeVariety;

#[derive(Clone, Debug)]
pub enum ScopeParent {
    ComplexType(Ref<ComplexTypeDefinition>),
    AttributeGroup(Ref<AttributeGroupDefinition>),
}

/// Property Record: Value Constraint (§3.2)
pub use shared::ValueConstraint;
pub use shared::ValueConstraintVariety;

impl NamedXml for AttributeDeclaration {
    fn get_name_from_xml(attribute: Node, schema: Node) -> QName {
        assert_eq!(attribute.tag_name().name(), Self::TAG_NAME);

        // {name}
        //   The ·actual value· of the name [attribute]
        let name = attribute
            .attribute("name")
            .map(|v| actual_value::<String>(v, attribute))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the parent <schema> element
        //   information item, or ·absent· if there is none.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, attribute));

        QName::with_optional_namespace(target_namespace, name)
    }
}

impl AttributeDeclaration {
    // TODO validate §3.2.3
    pub const TAG_NAME: &'static str = "attribute";

    pub(super) fn map_from_xml_global(
        context: &mut MappingContext,
        attribute: Node,
        schema: Node,
        tlref: Ref<Self>,
    ) -> Ref<Self> {
        assert_eq!(attribute.tag_name().name(), Self::TAG_NAME);

        let QName {
            local_name: name,
            namespace_name: target_namespace,
        } = Self::get_name_from_xml(attribute, schema);

        // {type definition}
        //   The simple type definition corresponding to the <simpleType> element information item in
        //   the [children], if present, otherwise the simple type definition ·resolved· to by the
        //   ·actual value· of the type [attribute], if present, otherwise ·xs:anySimpleType·.
        let simple_type_def = attribute
            .children()
            .find(|c| c.tag_name().name() == SimpleTypeDefinition::TAG_NAME)
            .map(|simple_type| {
                SimpleTypeDefinition::map_from_xml(context, simple_type, schema, None)
            });

        let type_definition = if let Some(simple_type_def) = simple_type_def {
            simple_type_def
        } else if let Some(type_) = attribute
            .attribute("type")
            .map(|v| actual_value::<QName>(v, attribute))
        {
            context.resolve(&type_)
        } else {
            todo!("xs:anySimpleType")
        };

        // {scope}
        //   A scope as follows:
        //     {variety} global
        //     {parent}  ·absent·
        let scope = Scope::new_global();

        // {value constraint}
        //   If there is a default or a fixed [attribute], then a Value Constraint as follows,
        //   otherwise ·absent·.
        //     {variety}      either default or fixed, as appropriate
        //     {value}        the ·actual value· (with respect to the {type definition}) of the
        //                    [attribute]
        //     {lexical form} the ·normalized value· (with respect to the {type definition}) of the
        //                    [attribute]
        let value_constraint = {
            let attrib = attribute
                .attribute("default")
                .map(|default| (default, ValueConstraintVariety::Default))
                .or_else(|| {
                    attribute
                        .attribute("fixed")
                        .map(|fixed| (fixed, ValueConstraintVariety::Fixed))
                });

            attrib.map(|(value, variety)| ValueConstraint {
                variety,
                value: actual_value::<String>(value, attribute),
                lexical_form: normalized_value(value).to_string(),
            })
        };

        // {inheritable}
        //   The ·actual value· of the inheritable [attribute], if present, otherwise false.
        let inheritable = attribute
            .attribute("inheritable")
            .map(|v| actual_value::<bool>(v, attribute))
            .unwrap_or(false);

        // {annotations}
        //   The ·annotation mapping· of the <attribute> element, as defined in XML Representation of
        //   Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, attribute);

        context.insert(
            tlref,
            AttributeDeclaration {
                annotations,
                name,
                target_namespace,
                type_definition,
                scope,
                value_constraint,
                inheritable,
            },
        )
    }

    // TODO Extract common attribute procedures
    pub(super) fn map_from_xml_local(
        context: &mut MappingContext,
        attribute: Node,
        schema: Node,
        parent: ScopeParent,
        // TODO is the first needed?
    ) -> (Option<Ref<Self>>, Ref<AttributeUse>) {
        assert_eq!(attribute.tag_name().name(), Self::TAG_NAME);

        // TODO handle use = 'prohibited'

        // == Common properties for both paths ==

        // {required}
        //   true if use = required, otherwise false.
        let required = attribute.attribute("use") == Some("required");

        // {annotations}
        //   The ·annotation mapping· of the <attribute> element, as defined in XML Representation of
        //   Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, attribute);

        // Decide whether the attribute is a reference to a top-level attribute declaration (if ref
        // [attribute] is present) or a local attribute declaration.
        let ref_ = attribute.attribute("ref");
        if let Some(ref_) = ref_ {
            // ===== Attribute Use =====

            // {attribute declaration}
            //   The (top-level) attribute declaration ·resolved· to by the ·actual value· of the ref
            //   [attribute]
            let ref_ = actual_value::<QName>(ref_, attribute);
            let attribute_declaration: Ref<AttributeDeclaration> = context.resolve(&ref_);

            // {value constraint}
            //   If there is a default or a fixed [attribute], then a Value Constraint as follows,
            //   otherwise ·absent·.
            //     {variety}
            //       either default or fixed, as appropriate
            //     {value}
            //       the ·actual value· of the [attribute] (with respect to {attribute
            //       declaration}.{type definition})
            //     {lexical form}
            //       the ·normalized value· of the [attribute] (with respect to {attribute
            //       declaration}.{type definition})
            let value_constraint = {
                let attrib = attribute
                    .attribute("default")
                    .map(|default| (default, ValueConstraintVariety::Default))
                    .or_else(|| {
                        attribute
                            .attribute("fixed")
                            .map(|fixed| (fixed, ValueConstraintVariety::Fixed))
                    });

                attrib.map(|(value, variety)| ValueConstraint {
                    variety,
                    value: actual_value::<String>(value, attribute),
                    lexical_form: normalized_value(value).to_string(),
                })
            };

            // {inheritable}
            //   The ·actual value· of the inheritable [attribute], if present, otherwise {attribute
            //   declaration}.{inheritable}.
            let inheritable = attribute
                .attribute("inheritable")
                .map(|v| actual_value::<bool>(v, attribute))
                .unwrap_or_else(|| context.request(attribute_declaration).inheritable);

            let attribute_use = context.create(AttributeUse {
                annotations,
                attribute_declaration,
                value_constraint,
                required,
                inheritable,
            });

            (None, attribute_use)
        } else {
            // ===== Attribute Declaration =====

            // {name}
            //   The ·actual value· of the name [attribute]
            let name = attribute
                .attribute("name")
                .map(|v| actual_value::<String>(v, attribute))
                .unwrap();

            // {target namespace}
            //   The appropriate case among the following:
            //   1 If targetNamespace is present, then its ·actual value·.
            //   2 If targetNamespace is not present and one of the following is true
            //     2.1 form = qualified
            //     2.2 form is absent and the <schema> ancestor has attributeFormDefault = qualified
            //     then the ·actual value· of the targetNamespace [attribute] of the ancestor <schema>
            //     element information item, or ·absent· if there is none.
            //   3 otherwise ·absent·.
            let target_namespace =
                if let Some(target_namespace) = attribute.attribute("targetNamespace") {
                    Some(actual_value::<String>(target_namespace, attribute))
                } else {
                    let form = attribute
                        .attribute("form")
                        .or_else(|| schema.attribute("attributeFormDefault"));
                    if form == Some("qualified") {
                        schema
                            .attribute("targetNamespace")
                            .map(|v| actual_value::<String>(v, attribute))
                    } else {
                        None
                    }
                };

            // {type definition}
            //   The simple type definition corresponding to the <simpleType> element information
            //   item in the [children], if present, otherwise the simple type definition
            //   ·resolved· to by the ·actual value· of the type [attribute], if present, otherwise
            //   ·xs:anySimpleType·.
            let simple_type_def = attribute
                .children()
                .find(|c| c.tag_name().name() == "simpleType")
                .map(|simple_type| {
                    SimpleTypeDefinition::map_from_xml(context, simple_type, schema, None)
                });

            let type_definition = if let Some(simple_type_def) = simple_type_def {
                simple_type_def
            } else if let Some(type_) = attribute
                .attribute("type")
                .map(|v| actual_value::<QName>(v, attribute))
            {
                context.resolve(&type_)
            } else {
                todo!("xs:anySimpleType")
            };

            // {scope} A Scope as follows:
            //   {variety} local
            //   {parent}  If the <attribute> element information item has <complexType> as an
            //             ancestor, the Complex Type Definition corresponding to that item,
            //             otherwise (the <attribute> element information item is within an
            //             <attributeGroup> element information item), the Attribute Group
            //             Definition corresponding to that item.
            let scope = Scope::new_local(parent);

            // {value constraint}
            //   ·absent·.
            let value_constraint = None;

            // {inheritable}
            //   The ·actual value· of the inheritable [attribute], if present, otherwise false.
            let inheritable = attribute
                .attribute("inheritable")
                .map(|v| actual_value::<bool>(v, attribute))
                .unwrap_or(false);

            let attribute_declaration = context.create(AttributeDeclaration {
                annotations: annotations.clone(),
                name,
                target_namespace,
                type_definition,
                scope,
                value_constraint,
                inheritable,
            });

            // ===== Attribute Use =====

            // {attribute declaration}
            //   -- just constructed above --

            // {value constraint}
            //   If there is a default or a fixed [attribute], then a Value Constraint as follows,
            //   otherwise ·absent·.
            //     {variety}
            //       either default or fixed, as appropriate
            //     {value}
            //       the ·actual value· of the [attribute] (with respect to {attribute
            //       declaration}.{type definition})
            //     {lexical form}
            //       the ·normalized value· of the [attribute] (with respect to {attribute
            //       declaration}.{type definition})
            let value_constraint = {
                let attrib = attribute
                    .attribute("default")
                    .map(|default| (default, ValueConstraintVariety::Default))
                    .or_else(|| {
                        attribute
                            .attribute("fixed")
                            .map(|fixed| (fixed, ValueConstraintVariety::Fixed))
                    });

                attrib.map(|(value, variety)| ValueConstraint {
                    variety,
                    value: actual_value::<String>(value, attribute),
                    lexical_form: normalized_value(value).to_string(),
                })
            };

            // {inheritable}
            //   -- reused from {attribute_declaration}.{inheritable} --

            let attribute_use = context.create(AttributeUse {
                annotations,
                attribute_declaration,
                value_constraint,
                required,
                inheritable,
            });

            (Some(attribute_declaration), attribute_use)
        }
    }
}

impl Component for AttributeDeclaration {
    const DISPLAY_NAME: &'static str = "AttributeDeclaration";
}

impl Named for AttributeDeclaration {
    fn name(&self) -> Option<QName> {
        Some(QName::with_optional_namespace(
            self.target_namespace.as_ref(),
            &self.name,
        ))
    }
}

impl TopLevelMappable for AttributeDeclaration {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        attribute: Node,
        schema: Node,
    ) {
        // TODO inline?
        Self::map_from_xml_global(context, attribute, schema, self_ref);
    }
}
