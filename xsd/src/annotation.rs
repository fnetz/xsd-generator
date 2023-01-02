use super::components::Component;
use super::xstypes::Sequence;
use super::{MappingContext, Ref};
use roxmltree::Node;

/// Schema Component: Annotation, a kind of Component (§3.15)
///
/// Note: Instead of storing the actual Element information items (i.e. nodes), this type stores the
/// original source string of the respective elements.
#[derive(Clone, Debug)]
pub struct Annotation {
    pub application_information: Sequence<String>,
    pub user_information: Sequence<String>,
    pub attributes: Sequence<()>,
}

impl Annotation {
    pub const TAG_NAME: &'static str = "annotation";

    /// Generate a textual representation of the content of an annotation.
    /// This is not meant to be the exact source or event efficient; it should however be able to
    /// parse back into similar XML.
    fn content_to_text(node: Node) -> String {
        use std::fmt::Write;

        let mut text = String::new();
        for child in node.children() {
            match child.node_type() {
                roxmltree::NodeType::Text => text.push_str(child.text().unwrap()),
                roxmltree::NodeType::Element => {
                    let tag_name = child.tag_name().name();
                    write!(
                        text,
                        "<{tag_name}>{}</{tag_name}>",
                        Self::content_to_text(child)
                    )
                    .unwrap();
                }
                _ => {}
            }
        }
        text
    }

    pub(super) fn map_from_xml(context: &mut MappingContext, annotation: Node) -> Ref<Self> {
        assert_eq!(annotation.tag_name().name(), Self::TAG_NAME);

        // {application information}
        //   A sequence of the <appinfo> element information items from among the [children], in
        //   order, if any, otherwise the empty sequence.
        let application_information = annotation
            .children()
            .filter(|child| child.tag_name().name() == "appinfo")
            .map(Self::content_to_text)
            .collect();

        // {user information}
        //   A sequence of the <documentation> element information items from among the [children],
        //   in order, if any, otherwise the empty sequence.
        let user_information = annotation
            .children()
            .filter(|child| child.tag_name().name() == "documentation")
            .map(Self::content_to_text)
            .collect();

        // {attributes}
        //   A set of attribute information items, namely those allowed by the attribute wildcard
        //   in the type definition for the <annotation> item itself or for the enclosing items
        //   which correspond to the component within which the annotation component is located.
        // TODO
        let attributes = Sequence::new();

        context.create(Self {
            application_information,
            user_information,
            attributes,
        })
    }

    /// Corresponds to the [annotation mapping](https://www.w3.org/TR/xmlschema11-1/#key-am-set) of
    /// a set of element information items (represented here as slice for simplicity)
    pub(super) fn xml_element_set_annotation_mapping(
        context: &mut MappingContext,
        es: &[Node],
    ) -> Sequence<Ref<Self>> {
        // FIXME Actually implement the annotation mapping procedure in the spec
        let mut as_ = Sequence::new();
        for e in es {
            let child_annotations = e
                .children()
                .filter(|c| c.tag_name().name() == Self::TAG_NAME);
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
    pub(super) fn xml_element_annotation_mapping(
        context: &mut MappingContext,
        e: Node,
    ) -> Sequence<Ref<Self>> {
        Self::xml_element_set_annotation_mapping(context, &[e])
    }
}

impl Component for Annotation {
    const DISPLAY_NAME: &'static str = "Annotation";
}
