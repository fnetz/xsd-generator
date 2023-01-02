use super::{
    complex_type_def::ComplexTypeDefinition,
    components::{ComponentTable, Named, RefNamed},
    element_decl::ElementDeclaration,
    model_group::ModelGroup,
    simple_type_def::SimpleTypeDefinition,
    wildcard::Wildcard,
    xstypes::QName,
    Ref,
};

/// Common type for [attribute_decl::ScopeVariety](super::attribute_decl::ScopeVariety) and
/// [element_decl::ScopeVariety](super::element_decl::ScopeVariety)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/// Helper: Resolved [`TypeDefinition`]
enum RTD<'a> {
    Simple(&'a SimpleTypeDefinition),
    Complex(&'a ComplexTypeDefinition),
}

impl RefNamed for TypeDefinition {
    fn name(&self, ct: &impl ComponentTable) -> Option<QName> {
        match self.get(ct) {
            RTD::Simple(s) => s.name(),
            RTD::Complex(c) => c.name(),
        }
    }
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

    fn get<'a>(&self, components: &'a impl ComponentTable) -> RTD<'a> {
        match self {
            Self::Simple(s) => RTD::Simple(s.get(components)),
            Self::Complex(c) => RTD::Complex(c.get(components)),
        }
    }

    pub fn base_type_definition(&self, components: &impl ComponentTable) -> TypeDefinition {
        match self.get(components) {
            RTD::Simple(s) => s.base_type_definition,
            RTD::Complex(c) => c.base_type_definition,
        }
    }

    pub fn ancestors<'a, T: ComponentTable>(&self, components: &'a T) -> Ancestors<'a, T> {
        Ancestors::new(self.base_type_definition(components), components)
    }

    pub fn is_primitive(&self, components: &impl ComponentTable) -> bool {
        match self {
            Self::Simple(s) => s.get(components).is_primitive(),
            Self::Complex(_) => false,
        }
    }
}

/// Supertype of the three types that can appear in [Particle](super::Particle)s (§2.2.3.2)
#[derive(Copy, Clone, Debug)]
pub enum Term {
    ElementDeclaration(Ref<ElementDeclaration>),
    ModelGroup(Ref<ModelGroup>),
    Wildcard(Ref<Wildcard>),
}

impl Term {
    pub fn model_group(self) -> Option<Ref<ModelGroup>> {
        match self {
            Self::ModelGroup(g) => Some(g),
            _ => None,
        }
    }

    pub fn is_basic(&self) -> bool {
        matches!(self, Self::ElementDeclaration(_) | Self::Wildcard(_))
    }
}

/// Iterator over the ancestors of a Type Definition.
/// > The ancestors of a ·type definition· are its {base type definition} and the ·ancestors· of
/// > its {base type definition}. (pt. 1, §3.16.2.2)
///
/// Note that, since the "root" type `xs:anyType`'s base type is itself, this iterator is infinite.
/// In other words, once the anyType is reached, `next()` will forever return `Some(<xs:anyType>)`.
pub struct Ancestors<'a, T: ComponentTable> {
    current: TypeDefinition,
    components: &'a T,
}

impl<'a, T: ComponentTable> Ancestors<'a, T> {
    pub(super) fn new(start: TypeDefinition, components: &'a T) -> Self {
        Self {
            current: start,
            components,
        }
    }
}

impl<'a, T: ComponentTable> Iterator for Ancestors<'a, T> {
    type Item = TypeDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        let base_type = self.current.base_type_definition(self.components);
        let current = self.current;
        self.current = base_type;
        Some(current)
    }
}
