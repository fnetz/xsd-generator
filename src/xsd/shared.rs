use super::{
    complex_type_def::ComplexTypeDefinition, element_decl::ElementDeclaration,
    model_group::ModelGroup, simple_type_def::SimpleTypeDefinition, wildcard::Wildcard, Ref,
    RefVisitor, RefsVisitable,
};

/// Common type for [attribute_decl::ScopeVariety](super::attribute_decl::ScopeVariety) and
/// [element_decl::ScopeVariety](super::element_decl::ScopeVariety)
#[derive(Clone, Debug)]
pub enum ScopeVariety {
    Global,
    Local,
}

#[derive(Clone, Debug)]
pub enum ValueConstraintVariety {
    Default,
    Fixed,
}

/// Property Record: Value Constraint -- Common type for
/// [attribute_decl::ValueConstraint](super::attribute_decl::ValueConstraint),
/// [element_decl::ValueConstraint](super::element_decl::ValueConstraint) and
/// [attribute_use::ValueConstraint](super::attribute_use::ValueConstraint)
#[derive(Clone, Debug)]
pub struct ValueConstraint {
    pub variety: ValueConstraintVariety,
    pub value: String, // TODO *actual value*
    pub lexical_form: String,
}

/// Supertype of [simple](SimpleTypeDefinition) and [complex](ComplexTypeDefinition) type
/// definition (§2.2.1)
#[derive(Copy, Clone, Debug)]
pub enum TypeDefinition {
    Simple(Ref<SimpleTypeDefinition>),
    Complex(Ref<ComplexTypeDefinition>),
}

impl TypeDefinition {
    pub fn simple(self) -> Option<Ref<SimpleTypeDefinition>> {
        match self {
            Self::Simple(simple) => Some(simple),
            Self::Complex(_) => None,
        }
    }

    pub fn complex(self) -> Option<Ref<ComplexTypeDefinition>> {
        match self {
            Self::Complex(complex) => Some(complex),
            Self::Simple(_) => None,
        }
    }
}

impl RefsVisitable for TypeDefinition {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        match self {
            Self::Simple(simple) => visitor.visit_ref(simple),
            Self::Complex(complex) => visitor.visit_ref(complex),
        }
    }
}

/// Supertype of the three types that can appear in [Particle](super::Particle)s (§2.2.3.2)
#[derive(Clone, Debug)]
pub enum Term {
    ElementDeclaration(Ref<ElementDeclaration>),
    ModelGroup(Ref<ModelGroup>),
    Wildcard(Ref<Wildcard>),
}

impl RefsVisitable for Term {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        match self {
            Self::ElementDeclaration(element) => visitor.visit_ref(element),
            Self::ModelGroup(model_group) => visitor.visit_ref(model_group),
            Self::Wildcard(wildcard) => visitor.visit_ref(wildcard),
        }
    }
}
