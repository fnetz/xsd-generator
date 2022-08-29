use super::{
    annotation::Annotation,
    assertion::XPathExpression,
    xstypes::{AnyURI, NCName, Sequence},
    Ref, RefVisitor, RefsVisitable,
};

/// Schema Component: Identity-Constraint Definition, a kind of Annotated Component (§3.11)
#[derive(Clone, Debug)]
pub struct IdentityConstraintDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub identity_constraint_category: IdentityConstraintCategory,
    pub selector: XPathExpression,
    pub fields: Sequence<XPathExpression>,
    pub referenced_key: Option<Ref<IdentityConstraintDefinition>>,
}

#[derive(Clone, Debug)]
pub enum IdentityConstraintCategory {
    Key,
    KeyRef,
    Unique,
}

impl RefsVisitable for IdentityConstraintDefinition {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
        self.referenced_key
            .as_mut()
            .map(|key| visitor.visit_ref(key));
    }
}
