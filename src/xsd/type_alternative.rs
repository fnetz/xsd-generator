use super::{
    annotation::Annotation, assertion::XPathExpression, shared::TypeDefinition, xstypes::Sequence,
    Ref, RefVisitor, RefsVisitable,
};
use roxmltree::Node;

/// Schema Component: Type Alternative, a kind of Annotated Component (§3.12)
#[derive(Clone, Debug)]
pub struct TypeAlternative {
    pub annotations: Sequence<Ref<Annotation>>,
    pub test: Option<XPathExpression>,
    pub type_definition: Ref<TypeDefinition>,
}

impl TypeAlternative {
    pub fn map_from_xml(_alternative: Node, _schema: Node) -> Ref<Self> {
        todo!()
    }
}

impl RefsVisitable for TypeAlternative {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
        visitor.visit_ref(&mut self.type_definition);
    }
}
