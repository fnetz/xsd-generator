use super::xstypes::Sequence;
use super::{MappingContext, Ref, RefVisitor, RefsVisitable};
use roxmltree::Node;

/// Schema Component: Annotation, a kind of Component (§3.15)
///
/// Note: Instead of storing the actual Element information items (i.e. nodes), we store the
/// original source string of the element.
#[derive(Clone, Debug)]
pub struct Annotation {
    pub application_information: Sequence<String>,
    pub user_information: Sequence<String>,
    pub attributes: Sequence<()>,
}

impl Annotation {
    fn source_string(node: Node) -> String {
        node.document().input_text()[node.range()].to_string()
    }

    pub fn map_from_xml(context: &mut MappingContext, annotation: Node) -> Ref<Self> {
        assert_eq!(annotation.tag_name().name(), "annotation");

        // {application information}
        //   A sequence of the <appinfo> element information items from among the [children], in
        //   order, if any, otherwise the empty sequence.
        let application_information = annotation
            .children()
            .filter(|child| child.tag_name().name() == "appinfo")
            .map(Self::source_string)
            .collect();

        // {user information}
        //   A sequence of the <documentation> element information items from among the [children],
        //   in order, if any, otherwise the empty sequence.
        let user_information = annotation
            .children()
            .filter(|child| child.tag_name().name() == "documentation")
            .map(Self::source_string)
            .collect();

        // {attributes}
        //   A set of attribute information items, namely those allowed by the attribute wildcard
        //   in the type definition for the <annotation> item itself or for the enclosing items
        //   which correspond to the component within which the annotation component is located.
        // TODO
        let attributes = Sequence::new();

        context.components.create(Self {
            application_information,
            user_information,
            attributes,
        })
    }

    /// Corresponds to the [annotation mapping](https://www.w3.org/TR/xmlschema11-1/#key-am-set) of
    /// a set of element information items (represented here as slice for simplicity)
    pub fn xml_element_set_annotation_mapping(
        context: &mut MappingContext,
        es: &[Node],
    ) -> Sequence<Ref<Self>> {
        // FIXME Actually implement the annotation mapping procedure in the spec
        let mut as_ = Sequence::new();
        for e in es {
            let child_annotations = e.children().filter(|c| c.tag_name().name() == "annotation");
            for child in child_annotations {
                as_.push(Self::map_from_xml(context, child));
            }
        }
        as_
    }

    /// Shorthand for the [annotation mapping](https://www.w3.org/TR/xmlschema11-1/#key-am-single)
    /// of a single element information item (same as calling
    /// [xml_element_set_annotation_mapping()](Self::xml_element_set_annotation_mapping()) with a
    /// single-element slice containing `e`)
    pub fn xml_element_annotation_mapping(
        context: &mut MappingContext,
        e: Node,
    ) -> Sequence<Ref<Self>> {
        Self::xml_element_set_annotation_mapping(context, &[e])
    }
}

impl RefsVisitable for Annotation {
    fn visit_refs(&mut self, _visitor: &mut impl RefVisitor) {}
}
