use super::{
    annotation::Annotation,
    attribute_decl::AttributeDeclaration,
    components::{AnnotatedComponent, Component},
    shared,
    xstypes::Sequence,
    Ref,
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

impl Component for AttributeUse {
    const DISPLAY_NAME: &'static str = "AttributeUse";
}

impl AnnotatedComponent for AttributeUse {
    fn annotations(&self) -> &[Ref<Annotation>] {
        &self.annotations
    }
}
