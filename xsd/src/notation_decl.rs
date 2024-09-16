use super::{
    annotation::Annotation,
    components::{Component, Named, NamedXml},
    error::XsdError,
    mapping_context::TopLevelMappable,
    values::actual_value,
    xstypes::{AnyURI, NCName, QName, Sequence},
    MappingContext, Ref,
};
use roxmltree::Node;

/// Schema Component: Notation Declaration, a kind of Annotated Component (§3.14)
#[derive(Clone, Debug)]
pub struct NotationDeclaration {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub system_identifier: Option<AnyURI>,
    pub public_identifier: Option<String>, // TODO publicID
}

impl NamedXml for NotationDeclaration {
    fn get_name_from_xml(notation: Node, schema: Node) -> QName {
        // {name} The ·actual value· of the name [attribute]
        let name = notation
            .attribute("name")
            .map(|v| actual_value::<NCName>(v, notation))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<AnyURI>(v, notation));

        QName::with_optional_namespace(target_namespace, name)
    }
}

impl NotationDeclaration {
    pub const TAG_NAME: &'static str = "notation";

    // TODO §3.14.6
    pub(super) fn map_from_xml(
        context: &mut MappingContext,
        notation: Node,
        schema: Node,
        tlref: Option<Ref<Self>>,
    ) -> Result<Ref<Self>, XsdError> {
        assert_eq!(notation.tag_name().name(), Self::TAG_NAME);

        let self_ref = tlref.unwrap_or_else(|| context.reserve());

        let QName {
            local_name: name,
            namespace_name: target_namespace,
        } = Self::get_name_from_xml(notation, schema);

        // {system identifier}
        //   The ·actual value· of the system [attribute], if present, otherwise ·absent·.
        let system_identifier = notation
            .attribute("system")
            .map(|v| actual_value::<AnyURI>(v, notation));

        // {public identifier}
        //   The ·actual value· of the public [attribute], if present, otherwise ·absent·.
        let public_identifier = notation
            .attribute("public")
            .map(|v| actual_value::<AnyURI>(v, notation));

        // {annotations}
        //   The ·annotation mapping· of the <notation> element, as defined in XML Representation
        //   of Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, notation);

        Ok(context.insert(
            self_ref,
            Self {
                annotations,
                name,
                target_namespace,
                system_identifier,
                public_identifier,
            },
        ))
    }
}

impl Component for NotationDeclaration {
    const DISPLAY_NAME: &'static str = "NotationDeclaration";
}

impl Named for NotationDeclaration {
    fn name(&self) -> Option<QName> {
        Some(QName::with_optional_namespace(
            self.target_namespace.as_ref(),
            &self.name,
        ))
    }
}

impl TopLevelMappable for NotationDeclaration {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        notation: Node,
        schema: Node,
    ) -> Result<(), XsdError> {
        Self::map_from_xml(context, notation, schema, Some(self_ref))?;
        Ok(())
    }
}
