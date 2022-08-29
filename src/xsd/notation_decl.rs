use super::{
    annotation::Annotation,
    values::actual_value,
    xstypes::{AnyURI, NCName, Sequence},
    MappingContext, Ref, RefVisitor, RefsVisitable,
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

impl NotationDeclaration {
    // TODO §3.14.6
    pub fn map_from_xml(context: &mut MappingContext, notation: Node, schema: Node) -> Ref<Self> {
        assert_eq!(notation.tag_name().name(), "notation");

        // {name}
        //   The ·actual value· of the name [attribute]
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

        context.components.create(Self {
            annotations,
            name,
            target_namespace,
            system_identifier,
            public_identifier,
        })
    }
}

impl RefsVisitable for NotationDeclaration {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
    }
}
