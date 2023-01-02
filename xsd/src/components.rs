use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::{NonZeroU32, NonZeroUsize};

use crate::BuiltinOverwriteAction;

use roxmltree::Node;

use super::xstypes::QName;
use super::{
    builtins, Annotation, Assertion, AttributeDeclaration, AttributeGroupDefinition, AttributeUse,
    ComplexTypeDefinition, ConstrainingFacet, ElementDeclaration, IdentityConstraintDefinition,
    ModelGroup, ModelGroupDefinition, NotationDeclaration, Particle, SimpleTypeDefinition,
    TypeAlternative, TypeDefinition, Wildcard,
};

/// Trait implemented by all concrete schema components.
pub trait Component {
    const DISPLAY_NAME: &'static str;
}

/// Type on which internal component traits are implemented.
///
/// This type is used to prevent leaking internal functions into the [`Component`]
pub struct ComponentTraits;

/// A component referencable via [`Ref`]. Intended for internal use.
pub trait HasArenaContainer<R: Component>: Sized {
    fn get_container_from_construction_component_table(
        table: &ConstructionComponentTable,
    ) -> &[Option<R>];
    fn get_container_from_construction_component_table_mut(
        table: &mut ConstructionComponentTable,
    ) -> &mut Vec<Option<R>>;
    fn get_container_from_schema_component_table(table: &SchemaComponentTable) -> &[R];
}

/// A reference to a [`Component`] stored in a [`ComponentTable`]
pub struct Ref<R>(NonZeroU32, PhantomData<R>)
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>;

impl<R> Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
    const fn from_inner(inner: NonZeroU32) -> Self {
        Self(inner, PhantomData)
    }

    const fn inner(self) -> NonZeroU32 {
        self.0
    }

    fn index(self) -> usize {
        let size: NonZeroUsize = self
            .0
            .try_into()
            .expect("Could not convert component reference to usize index");
        usize::from(size) - 1
    }

    pub fn get(self, table: &impl ComponentTable) -> &R {
        table.get(self)
    }
}

// derive(...) does not work if T itself does not derive the trait, even though it is only "used"
// in the PhantomData; hence we have to manually implement required traits for the Ref type.

impl<R> Copy for Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
}

impl<R> Clone for Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<R> fmt::Debug for Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} #{}>", R::DISPLAY_NAME, self.0)
    }
}

impl<R> PartialEq for Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<R> Eq for Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
}

impl<R> Hash for Ref<R>
where
    R: Component,
    ComponentTraits: HasArenaContainer<R>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A reference to a component whose type is not known at compile time.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct DynamicRef(TypeId, NonZeroU32);

impl<R> From<Ref<R>> for DynamicRef
where
    R: Component + 'static,
    ComponentTraits: HasArenaContainer<R>,
{
    fn from(ref_: Ref<R>) -> Self {
        Self(TypeId::of::<R>(), ref_.inner())
    }
}

/// An arena-like container for various [`Component`]s
pub trait ComponentTable {
    /// Retrieves a component's value by reference from this component table.
    /// This function panics if the component value is not present in the table.
    fn get<R>(&self, ref_: Ref<R>) -> &R
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>;
}

/// The [component table](ComponentTable) implementation that is used during schema parsing and
/// construction.
///
/// The individual container `Vec`s contain the components wrapped in `Option`s, since components
/// often need to reference themselves, and thus are constructed after the `Ref` itself.
#[derive(Default)]
// TODO restrict to pub(super)
pub struct ConstructionComponentTable {
    annotations: Vec<Option<Annotation>>,
    assertions: Vec<Option<Assertion>>,
    attribute_declarations: Vec<Option<AttributeDeclaration>>,
    attribute_group_definitions: Vec<Option<AttributeGroupDefinition>>,
    attribute_uses: Vec<Option<AttributeUse>>,
    complex_type_definitions: Vec<Option<ComplexTypeDefinition>>,
    constraining_facets: Vec<Option<ConstrainingFacet>>,
    element_declarations: Vec<Option<ElementDeclaration>>,
    identity_constraint_definitions: Vec<Option<IdentityConstraintDefinition>>,
    model_group_definitions: Vec<Option<ModelGroupDefinition>>,
    model_groups: Vec<Option<ModelGroup>>,
    notation_declarations: Vec<Option<NotationDeclaration>>,
    particles: Vec<Option<Particle>>,
    simple_type_definitions: Vec<Option<SimpleTypeDefinition>>,
    type_alternatives: Vec<Option<TypeAlternative>>,
    wildcards: Vec<Option<Wildcard>>,
}

impl ComponentTable for ConstructionComponentTable {
    fn get<R>(&self, ref_: Ref<R>) -> &R
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>,
    {
        let container = ComponentTraits::get_container_from_construction_component_table(self);
        container
            .get(ref_.index())
            .expect("Invalid component reference (out-of-bounds)")
            .as_ref()
            .expect("Component is not present")
    }
}

impl ConstructionComponentTable {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Creates a [`Ref`] which points to an absent, reserved slot in the table.
    pub(super) fn reserve<R>(&mut self) -> Ref<R>
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>,
    {
        let container = ComponentTraits::get_container_from_construction_component_table_mut(self);

        // Reserve a slot by inserting None
        container.push(None);

        // We use the size for the ref's ID, which is non-zero after the push
        let size = NonZeroUsize::new(container.len()).unwrap();
        let id: NonZeroU32 = size.try_into().expect("ID did not fit into 32-bit integer");

        Ref::from_inner(id)
    }

    /// Inserts the `value` into the slot pointed to by `ref_`. Returns `ref_` for convenience.
    pub(super) fn insert<R>(&mut self, ref_: Ref<R>, value: R) -> Ref<R>
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>,
    {
        let container = ComponentTraits::get_container_from_construction_component_table_mut(self);

        let slot = container
            .get_mut(ref_.index())
            .expect("Invalid component reference (out-of-bounds)");

        *slot = Some(value);

        ref_
    }

    /// Shorthand for `insert(reserve(), value)`
    pub(super) fn create<R>(&mut self, value: R) -> Ref<R>
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>,
    {
        let ref_ = self.reserve();
        self.insert(ref_, value)
    }

    pub(super) fn is_present<R>(&self, ref_: Ref<R>) -> bool
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>,
    {
        let container = ComponentTraits::get_container_from_construction_component_table(self);

        let slot = container
            .get(ref_.index())
            .expect("Invalid component reference (out-of-bounds)");

        slot.is_some()
    }

    /// Tries to convert this construction table to a [schema table](`SchemaComponentTable`).
    /// If a component value is absent, `None` is returned instead.
    pub(super) fn convert_to_schema_table(self) -> Option<SchemaComponentTable> {
        Some(SchemaComponentTable {
            annotations: Self::convert_container(self.annotations)?,
            assertions: Self::convert_container(self.assertions)?,
            attribute_declarations: Self::convert_container(self.attribute_declarations)?,
            attribute_group_definitions: Self::convert_container(self.attribute_group_definitions)?,
            attribute_uses: Self::convert_container(self.attribute_uses)?,
            complex_type_definitions: Self::convert_container(self.complex_type_definitions)?,
            constraining_facets: Self::convert_container(self.constraining_facets)?,
            element_declarations: Self::convert_container(self.element_declarations)?,
            identity_constraint_definitions: Self::convert_container(
                self.identity_constraint_definitions,
            )?,
            model_group_definitions: Self::convert_container(self.model_group_definitions)?,
            model_groups: Self::convert_container(self.model_groups)?,
            notation_declarations: Self::convert_container(self.notation_declarations)?,
            particles: Self::convert_container(self.particles)?,
            simple_type_definitions: Self::convert_container(self.simple_type_definitions)?,
            type_alternatives: Self::convert_container(self.type_alternatives)?,
            wildcards: Self::convert_container(self.wildcards)?,
        })
    }

    /// Helper for [`Self::convert_to_schema_table()`]
    fn convert_container<R>(container: Vec<Option<R>>) -> Option<Box<[R]>> {
        let mut result = Vec::<R>::with_capacity(container.len());
        for component in container {
            result.push(component?);
        }
        Some(result.into_boxed_slice())
    }
}

/// The [component table](ComponentTable) implementation that is used alongside the final schema.
///
/// Components for which a [`Ref`] exists will always be present in this table.
///
/// Since this table is meant to be read-only, the components are stored in boxed slices, which
/// reduces the struct's size by one pointer per component type compared to the `Vec`-storage used
/// in the [`ConstructionComponentTable`].
pub struct SchemaComponentTable {
    annotations: Box<[Annotation]>,
    assertions: Box<[Assertion]>,
    attribute_declarations: Box<[AttributeDeclaration]>,
    attribute_group_definitions: Box<[AttributeGroupDefinition]>,
    attribute_uses: Box<[AttributeUse]>,
    complex_type_definitions: Box<[ComplexTypeDefinition]>,
    constraining_facets: Box<[ConstrainingFacet]>,
    element_declarations: Box<[ElementDeclaration]>,
    identity_constraint_definitions: Box<[IdentityConstraintDefinition]>,
    model_group_definitions: Box<[ModelGroupDefinition]>,
    model_groups: Box<[ModelGroup]>,
    notation_declarations: Box<[NotationDeclaration]>,
    particles: Box<[Particle]>,
    simple_type_definitions: Box<[SimpleTypeDefinition]>,
    type_alternatives: Box<[TypeAlternative]>,
    wildcards: Box<[Wildcard]>,
}

impl ComponentTable for SchemaComponentTable {
    fn get<R>(&self, ref_: Ref<R>) -> &R
    where
        R: Component,
        ComponentTraits: HasArenaContainer<R>,
    {
        let container = ComponentTraits::get_container_from_schema_component_table(self);
        container
            .get(ref_.index())
            .expect("Invalid component reference (out-of-bounds)")
    }
}

macro_rules! has_arena_container_impl {
    ($type_name:ty, $field_name:ident) => {
        impl HasArenaContainer<$type_name> for ComponentTraits {
            fn get_container_from_construction_component_table(
                table: &ConstructionComponentTable,
            ) -> &[Option<$type_name>] {
                &table.$field_name
            }

            fn get_container_from_construction_component_table_mut(
                table: &mut ConstructionComponentTable,
            ) -> &mut Vec<Option<$type_name>> {
                &mut table.$field_name
            }

            fn get_container_from_schema_component_table(
                table: &SchemaComponentTable,
            ) -> &[$type_name] {
                &table.$field_name
            }
        }
    };
}

has_arena_container_impl!(Annotation, annotations);
has_arena_container_impl!(Assertion, assertions);
has_arena_container_impl!(AttributeDeclaration, attribute_declarations);
has_arena_container_impl!(AttributeGroupDefinition, attribute_group_definitions);
has_arena_container_impl!(AttributeUse, attribute_uses);
has_arena_container_impl!(ComplexTypeDefinition, complex_type_definitions);
has_arena_container_impl!(ConstrainingFacet, constraining_facets);
has_arena_container_impl!(ElementDeclaration, element_declarations);
has_arena_container_impl!(
    IdentityConstraintDefinition,
    identity_constraint_definitions
);
has_arena_container_impl!(ModelGroupDefinition, model_group_definitions);
has_arena_container_impl!(ModelGroup, model_groups);
has_arena_container_impl!(NotationDeclaration, notation_declarations);
has_arena_container_impl!(Particle, particles);
has_arena_container_impl!(SimpleTypeDefinition, simple_type_definitions);
has_arena_container_impl!(TypeAlternative, type_alternatives);
has_arena_container_impl!(Wildcard, wildcards);

/// A component that may have a [qualified name](QName)
pub trait Named: Component {
    /// The optional name.
    /// Some components (like [`ElementDeclaration`]) always have a name, and always return `Some`.
    fn name(&self) -> Option<QName>;
}

/// Any type that indirectly implements [`Named`], i.e. where first a [`Ref`] has to be dereferenced
/// to get to the name.
/// This is trivially the case for all `Ref<Named>`, and additionally for [`TypeDefinition`].
pub trait RefNamed {
    fn name(&self, table: &impl ComponentTable) -> Option<QName>;
}

impl<R> RefNamed for Ref<R>
where
    R: Named,
    ComponentTraits: HasArenaContainer<R>,
{
    fn name(&self, table: &impl ComponentTable) -> Option<QName> {
        self.get(table).name()
    }
}

/// Trait that allows components to be looked up by their [qualified name](QName).
/// `V` is the value type (usually `Ref<Component>` or a wrapper like [`TypeDefinition`]).
pub(super) trait Lookup<V: Copy> {
    /// Registers a value for lookup in its respective symbol space.
    /// Returns `true` if the name given by the `key` parameter was already associated with a value.
    fn register_value_for_lookup(&mut self, key: QName, value: V) -> bool;

    /// Looks up the value associated with the `key`; returns `None` if there is no such value.
    fn lookup_value(&self, key: &QName) -> Option<V>;
}

type LookupTable<T> = HashMap<QName, T>;

#[derive(Default)]
pub(super) struct LookupTables {
    /// Shared symbol space for simple and complex type definitions
    type_definitions: LookupTable<TypeDefinition>,
    attribute_declarations: LookupTable<Ref<AttributeDeclaration>>,
    element_declarations: LookupTable<Ref<ElementDeclaration>>,
    attribute_group_definitions: LookupTable<Ref<AttributeGroupDefinition>>,
    model_group_definitions: LookupTable<Ref<ModelGroupDefinition>>,
    notation_declarations: LookupTable<Ref<NotationDeclaration>>,
    identity_constraint_definitions: LookupTable<Ref<IdentityConstraintDefinition>>,
}

macro_rules! impl_lookup {
    ($field_name:ident: $value_type:ty) => {
        impl Lookup<$value_type> for LookupTables {
            fn register_value_for_lookup(&mut self, key: QName, value: $value_type) -> bool {
                self.$field_name.insert(key, value).is_some()
            }

            fn lookup_value(&self, key: &QName) -> Option<$value_type> {
                self.$field_name.get(key).copied()
            }
        }
    };
}

impl_lookup!(type_definitions: TypeDefinition);
impl_lookup!(attribute_declarations: Ref<AttributeDeclaration>);
impl_lookup!(element_declarations: Ref<ElementDeclaration>);
impl_lookup!(attribute_group_definitions: Ref<AttributeGroupDefinition>);
impl_lookup!(model_group_definitions: Ref<ModelGroupDefinition>);
impl_lookup!(notation_declarations: Ref<NotationDeclaration>);
impl_lookup!(identity_constraint_definitions: Ref<IdentityConstraintDefinition>);

impl Lookup<Ref<SimpleTypeDefinition>> for LookupTables {
    fn register_value_for_lookup(&mut self, key: QName, value: Ref<SimpleTypeDefinition>) -> bool {
        self.type_definitions
            .insert(key, TypeDefinition::Simple(value))
            .is_some()
    }

    fn lookup_value(&self, key: &QName) -> Option<Ref<SimpleTypeDefinition>> {
        self.type_definitions
            .get(key)
            .and_then(|type_def| type_def.simple())
    }
}

impl Lookup<Ref<ComplexTypeDefinition>> for LookupTables {
    fn register_value_for_lookup(&mut self, key: QName, value: Ref<ComplexTypeDefinition>) -> bool {
        self.type_definitions
            .insert(key, TypeDefinition::Complex(value))
            .is_some()
    }

    fn lookup_value(&self, key: &QName) -> Option<Ref<ComplexTypeDefinition>> {
        self.type_definitions
            .get(key)
            .and_then(|type_def| type_def.complex())
    }
}

/// QName resolution according to §3.17.6.2
pub(super) struct ComponentResolver {
    lookup_tables: LookupTables,
    builtin_overwrite: BuiltinOverwriteAction,
}

impl ComponentResolver {
    pub(super) fn new(builtin_overwrite: BuiltinOverwriteAction) -> Self {
        Self {
            lookup_tables: LookupTables::default(),
            builtin_overwrite,
        }
    }

    pub(super) fn resolve<R>(&self, key: &QName) -> R
    where
        R: Copy,
        LookupTables: Lookup<R>,
    {
        // TODO error handling
        self.lookup_tables.lookup_value(key).unwrap()
    }

    pub(super) fn register_with_name<R>(&mut self, name: QName, value: R)
    where
        R: Copy,
        LookupTables: Lookup<R>,
    {
        // TODO don't clone name
        let prev = self
            .lookup_tables
            .register_value_for_lookup(name.clone(), value);
        if prev {
            if builtins::is_builtin_name(&name) {
                match self.builtin_overwrite {
                    BuiltinOverwriteAction::Deny => {
                        panic!("Tried to overwrite built-in component: {name}");
                    }
                    BuiltinOverwriteAction::Warn => {
                        eprintln!("WARN: Overwriting built-in component {name}");
                    }
                    BuiltinOverwriteAction::Allow => {}
                }
            } else {
                panic!(
                    "Duplicate component: {name} ({})",
                    std::any::type_name::<R>()
                ); // TODO propagate
            }
        }
    }

    pub(super) fn register<R>(&mut self, value: R, table: &impl ComponentTable)
    where
        R: RefNamed + Copy,
        LookupTables: Lookup<R>,
    {
        let name = value
            .name(table)
            .expect("Tried to register unnamed component");
        self.register_with_name(name, value)
    }
}

/// (Top-level) Components whose name is always available from XML
pub(super) trait NamedXml: Component {
    fn get_name_from_xml(this_node: Node, schema_node: Node) -> QName;
}
