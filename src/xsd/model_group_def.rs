use super::{
    annotation::Annotation,
    model_group::ModelGroup,
    xstypes::{AnyURI, NCName, Sequence},
    MappingContext, Ref, RefVisitor, RefsVisitable,
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

impl ModelGroupDefinition {
    pub fn map_from_xml(_context: &mut MappingContext, _group: Node, _schema: Node) -> Ref<Self> {
        todo!()
    }
}

impl RefsVisitable for ModelGroupDefinition {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        visitor.visit_ref(&mut self.model_group);
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
    }
}
