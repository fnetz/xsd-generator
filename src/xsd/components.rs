use super::*;

macro_rules! compcontainer {
    ($($name:ident: $typ:ty,)+) => {
        $(
            impl Refable for $typ {
                fn intermediate_container(container: &IntermediateComponentContainer) -> &[Intermediate<Self>] {
                    &container.$name
                }
                fn intermediate_container_mut(container: &mut IntermediateComponentContainer) -> &mut Vec<Intermediate<Self>> {
                    &mut container.$name
                }

                fn container(container: &SchemaComponentContainer) -> &[Self] {
                    &container.$name
                }

                fn visit_ref<V: ConcreteRefVisitor + ?Sized>(ref_: &mut Ref<Self>, visitor: &mut V) {
                    // TODO better name
                    visitor.$name(ref_)
                }
            }
        )+
    };
}

type Intermediate<T> = Option<T>;

// TODO rename to Tables... something
#[derive(Default)]
pub struct IntermediateComponentContainer {
    attribute_declarations: Vec<Intermediate<AttributeDeclaration>>,
    element_declarations: Vec<Intermediate<ElementDeclaration>>,
    complex_type_definitions: Vec<Intermediate<ComplexTypeDefinition>>,
    attribute_uses: Vec<Intermediate<AttributeUse>>,
    attribute_group_definitions: Vec<Intermediate<AttributeGroupDefinition>>,
    model_group_definitions: Vec<Intermediate<ModelGroupDefinition>>,
    model_groups: Vec<Intermediate<ModelGroup>>,
    particles: Vec<Intermediate<Particle>>,
    wildcards: Vec<Intermediate<Wildcard>>,
    identity_constraint_definitions: Vec<Intermediate<IdentityConstraintDefinition>>,
    type_alternatives: Vec<Intermediate<TypeAlternative>>,
    assertions: Vec<Intermediate<Assertion>>,
    notation_declarations: Vec<Intermediate<NotationDeclaration>>,
    annotations: Vec<Intermediate<Annotation>>,
    simple_type_definitions: Vec<Intermediate<SimpleTypeDefinition>>,

    type_definitions: Vec<Intermediate<TypeDefinition>>,
    constraining_facets: Vec<Intermediate<ConstrainingFacet>>,

    // TODO merge?
    unresolved_type_definitions: Vec<QName>,
    unresolved_simple_type_definitions: Vec<QName>,
    unresolved_complex_type_definitions: Vec<QName>,

    // Simple and complex type definitions share a symbol space
    type_definition_lookup: HashMap<QName, RRef<TypeDefinition>>,
}

#[derive(Debug)]
pub struct SchemaComponentContainer {
    attribute_declarations: Vec<AttributeDeclaration>,
    element_declarations: Vec<ElementDeclaration>,
    complex_type_definitions: Vec<ComplexTypeDefinition>,
    attribute_uses: Vec<AttributeUse>,
    attribute_group_definitions: Vec<AttributeGroupDefinition>,
    model_group_definitions: Vec<ModelGroupDefinition>,
    model_groups: Vec<ModelGroup>,
    particles: Vec<Particle>,
    wildcards: Vec<Wildcard>,
    identity_constraint_definitions: Vec<IdentityConstraintDefinition>,
    type_alternatives: Vec<TypeAlternative>,
    assertions: Vec<Assertion>,
    notation_declarations: Vec<NotationDeclaration>,
    annotations: Vec<Annotation>,
    simple_type_definitions: Vec<SimpleTypeDefinition>,

    type_definitions: Vec<TypeDefinition>, // TODO
    constraining_facets: Vec<ConstrainingFacet>,
}

/// In case of built-in resolutions, `Immediate` can be used to indicate that the
/// resolution can be done directly (e.g. for built-in components). Otherwise, the resolution is
/// deferred until the Ref resolution pass.
#[derive(Copy, Clone)]
pub enum Resolution {
    Immediate,
    Deferred,
}

impl IntermediateComponentContainer {
    pub fn reserve<T: Refable>(&mut self) -> Ref<T> {
        let index = T::intermediate_container(self).len();
        T::intermediate_container_mut(self).push(None);
        Ref::new(index as u32)
    }

    pub fn populate<T: Refable>(&mut self, ref_: Ref<T>, value: T) {
        T::intermediate_container_mut(self)[ref_.index()] = Some(value);
    }

    pub fn create<T: Refable>(&mut self, value: T) -> Ref<T> {
        let ref_ = self.reserve::<T>();
        self.populate(ref_, value);
        ref_
    }

    pub fn is_populated<T: Refable>(&self, ref_: Ref<T>) -> bool {
        T::intermediate_container(self)[ref_.index()].is_some()
    }

    pub fn resolve_simple_type_def(
        &mut self,
        name: &QName,
        when: Resolution,
    ) -> Ref<SimpleTypeDefinition> {
        // FIXME namespace compare
        // FIXME option?
        match when {
            Resolution::Immediate => todo!("immediate simple type resolution"),
            Resolution::Deferred => {
                let id = self.unresolved_simple_type_definitions.len() as u32;
                self.unresolved_simple_type_definitions.push(name.clone());
                Ref::new_unresolved(id)
            }
        }
    }

    // TODO create typedef along with simple/complex typedef

    pub fn resolve_type_def(&mut self, name: &QName, when: Resolution) -> Ref<TypeDefinition> {
        println!(
            "Resolving {:?} {}",
            name,
            match when {
                Resolution::Immediate => "immediately",
                Resolution::Deferred => "deferred",
            }
        );
        match when {
            Resolution::Immediate => {
                println!("Types currently in LUT:");
                for (key, ref_) in self.type_definition_lookup.iter() {
                    println!("{:?} -> {:?}", key, **ref_);
                }
                *self.type_definition_lookup.get(name).copied().unwrap()
            }
            Resolution::Deferred => {
                let id = self.unresolved_type_definitions.len() as u32;
                self.unresolved_type_definitions.push(name.clone());
                Ref::new_unresolved(id)
            }
        }
    }

    pub fn resolve_attribute_declaration(
        &mut self,
        _name: &QName,
        _when: Resolution,
    ) -> Ref<AttributeDeclaration> {
        todo!()
    }

    pub fn resolve_element_declaration(
        &mut self,
        _name: &QName,
        _when: Resolution,
    ) -> Ref<ElementDeclaration> {
        todo!()
    }

    pub(super) fn register_type(&mut self, type_def: Ref<TypeDefinition>) {
        let type_def = type_def
            .as_resolved()
            .expect("Tried to register unresolved type");
        let res_type_def = type_def.get_intermediate(self).unwrap();
        // TODO move to TypeDefinition
        let (namespace_name, name) = match res_type_def {
            TypeDefinition::Simple(ref_) => {
                let simple_type_def = ref_.get_intermediate(self).unwrap();
                (
                    simple_type_def.target_namespace.clone(),
                    simple_type_def
                        .name
                        .clone()
                        .expect("Can't register unnamed type"),
                )
            }
            TypeDefinition::Complex(ref_) => {
                let complex_type_def = ref_.get_intermediate(self).unwrap();
                (
                    complex_type_def.target_namespace.clone(),
                    complex_type_def
                        .name
                        .clone()
                        .expect("Can't register unnamed type"),
                )
            }
        };

        let qname = QName(namespace_name.unwrap_or_default(), name);

        self.type_definition_lookup.insert(qname, type_def);
    }

    pub fn perform_ref_resolution_pass(self) -> SchemaComponentContainer {
        // The Ref resolution pass generally can be divided into two parts:
        // 1. Resolve all unresolved components
        // 2. Visit all Refs and set them to the resolved value's Ref.
        //    As a byproduct, it is ensured that all components have been constructed.

        let mut resolved_type_definitions =
            Vec::<RRef<TypeDefinition>>::with_capacity(self.unresolved_type_definitions.len());
        let mut resolved_simple_type_definitions = Vec::<RRef<SimpleTypeDefinition>>::with_capacity(
            self.unresolved_simple_type_definitions.len(),
        );
        let mut resolved_complex_type_definitions =
            Vec::<RRef<ComplexTypeDefinition>>::with_capacity(
                self.unresolved_complex_type_definitions.len(),
            );

        // TODO resolution failure

        for name in self.unresolved_type_definitions.iter() {
            println!("Deferred resolve {:?}", name);
            println!("Types currently in LUT:");
            for (key, ref_) in self.type_definition_lookup.iter() {
                println!("{:?} -> {:?}", key, **ref_);
            }
            let resolved = *self.type_definition_lookup.get(name).unwrap();
            resolved_type_definitions.push(resolved);
        }

        // TODO unresolved inner possible?

        // TODO find solution without clone
        for i in 0..self.unresolved_simple_type_definitions.len() {
            let name = self.unresolved_simple_type_definitions[i].clone();
            println!("Deferred resolve {:?}", name);
            println!("Types currently in LUT:");
            for (key, ref_) in self.type_definition_lookup.iter() {
                println!("{:?} -> {:?}", key, **ref_);
            }
            let type_def = *self.type_definition_lookup.get(&name).unwrap();
            let type_def = type_def.get_intermediate(&self).unwrap().to_owned();
            let simple_type_def = type_def.simple().unwrap();
            let simple_type_def = simple_type_def
                .as_resolved()
                .unwrap_or_else(|| todo!("unresolved simple type def"));
            resolved_simple_type_definitions.push(simple_type_def);
        }

        for i in 0..self.unresolved_complex_type_definitions.len() {
            let name = self.unresolved_complex_type_definitions[i].clone();
            let type_def = *self.type_definition_lookup.get(&name).unwrap();
            let type_def = type_def.get_intermediate(&self).unwrap().to_owned();
            let complex_type_def = type_def.complex().unwrap();
            let complex_type_def = complex_type_def
                .as_resolved()
                .unwrap_or_else(|| todo!("unresolved complex type def"));
            resolved_complex_type_definitions.push(complex_type_def);
        }

        let mut rv = ARefVisitor {
            resolved_type_definitions,
            resolved_simple_type_definitions,
            resolved_complex_type_definitions,
        };

        // Step 2
        let mut attribute_declarations = Self::finalize_list(self.attribute_declarations);
        for component in attribute_declarations.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut element_declarations = Self::finalize_list(self.element_declarations);
        for component in element_declarations.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut complex_type_definitions = Self::finalize_list(self.complex_type_definitions);
        for component in complex_type_definitions.iter_mut() {
            component.visit_refs(&mut rv)
        }

        let mut attribute_uses = Self::finalize_list(self.attribute_uses);
        for component in attribute_uses.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut attribute_group_definitions = Self::finalize_list(self.attribute_group_definitions);
        for component in attribute_group_definitions.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut model_group_definitions = Self::finalize_list(self.model_group_definitions);
        for component in model_group_definitions.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut model_groups = Self::finalize_list(self.model_groups);
        for component in model_groups.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut particles = Self::finalize_list(self.particles);
        for component in particles.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut wildcards = Self::finalize_list(self.wildcards);
        for component in wildcards.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut identity_constraint_definitions =
            Self::finalize_list(self.identity_constraint_definitions);
        for component in identity_constraint_definitions.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut type_alternatives = Self::finalize_list(self.type_alternatives);
        for component in type_alternatives.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut assertions = Self::finalize_list(self.assertions);
        for component in assertions.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut notation_declarations = Self::finalize_list(self.notation_declarations);
        for component in notation_declarations.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut annotations = Self::finalize_list(self.annotations);
        for component in annotations.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut simple_type_definitions = Self::finalize_list(self.simple_type_definitions);
        for component in simple_type_definitions.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut type_definitions = Self::finalize_list(self.type_definitions);
        for component in type_definitions.iter_mut() {
            component.visit_refs(&mut rv);
        }

        let mut constraining_facets = Self::finalize_list(self.constraining_facets);
        for component in constraining_facets.iter_mut() {
            component.visit_refs(&mut rv);
        }

        SchemaComponentContainer {
            attribute_declarations,
            element_declarations,
            complex_type_definitions,
            attribute_uses,
            attribute_group_definitions,
            model_group_definitions,
            model_groups,
            particles,
            wildcards,
            identity_constraint_definitions,
            type_alternatives,
            assertions,
            notation_declarations,
            annotations,
            simple_type_definitions,
            type_definitions,
            constraining_facets,
        }
    }

    fn finalize_list<T>(list: Vec<Intermediate<T>>) -> Vec<T> {
        list.into_iter()
            .map(|component| component.unwrap())
            .collect()
    }
}

struct ARefVisitor {
    resolved_type_definitions: Vec<RRef<TypeDefinition>>,
    resolved_simple_type_definitions: Vec<RRef<SimpleTypeDefinition>>,
    resolved_complex_type_definitions: Vec<RRef<ComplexTypeDefinition>>,
}

impl ConcreteRefVisitor for ARefVisitor {
    fn type_definitions(&mut self, ref_: &mut Ref<TypeDefinition>) {
        *ref_ = *self.resolved_type_definitions[ref_.index()];
    }

    fn simple_type_definitions(&mut self, ref_: &mut Ref<SimpleTypeDefinition>) {
        *ref_ = *self.resolved_simple_type_definitions[ref_.index()];
    }

    fn complex_type_definitions(&mut self, ref_: &mut Ref<ComplexTypeDefinition>) {
        *ref_ = *self.resolved_complex_type_definitions[ref_.index()];
    }
}

impl RefVisitor for ARefVisitor {
    fn visit_ref<T: Refable>(&mut self, ref_: &mut Ref<T>) {
        if !ref_.is_resolved() {
            T::visit_ref(ref_, self)
        }
    }
}

pub trait ConcreteRefVisitor {
    fn attribute_declarations(&mut self, _ref_: &mut Ref<AttributeDeclaration>) {}
    fn element_declarations(&mut self, _ref_: &mut Ref<ElementDeclaration>) {}
    fn complex_type_definitions(&mut self, _ref_: &mut Ref<ComplexTypeDefinition>) {}
    fn attribute_uses(&mut self, _ref_: &mut Ref<AttributeUse>) {}
    fn attribute_group_definitions(&mut self, _ref_: &mut Ref<AttributeGroupDefinition>) {}
    fn model_group_definitions(&mut self, _ref_: &mut Ref<ModelGroupDefinition>) {}
    fn model_groups(&mut self, _ref_: &mut Ref<ModelGroup>) {}
    fn particles(&mut self, _ref_: &mut Ref<Particle>) {}
    fn wildcards(&mut self, _ref_: &mut Ref<Wildcard>) {}
    fn identity_constraint_definitions(&mut self, _ref_: &mut Ref<IdentityConstraintDefinition>) {}
    fn type_alternatives(&mut self, _ref_: &mut Ref<TypeAlternative>) {}
    fn assertions(&mut self, _ref_: &mut Ref<Assertion>) {}
    fn notation_declarations(&mut self, _ref_: &mut Ref<NotationDeclaration>) {}
    fn annotations(&mut self, _ref_: &mut Ref<Annotation>) {}
    fn simple_type_definitions(&mut self, _ref_: &mut Ref<SimpleTypeDefinition>) {}
    fn type_definitions(&mut self, _ref_: &mut Ref<TypeDefinition>) {}
    fn constraining_facets(&mut self, _ref_: &mut Ref<ConstrainingFacet>) {}
}

compcontainer! {
    attribute_declarations: AttributeDeclaration,
    element_declarations: ElementDeclaration,
    complex_type_definitions: ComplexTypeDefinition,
    attribute_uses: AttributeUse,
    attribute_group_definitions: AttributeGroupDefinition,
    model_group_definitions: ModelGroupDefinition,
    model_groups: ModelGroup,
    particles: Particle,
    wildcards: Wildcard,
    identity_constraint_definitions: IdentityConstraintDefinition,
    type_alternatives: TypeAlternative,
    assertions: Assertion,
    notation_declarations: NotationDeclaration,
    annotations: Annotation,
    simple_type_definitions: SimpleTypeDefinition,

    type_definitions: TypeDefinition, // TODO
    constraining_facets: ConstrainingFacet,
}

pub trait Refable: Sized {
    fn intermediate_container(container: &IntermediateComponentContainer) -> &[Intermediate<Self>];
    fn intermediate_container_mut(
        container: &mut IntermediateComponentContainer,
    ) -> &mut Vec<Intermediate<Self>>;

    fn container(container: &SchemaComponentContainer) -> &[Self];

    fn visit_ref<V: ConcreteRefVisitor + ?Sized>(ref_: &mut Ref<Self>, visitor: &mut V);
}

pub struct Ref<T: Refable>(i32, PhantomData<T>);

// derive(...) does not work if T itself does not derive the trait, even though it is only "used"
// in the PhantomData; hence we have to manually implement Copy/Clone/Debug for the Ref type.

impl<T: Refable> Copy for Ref<T> {}

impl<T: Refable> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

use std::fmt;
impl<T: Refable> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (_, type_name) = std::any::type_name::<T>().rsplit_once(':').unwrap();
        write!(f, "Ref<{}>({})", type_name, self.0)
    }
}

impl<T: Refable> Ref<T> {
    fn new(id: u32) -> Self {
        let id: i32 = id.try_into().unwrap();
        Self(id, PhantomData)
    }

    fn new_unresolved(id: u32) -> Self {
        let id: i32 = id.try_into().unwrap();
        Self(-id, PhantomData)
    }

    pub fn id(&self) -> i32 {
        self.0
    }

    fn index(&self) -> usize {
        self.id()
            .unsigned_abs()
            .try_into()
            .expect("ID did not fit into usize")
    }

    pub fn get(self, container: &SchemaComponentContainer) -> &T {
        assert!(self.is_resolved());
        &T::container(container)[self.index()]
    }

    pub fn get_intermediate(self, container: &IntermediateComponentContainer) -> Option<&T> {
        assert!(self.is_resolved());
        T::intermediate_container(container)[self.index()].as_ref()
    }

    pub fn is_resolved(&self) -> bool {
        self.id() >= 0
    }

    fn as_resolved(self) -> Option<RRef<T>> {
        if self.is_resolved() {
            Some(RRef(self))
        } else {
            None
        }
    }
}

/// (TODO) Newtype which ensures that a Ref is always resolved
#[derive(Debug)]
struct RRef<T: Refable>(Ref<T>);

impl<T: Refable> Copy for RRef<T> {}

impl<T: Refable> Clone for RRef<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

use std::ops::Deref;

impl<T: Refable> Deref for RRef<T> {
    type Target = Ref<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait RefVisitor {
    fn visit_ref<T: Refable>(&mut self, ref_: &mut Ref<T>);
}

pub trait RefsVisitable {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor);
}

pub struct MappingContext {
    pub components: IntermediateComponentContainer,
}

impl MappingContext {
    pub fn new() -> Self {
        let mut components = IntermediateComponentContainer::default();
        super::builtins::register_builtins(&mut components);
        MappingContext { components }
    }
}
