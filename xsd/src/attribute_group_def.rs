use super::{
    annotation::Annotation,
    attribute_decl::{self, AttributeDeclaration},
    attribute_use::AttributeUse,
    components::{Component, Named, NamedXml},
    error::XsdError,
    mapping_context::TopLevelMappable,
    values::actual_value,
    wildcard::Wildcard,
    xstypes::{AnyURI, NCName, QName, Sequence, Set},
    MappingContext, Ref,
};
use roxmltree::Node;

/// Schema Component: Attribute Group Definition, a kind of Annotated Component (§3.6)
#[derive(Clone, Debug)]
pub struct AttributeGroupDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub attribute_uses: Set<Ref<AttributeUse>>,
    pub attribute_wildcard: Option<Ref<Wildcard>>,
}

impl NamedXml for AttributeGroupDefinition {
    fn get_name_from_xml(attribute_group: Node, schema: Node) -> QName {
        // {name}
        //   The ·actual value· of the name [attribute]
        let name = attribute_group
            .attribute("name")
            .map(|v| actual_value::<NCName>(v, attribute_group))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<AnyURI>(v, attribute_group));

        QName::with_optional_namespace(target_namespace, name)
    }
}

impl AttributeGroupDefinition {
    pub const TAG_NAME: &'static str = "attributeGroup";

    pub(super) fn map_from_xml(
        context: &mut MappingContext,
        attribute_group: Node,
        schema: Node,
        attrib_group_ref: Option<Ref<Self>>,
    ) -> Result<Ref<Self>, XsdError> {
        let attrib_group_ref = attrib_group_ref.unwrap_or_else(|| context.reserve());

        let QName {
            local_name: name,
            namespace_name: target_namespace,
        } = Self::get_name_from_xml(attribute_group, schema);

        // {attribute uses}
        //     The union of the set of attribute uses corresponding to the <attribute> [children],
        //     if any, with the {attribute uses} of the attribute groups ·resolved· to by the
        //     ·actual value·s of the ref [attribute] of the <attributeGroup> [children], if any.
        // TODO set union
        let mut attribute_uses = Set::new();
        for attribute in attribute_group
            .children()
            .filter(|c| c.tag_name().name() == "attribute")
        {
            let attribute_use = AttributeDeclaration::map_from_xml_local(
                context,
                attribute,
                schema,
                attribute_decl::ScopeParent::AttributeGroup(attrib_group_ref),
            )?;
            if let Some(attribute_use) = attribute_use {
                attribute_uses.push(attribute_use);
            }
        }

        // {attribute wildcard}
        //   The Wildcard determined by applying the attribute-wildcard mapping described in Common
        //   Rules for Attribute Wildcards (§3.6.2.2) to the <attributeGroup> element information
        //   item.
        // TODO
        let attribute_wildcard = None;

        // {annotations}
        //   The ·annotation mapping· of the <attributeGroup> element and its <attributeGroup>
        //   [children], if present, as defined in XML Representation of Annotation Schema
        //   Components (§3.15.2).
        let mut annot_elements = vec![attribute_group];
        attribute_group
            .children()
            .filter(|c| c.tag_name().name() == "attributeGroup")
            .for_each(|c| annot_elements.push(c));
        let annotations = Annotation::xml_element_set_annotation_mapping(context, &annot_elements);

        Ok(context.insert(
            attrib_group_ref,
            Self {
                annotations,
                name,
                target_namespace,
                attribute_uses,
                attribute_wildcard,
            },
        ))
    }
}

impl Named for AttributeGroupDefinition {
    fn name(&self) -> Option<QName> {
        Some(QName::with_optional_namespace(
            self.target_namespace.as_ref(),
            &self.name,
        ))
    }
}

impl Component for AttributeGroupDefinition {
    const DISPLAY_NAME: &'static str = "AttributeGroupDefinition";
}

impl TopLevelMappable for AttributeGroupDefinition {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        attribute_group: Node,
        schema: Node,
    ) -> Result<(), XsdError> {
        Self::map_from_xml(context, attribute_group, schema, Some(self_ref))?;
        Ok(())
    }
}
