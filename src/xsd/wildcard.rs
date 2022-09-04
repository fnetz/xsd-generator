use super::{
    annotation::Annotation,
    components::Component,
    xstypes::{AnyURI, QName, Sequence, Set},
    Ref,
};

/// Schema Component: Wildcard, a kind of [Term](super::shared::Term) (§3.10)
#[derive(Clone, Debug)]
pub struct Wildcard {
    pub annotations: Sequence<Ref<Annotation>>,
    pub namespace_constraint: NamespaceConstraint,
    pub process_contents: ProcessContents,
}

#[derive(Clone, Debug)]
pub enum ProcessContents {
    Skip,
    Strict,
    Lax,
}

/// Property Record: Namespace Constraint (§3.10)
#[derive(Clone, Debug)]
pub struct NamespaceConstraint {
    pub variety: NamespaceConstraintVariety,
    pub namespaces: Set<Option<AnyURI>>,
    pub disallowed_names: Set<DisallowedName>,
}

#[derive(Clone, Debug)]
pub enum NamespaceConstraintVariety {
    Any,
    Enumeration,
    Not,
}

#[derive(Clone, Debug)]
pub enum DisallowedName {
    QName(QName),
    Defined,
    Sibling,
}

impl Component for Wildcard {
    const DISPLAY_NAME: &'static str = "Wildcard";
}
