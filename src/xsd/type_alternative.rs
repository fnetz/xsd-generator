use super::{
    annotation::Annotation, assertion::XPathExpression, components::Component,
    shared::TypeDefinition, xstypes::Sequence, Ref,
};
use roxmltree::Node;

/// Schema Component: Type Alternative, a kind of Annotated Component (§3.12)
#[derive(Clone, Debug)]
pub struct TypeAlternative {
    pub annotations: Sequence<Ref<Annotation>>,
    pub test: Option<XPathExpression>,
    pub type_definition: TypeDefinition,
}

impl TypeAlternative {
    pub fn map_from_xml(_alternative: Node, _schema: Node) -> Ref<Self> {
        todo!()
    }
}

impl Component for TypeAlternative {
    const DISPLAY_NAME: &'static str = "TypeAlternative";
}
