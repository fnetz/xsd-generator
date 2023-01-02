use super::{
    annotation::Annotation,
    components::{Component, Named, NamedXml},
    mapping_context::TopLevelMappable,
    model_group::{Compositor, ModelGroup},
    values::actual_value,
    xstypes::{AnyURI, NCName, QName, Sequence},
    MappingContext, Ref,
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
    ) -> Ref<Self> {
        let QName {
            local_name: name,
            namespace_name: target_namespace,
        } = Self::get_name_from_xml(group, schema);

        // TODO
        let self_ref = tlref.unwrap_or_else(|| context.reserve());
        let model_group = context.create(ModelGroup {
            annotations: Sequence::new(),
            compositor: Compositor::All,
            particles: Sequence::new(),
        });
        context.insert(
            self_ref,
            Self {
                annotations: Sequence::new(),
                name,
                target_namespace,
                model_group,
            },
        )
    }
}

impl Component for ModelGroupDefinition {
    const DISPLAY_NAME: &'static str = "ModelGroupDefinition";
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
    ) {
        Self::map_from_xml(context, group, schema, Some(self_ref));
    }
}
