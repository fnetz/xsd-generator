use super::{
    annotation::Annotation,
    components::{AnnotatedComponent, Component, Named, NamedXml},
    element_decl,
    error::XsdError,
    mapping_context::TopLevelMappable,
    model_group::ModelGroup,
    values::actual_value,
    xstypes::{AnyURI, NCName, QName, Sequence},
    MappingContext, Particle, Ref,
};
use roxmltree::Node;

/// Schema Component: Model Group Definition, a kind of Annotated Component (§3.7)
#[derive(Clone, Debug)]
pub struct ModelGroupDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub model_group: Ref<ModelGroup>,
}

impl NamedXml for ModelGroupDefinition {
    fn get_name_from_xml(group: Node, schema: Node) -> QName {
        // {name} The ·actual value· of the name [attribute]
        let name = group
            .attribute("name")
            .map(|n| actual_value::<NCName>(n, group))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|t| actual_value::<AnyURI>(t, schema));

        QName::with_optional_namespace(target_namespace, name)
    }
}

impl ModelGroupDefinition {
    pub const TAG_NAME: &'static str = "group";

    pub(super) fn map_from_xml(
        context: &mut MappingContext,
        group: Node,
        schema: Node,
        tlref: Option<Ref<Self>>,
    ) -> Result<Ref<Self>, XsdError> {
        // {name}, {target namespace}
        //   [see `get_name_from_xml()` above.]
        let QName {
            local_name: name,
            namespace_name: target_namespace,
        } = Self::get_name_from_xml(group, schema);

        let self_ref = tlref.unwrap_or_else(|| context.reserve());

        // {model group}
        //   A model group which is the {term} of a particle corresponding to the <all>, <choice>
        //   or <sequence> among the [children] (there must be exactly one).
        let all_child = group.children().find(|n| n.tag_name().name() == "all");
        let choice_child = group.children().find(|n| n.tag_name().name() == "choice");
        let sequence_child = group.children().find(|n| n.tag_name().name() == "sequence");
        let particle = all_child.xor(choice_child).xor(sequence_child);
        let particle = particle
            .expect("Model group definition needs to have EXACTLY one of all, choice or sequence");

        let model_group = Particle::map_from_xml_model_group_term(
            context,
            particle,
            schema,
            element_decl::ScopeParent::Group(self_ref),
        )?;

        // {annotations}
        //    The ·annotation mapping· of the <group> element, as defined in XML Representation of
        //    Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, group);

        Ok(context.insert(
            self_ref,
            Self {
                annotations,
                name,
                target_namespace,
                model_group,
            },
        ))
    }
}

impl Component for ModelGroupDefinition {
    const DISPLAY_NAME: &'static str = "ModelGroupDefinition";
}

impl AnnotatedComponent for ModelGroupDefinition {
    fn annotations(&self) -> &[Ref<Annotation>] {
        &self.annotations
    }
}

impl Named for ModelGroupDefinition {
    fn name(&self) -> Option<QName> {
        Some(QName::with_optional_namespace(
            self.target_namespace.as_ref(),
            &self.name,
        ))
    }
}

impl TopLevelMappable for ModelGroupDefinition {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        group: Node,
        schema: Node,
    ) -> Result<(), XsdError> {
        Self::map_from_xml(context, group, schema, Some(self_ref))?;
        Ok(())
    }
}
