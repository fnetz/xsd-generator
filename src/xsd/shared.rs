use super::{
    complex_type_def::ComplexTypeDefinition, element_decl::ElementDeclaration,
    model_group::ModelGroup, simple_type_def::SimpleTypeDefinition, wildcard::Wildcard, Ref,
    RefVisitor, RefsVisitable,
};

/// Common type for [attribute_decl::ScopeVariety](super::attribute_decl::ScopeVariety) and
/// [element_decl::ScopeVariety](super::element_decl::ScopeVariety)
#[derive(Copy, Clone, Debug)]
pub enum ScopeVariety {
    Global,
    Local,
}

/// Base type for [attribute_decl::Scope](super::attribute_decl::Scope) and
/// [element_decl::Scope](super::element_decl::Scope), with `P` being the Scope's parent
/// type.
///
/// This type deviates slightly from the specification to better represent the fact that {parent} is
/// required when the variety is local, and must be absent when the variety is global.
#[derive(Clone, Debug)]
pub enum Scope<P> {
    Global,
    Local(P),
}

impl<P> Scope<P> {
    /// Constructs a new scope with {variety} = global and {parent} = ·absent·
    pub const fn new_global() -> Self {
        Self::Global
    }

    /// Constructs a new scope with {variety} = local and {parent} = `parent`
    pub const fn new_local(parent: P) -> Self {
        Self::Local(parent)
    }

    /// Corresponds to the {variety} property of Scope.
    pub fn variety(&self) -> ScopeVariety {
        match self {
            Self::Global => ScopeVariety::Global,
            Self::Local(_) => ScopeVariety::Local,
        }
    }

    /// Corresponds to the {parent} property of Scope. `Some` if the variety is local,
    /// `None` otherwise
    pub fn parent(&self) -> Option<&P> {
        match self {
            Self::Global => None,
            Self::Local(p) => Some(p),
        }
    }

    /// Same as [`parent()`](Self::parent()) but mutable
    pub(super) fn parent_mut(&mut self) -> Option<&mut P> {
        match self {
            Self::Global => None,
            Self::Local(p) => Some(p),
        }
    }
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
