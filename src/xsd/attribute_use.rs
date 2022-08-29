use super::{
    annotation::Annotation, attribute_decl::AttributeDeclaration, shared, xstypes::Sequence, Ref,
    RefsVisitable,
};

/// Schema Component: Attribute Use, a kind of Annotated Component (§3.5)
#[derive(Clone, Debug)]
pub struct AttributeUse {
    pub annotations: Sequence<Ref<Annotation>>,
    pub required: bool,
    pub attribute_declaration: Ref<AttributeDeclaration>,
    pub value_constraint: Option<ValueConstraint>,
    pub inheritable: bool,
}

/// Property Record: Value Constraint (§3.5)
pub use shared::ValueConstraint;

impl RefsVisitable for AttributeUse {
    fn visit_refs(&mut self, visitor: &mut impl super::RefVisitor) {
        visitor.visit_ref(&mut self.attribute_declaration);
        for annotation in self.annotations.iter_mut() {
            visitor.visit_ref(annotation);
        }
    }
}
