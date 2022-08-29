use super::{
    annotation::Annotation,
    attribute_decl::{self, AttributeDeclaration},
    attribute_use::AttributeUse,
    values::actual_value,
    wildcard::Wildcard,
    xstypes::{AnyURI, NCName, Sequence, Set},
    MappingContext, Ref, RefVisitor, RefsVisitable,
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

impl AttributeGroupDefinition {
    pub fn map_from_xml(
        context: &mut MappingContext,
        attribute_group: Node,
        schema: Node,
    ) -> Ref<Self> {
        let attrib_group_ref = context.components.reserve();

        // {name}
        //   The ·actual value· of the name [attribute]
        let name = attribute_group
            .attribute("name")
            .map(|v| actual_value(v, attribute_group))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value(v, attribute_group));

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
            let (_attribute_decl, attribute_use) = AttributeDeclaration::map_from_xml_local(
                context,
                attribute,
                schema,
                attribute_decl::ScopeParent::AttributeGroup(attrib_group_ref),
            );
            attribute_uses.push(attribute_use);
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
        let mut annot_elements = Vec::new();
        annot_elements.push(attribute_group);
        attribute_group
            .children()
            .filter(|c| c.tag_name().name() == "attributeGroup")
            .for_each(|c| annot_elements.push(c));
        let annotations = Annotation::xml_element_set_annotation_mapping(context, &annot_elements);

        context.components.populate(
            attrib_group_ref,
            Self {
                annotations,
                name,
                target_namespace,
                attribute_uses,
                attribute_wildcard,
            },
        );
        attrib_group_ref
    }
}

impl RefsVisitable for AttributeGroupDefinition {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annotation| visitor.visit_ref(annotation));
        self.attribute_uses
            .iter_mut()
            .for_each(|attrib_use| visitor.visit_ref(attrib_use));
        if let Some(ref mut wildcard) = self.attribute_wildcard {
            visitor.visit_ref(wildcard);
        }
    }
}
