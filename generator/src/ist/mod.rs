pub mod builder;
pub mod passes;
pub mod xsd_visitor;

use std::{collections::HashMap, iter::FusedIterator};

use dt_xsd::{
    AttributeDeclaration, AttributeUse, ComplexTypeDefinition, ElementDeclaration, ModelGroup,
    Particle, Ref, SimpleTypeDefinition, TypeDefinition, Wildcard, particle::MaxOccurs,
    xstypes::QName,
};

#[derive(Debug, Clone)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: String) -> Self {
        Self { name }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Compositor {
    Structure,
    Union,
}

#[derive(Debug, Clone)]
pub struct CompositeType {
    pub compositor: Compositor,
    pub members: Vec<Member>,
}

impl CompositeType {
    /// Returns true if the structure can be "newtyped" (i.e. reduced
    /// to a type alias or similar target construct).
    /// This implies that the structure has exactly one field.
    pub fn is_thin(&self) -> bool {
        self.members.len() == 1
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: Option<Name>,
    pub type_: TypeRef,
    pub source: FieldSource,
    pub documentation: Documentation,

    /// The field's quantifier.
    ///
    /// Targets that *don't* support quantified fields must "outline" the quantifier into a
    /// separate type.
    pub quant: Quant,
}

impl Member {
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

    pub fn build(self) -> Member {
        Member {
            name: self.name,
            type_: self.type_,
            source: self.source.unwrap_or(FieldSource::Undefined),
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
    SimpleTypeMember(usize),
}

impl FieldSource {
    pub fn inlined(self) -> Self {
        FieldSource::Inlined(Box::new(self))
    }
}

#[derive(Debug, Clone)]
pub struct EnumType {
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Name,
    pub documentation: Documentation,
}

#[derive(Debug, Clone)]
pub struct UnionType {
    pub variants: Vec<Member>,
}

#[derive(Debug, Clone)]
pub struct UnionVariant {
    pub name: Option<Name>,
    pub type_: TypeRef,
    pub source: UnionVariantSource,
    pub documentation: Documentation,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Type {
    Composite(CompositeType),
    Enum(EnumType),
}

impl From<CompositeType> for Type {
    fn from(s: CompositeType) -> Self {
        Type::Composite(s)
    }
}

impl From<EnumType> for Type {
    fn from(e: EnumType) -> Self {
        Type::Enum(e)
    }
}

impl Type {
    pub fn create_quantified(type_: TypeRef, quant: Quant) -> Self {
        Self::Composite(CompositeType {
            compositor: Compositor::Structure,
            members: vec![Member::builder(type_).quant(quant).build()],
        })
    }

    pub fn create_newtype(type_: TypeRef) -> Self {
        Self::create_quantified(type_, Quant::exactly_one())
    }

    pub fn create_structure(members: Vec<Member>) -> Self {
        Self::Composite(CompositeType {
            compositor: Compositor::Structure,
            members,
        })
    }

    pub fn create_union(members: Vec<Member>) -> Self {
        Self::Composite(CompositeType {
            compositor: Compositor::Union,
            members,
        })
    }

    pub fn as_structure(&self) -> Option<&CompositeType> {
        match self {
            Type::Composite(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumType> {
        match self {
            Type::Enum(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_structure_mut(&mut self) -> Option<&mut CompositeType> {
        match self {
            Type::Composite(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_enum_mut(&mut self) -> Option<&mut EnumType> {
        match self {
            Type::Enum(e) => Some(e),
            _ => None,
        }
    }

    pub fn children(&self) -> Children {
        match self {
            Type::Composite(s) => Children::Composite(s.members.iter().map(|f| &f.type_)),
            Type::Enum(_e) => Children::Enum(std::iter::empty()),
        }
    }
}

type CompositeChildren<'a> =
    std::iter::Map<std::slice::Iter<'a, Member>, fn(&'a Member) -> &'a TypeRef>;

type EnumChildren<'a> = std::iter::Empty<&'a TypeRef>;
//     std::iter::Map<std::slice::Iter<'static, EnumVariant>, fn(&'static EnumVariant) -> TypeRef>;

/// Iterator over the children of a [`Type`].
pub enum Children<'a> {
    Composite(CompositeChildren<'a>),
    Enum(EnumChildren<'a>),
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a TypeRef;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Children::Composite(iter) => iter.next(),
            Children::Enum(iter) => iter.next(),
        }
    }
}

impl FusedIterator for Children<'_> {}

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

impl TypeBinding {
    pub fn should_emit(&self) -> bool {
        self.visibility != Visibility::Discard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_iter_structure() {
        let structure = Type::create_structure(vec![
            Member::builder(TypeRef::External(
                QName::without_namespace("type1"),
                ExternalKind::TypeDefinition,
            ))
            .name(Name::new("field1".to_string()))
            .build(),
            Member::builder(TypeRef::External(
                QName::without_namespace("type2"),
                ExternalKind::TypeDefinition,
            ))
            .name(Name::new("field2".to_string()))
            .build(),
        ]);

        let mut children = structure.children();

        let type1 = children.next().expect("Expected first child");
        let type1 = type1
            .as_external()
            .expect("Expected first child to be external type");
        assert_eq!(type1.0, &QName::without_namespace("type1"));
        assert_eq!(type1.1, ExternalKind::TypeDefinition);

        let type2 = children.next().expect("Expected second child");
        let type2 = type2
            .as_external()
            .expect("Expected second child to be external type");
        assert_eq!(type2.0, &QName::without_namespace("type2"));
        assert_eq!(type2.1, ExternalKind::TypeDefinition);

        assert!(children.next().is_none(), "Expected no more children");
    }

    #[test]
    fn children_iter_union() {
        let union = Type::create_union(vec![
            Member {
                name: Some(Name::new("variant1".to_string())),
                type_: TypeRef::External(
                    QName::without_namespace("type1"),
                    ExternalKind::TypeDefinition,
                ),
                source: FieldSource::ModelGroupParticle(0),
                documentation: None,
                quant: Quant::default(),
            },
            Member {
                name: Some(Name::new("variant2".to_string())),
                type_: TypeRef::External(
                    QName::without_namespace("type2"),
                    ExternalKind::TypeDefinition,
                ),
                source: FieldSource::SimpleTypeMember(0),
                documentation: None,
                quant: Quant::default(),
            },
        ]);

        let mut children = union.children();

        let variant1 = children.next().expect("Expected first child");
        let variant1 = variant1
            .as_external()
            .expect("Expected first child to be external type");
        assert_eq!(variant1.0, &QName::without_namespace("type1"));
        assert_eq!(variant1.1, ExternalKind::TypeDefinition);

        let variant2 = children.next().expect("Expected second child");
        let variant2 = variant2
            .as_external()
            .expect("Expected second child to be external type");
        assert_eq!(variant2.0, &QName::without_namespace("type2"));
        assert_eq!(variant2.1, ExternalKind::TypeDefinition);

        assert!(children.next().is_none(), "Expected no more children");
    }
}
