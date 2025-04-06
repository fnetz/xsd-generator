pub mod builder;
pub mod passes;
pub mod xsd_visitor;

use std::collections::HashMap;

use dt_xsd::{
    AttributeDeclaration, AttributeUse, ComplexTypeDefinition, ElementDeclaration, ModelGroup,
    Particle, Ref, SimpleTypeDefinition, TypeDefinition, Wildcard, particle::MaxOccurs,
    xstypes::QName,
};

#[derive(Debug, Clone)]
pub struct Name {
    pub name: String,
    // pub scope: NameScope,
    pub generation: u32,
}

impl Name {
    pub fn new(name: String) -> Self {
        Self {
            name,
            generation: 0,
        }
    }

    pub fn clone_new_generation(&self) -> Self {
        Self {
            name: self.name.clone(),
            generation: self.generation + 1,
        }
    }
}

pub type Documentation = Option<String>;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TypeIndex {
    ComplexType(Ref<ComplexTypeDefinition>),
    SimpleType(Ref<SimpleTypeDefinition>),
    ElementDecl(Ref<ElementDeclaration>),
    Wildcard(Ref<Wildcard>),
    Particle(Ref<Particle>),
    ParticleTerm(Ref<Particle>),
    ModelGroup(Ref<ModelGroup>),
    AttributeDecl(Ref<AttributeDeclaration>),
    AttributeUse(Ref<AttributeUse>),
    Generated(u32),
}

impl TypeIndex {
    pub fn sort_key(&self) -> (u8, u32) {
        match self {
            TypeIndex::ComplexType(r) => (0, r.inner().get()),
            TypeIndex::SimpleType(r) => (1, r.inner().get()),
            TypeIndex::ElementDecl(r) => (2, r.inner().get()),
            TypeIndex::Wildcard(r) => (3, r.inner().get()),
            TypeIndex::Particle(r) => (4, r.inner().get()),
            TypeIndex::ParticleTerm(r) => (5, r.inner().get()),
            TypeIndex::ModelGroup(r) => (6, r.inner().get()),
            TypeIndex::AttributeDecl(r) => (7, r.inner().get()),
            TypeIndex::AttributeUse(r) => (8, r.inner().get()),
            TypeIndex::Generated(idx) => (9, *idx),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ExternalKind {
    TypeDefinition,
    AttributeDeclaration,
    ElementDeclaration,
    AttributeGroupDefinition,
    ModelGroupDefinition,
    NotationDeclaration,
    IdentityConstraintDefinition,
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Builtin(TypeDefinition),
    Internal(TypeIndex),
    External(QName, ExternalKind),
}

impl TypeRef {
    pub fn is_builtin(&self) -> bool {
        matches!(self, TypeRef::Builtin(_))
    }

    pub fn is_internal(&self) -> bool {
        matches!(self, TypeRef::Internal(_))
    }

    pub fn is_external(&self) -> bool {
        matches!(self, TypeRef::External(_, _))
    }

    pub fn as_builtin(&self) -> Option<TypeDefinition> {
        match self {
            TypeRef::Builtin(typ) => Some(*typ),
            _ => None,
        }
    }

    pub fn as_internal(&self) -> Option<TypeIndex> {
        match self {
            TypeRef::Internal(iref) => Some(*iref),
            _ => None,
        }
    }

    pub fn as_external(&self) -> Option<(&QName, ExternalKind)> {
        match self {
            TypeRef::External(qname, kind) => Some((qname, *kind)),
            _ => None,
        }
    }

    pub fn wants_inlining(&self, m: &HashMap<TypeIndex, TypeBinding>) -> bool {
        match self {
            TypeRef::Internal(iref) => {
                if let Some(binding) = m.get(iref) {
                    binding.inline
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct StructureType {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Option<Name>,
    pub type_: TypeRef,
    pub source: FieldSource,
    pub documentation: Documentation,

    /// The field's quantifier. Currently, it must be set to `exactly_one()` in the XSD visitor.
    ///
    /// For targets that support optional fields, the inlining pass can inline a QuantifiedType's
    /// quantifier into this field.
    ///
    /// This behaviour might flip in the future, so that targets that *don't* support quantified
    /// fields must "outline" the quantifier into a separate type.
    pub quant: Quant,
}

impl Field {
    /// Same as [`FieldBuilder::new`].
    pub fn builder(type_: TypeRef) -> FieldBuilder {
        FieldBuilder::new(type_)
    }
}

pub struct FieldBuilder {
    type_: TypeRef,

    name: Option<Name>,
    source: Option<FieldSource>,
    documentation: Option<Documentation>,
    quant: Option<Quant>,
}

impl FieldBuilder {
    pub fn new(type_: TypeRef) -> Self {
        Self {
            type_,
            name: None,
            source: None,
            documentation: None,
            quant: None,
        }
    }

    pub fn build(self) -> Field {
        Field {
            name: self.name,
            type_: self.type_,
            source: self.source.unwrap_or(FieldSource::OpenContent),
            documentation: self.documentation.unwrap_or(None),
            quant: self.quant.unwrap_or(Quant::exactly_one()),
        }
    }

    pub fn name(mut self, name: Name) -> Self {
        self.name = Some(name);
        self
    }

    pub fn source(mut self, source: FieldSource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn documentation(mut self, doc: Documentation) -> Self {
        self.documentation = Some(doc);
        self
    }

    pub fn quant(mut self, quant: Quant) -> Self {
        self.quant = Some(quant);
        self
    }
}

#[derive(Debug, Clone)]
pub enum FieldSource {
    OpenContent,
    SimpleContent,
    ComplexContent,
    Term,
    SimpleTypeItem,
    ModelGroupParticle(usize),
    AttributeUse,
    Inlined(Box<FieldSource>),
    Undefined,
    // Element(Ref<ElementDeclaration>),
    // Attribute(Ref<AttributeUse>),
}

impl FieldSource {
    pub fn inlined(self) -> Self {
        FieldSource::Inlined(Box::new(self))
    }
}

#[derive(Debug)]
pub struct EnumType {
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: Name,
    pub documentation: Documentation,
}

#[derive(Debug)]
pub struct UnionType {
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug)]
pub struct UnionVariant {
    pub name: Option<Name>,
    pub type_: TypeRef,
    pub source: UnionVariantSource,
    pub documentation: Documentation,
}

#[derive(Debug)]
pub enum UnionVariantSource {
    ModelGroupParticle(usize),
    SimpleTypeMember(usize),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Quant {
    pub min: u64,
    pub max: MaxOccurs,
}

impl Default for Quant {
    fn default() -> Self {
        Self::exactly_one()
    }
}

impl Quant {
    pub const fn new(min: u64, max: MaxOccurs) -> Self {
        Self { min, max }
    }

    pub const fn exactly_one() -> Self {
        Self {
            min: 1,
            max: MaxOccurs::Count(1),
        }
    }

    pub const fn zero_or_one() -> Self {
        Self {
            min: 0,
            max: MaxOccurs::Count(1),
        }
    }

    pub const fn zero_or_more() -> Self {
        Self {
            min: 0,
            max: MaxOccurs::Unbounded,
        }
    }

    pub const fn into_min_max(self) -> (u64, MaxOccurs) {
        (self.min, self.max)
    }
}

#[derive(Debug)]
pub struct QuantifiedType {
    pub type_: TypeRef,
    pub quant: Quant,
}

impl QuantifiedType {
    pub const fn new(type_: TypeRef, quant: Quant) -> Self {
        Self { type_, quant }
    }
}

// #[derive(Debug)]
// pub enum QuantifiedSource {
//     Undefined,
//     SimpleTypeItem,
//     Particle(Ref<Particle>),
//     ElementDecl(Ref<ElementDeclaration>),
// }

#[derive(Debug)]
pub enum Type {
    Structure(StructureType),
    Enum(EnumType),
    Union(UnionType),
    Quantified(QuantifiedType),
    // NewType(TypeRef),
}

impl From<StructureType> for Type {
    fn from(s: StructureType) -> Self {
        Type::Structure(s)
    }
}

impl From<EnumType> for Type {
    fn from(e: EnumType) -> Self {
        Type::Enum(e)
    }
}

impl From<UnionType> for Type {
    fn from(u: UnionType) -> Self {
        Type::Union(u)
    }
}

impl From<QuantifiedType> for Type {
    fn from(q: QuantifiedType) -> Self {
        Type::Quantified(q)
    }
}

impl Type {
    pub const fn create_newtype(type_: TypeRef) -> Self {
        Type::Quantified(QuantifiedType {
            type_,
            quant: Quant::exactly_one(),
        })
    }

    pub fn as_structure(&self) -> Option<&StructureType> {
        match self {
            Type::Structure(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumType> {
        match self {
            Type::Enum(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_union(&self) -> Option<&UnionType> {
        match self {
            Type::Union(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_quantified(&self) -> Option<&QuantifiedType> {
        match self {
            Type::Quantified(q) => Some(q),
            _ => None,
        }
    }

    pub fn as_structure_mut(&mut self) -> Option<&mut StructureType> {
        match self {
            Type::Structure(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_enum_mut(&mut self) -> Option<&mut EnumType> {
        match self {
            Type::Enum(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_union_mut(&mut self) -> Option<&mut UnionType> {
        match self {
            Type::Union(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_quantified_mut(&mut self) -> Option<&mut QuantifiedType> {
        match self {
            Type::Quantified(q) => Some(q),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// Always externally visible (exported). Can be used by other modules.
    Public,

    /// An intermediate item that may be externally visible (depending on the target), but can be
    /// discarded if not used.
    Intermediate,

    /// The item is not externally visible (not exported), but it can be used by other items in the
    /// same module. It can not be downgraded to `Discard`.
    // PrivateHelper,

    /// The item is not externally visible (not exported), and will not be emitted.
    Discard,
}

impl Visibility {
    /// Returns true if the item must be kept, even if it is not used. This is the case for
    /// `Public` items and (in the future, maybe) private helper types.
    pub fn must_be_kept(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    pub fn is_discard(&self) -> bool {
        *self == Visibility::Discard
    }
}

#[derive(Debug)]
pub struct TypeBinding {
    pub xml_name: Option<QName>,
    pub name: Option<Name>,
    pub type_: Type,
    // pub global: bool,
    pub documentation: Documentation,

    pub owner: Option<TypeIndex>,
    pub inline: bool,
    pub visibility: Visibility,
}
