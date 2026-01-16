use crate::{
    Ref,
    annotation::Annotation,
    assertion::XPathExpression,
    components::{AnnotatedComponent, Component},
    error::XsdError,
    shared::TypeDefinition,
    xstypes::Sequence,
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

impl AnnotatedComponent for TypeAlternative {
    fn annotations(&self) -> &[Ref<Annotation>] {
        &self.annotations
    }
}

pub(crate) struct TypeAlternativeP0 {}

impl TypeAlternativeP0 {
    pub(crate) fn map_from_xml(alternative: Node, schema: Node) -> Result<Self, XsdError> {
        todo!("type alternative phase 0")
    }

    pub(crate) fn when_alternative_has_test() -> Result<Self, XsdError> {
        todo!()
    }
}
