use std::collections::{HashMap, HashSet};

use roxmltree::{Node, NodeId};

use super::{
    components::{
        Component, ComponentResolver, ComponentTraits, ConstructionComponentTable, DynamicRef,
        HasArenaContainer, Lookup, LookupTables, RefNamed,
    },
    AttributeDeclaration, AttributeGroupDefinition, ComplexTypeDefinition, ElementDeclaration,
    IdentityConstraintDefinition, ModelGroupDefinition, NotationDeclaration, Ref,
    SimpleTypeDefinition,
};

#[derive(Default)]
pub(super) struct TopLevelElements<'a, 'input> {
    simple_type_definitions: HashMap<NodeId, Ref<SimpleTypeDefinition>>,
    complex_type_definitions: HashMap<NodeId, Ref<ComplexTypeDefinition>>,
    attribute_declarations: HashMap<NodeId, Ref<AttributeDeclaration>>,
    element_declarations: HashMap<NodeId, Ref<ElementDeclaration>>,
    attribute_group_definitions: HashMap<NodeId, Ref<AttributeGroupDefinition>>,
    model_group_definitions: HashMap<NodeId, Ref<ModelGroupDefinition>>,
    notation_declarations: HashMap<NodeId, Ref<NotationDeclaration>>,
    identity_constraint_definitions: HashMap<NodeId, Ref<IdentityConstraintDefinition>>,

    ref_to_node: HashMap<DynamicRef, Node<'a, 'input>>,
}

impl<'a, 'input: 'a> TopLevelElements<'a, 'input> {
    fn get_node_by_dyn_ref(&self, dynref: DynamicRef) -> Option<Node<'a, 'input>> {
        self.ref_to_node.get(&dynref).copied()
    }
}

pub(super) trait TopLevel<'a, 'input, C>
where
    C: Component,
    ComponentTraits: HasArenaContainer<C>,
{
    fn insert(&mut self, id: Node<'a, 'input>, ref_: Ref<C>);
    fn get_ref_by_node_id(&self, id: NodeId) -> Ref<C>;
    fn get_node_by_ref(&self, ref_: Ref<C>) -> Option<Node<'a, 'input>>;
}

macro_rules! impl_top_level {
    ($field_name:ident: $value_type:ty) => {
        impl<'a, 'input> TopLevel<'a, 'input, $value_type> for TopLevelElements<'a, 'input> {
            fn insert(&mut self, node: Node<'a, 'input>, ref_: Ref<$value_type>) {
                self.$field_name.insert(node.id(), ref_);
                self.ref_to_node.insert(ref_.into(), node);
            }

            fn get_ref_by_node_id(&self, id: NodeId) -> Ref<$value_type> {
                *self.$field_name.get(&id).unwrap()
            }

            fn get_node_by_ref(&self, ref_: Ref<$value_type>) -> Option<Node<'a, 'input>> {
                self.get_node_by_dyn_ref(ref_.into())
            }
        }
    };
}

impl_top_level!(simple_type_definitions: SimpleTypeDefinition);
impl_top_level!(complex_type_definitions: ComplexTypeDefinition);
impl_top_level!(attribute_declarations: AttributeDeclaration);
impl_top_level!(element_declarations: ElementDeclaration);
impl_top_level!(attribute_group_definitions: AttributeGroupDefinition);
impl_top_level!(model_group_definitions: ModelGroupDefinition);
impl_top_level!(notation_declarations: NotationDeclaration);
impl_top_level!(identity_constraint_definitions: IdentityConstraintDefinition);

pub(super) trait TopLevelMappable: Component + Sized
where
    ComponentTraits: HasArenaContainer<Self>,
{
    /// Map this component from a top-level XML element. `self_ref` is the pre-reserved [`Ref`] for
    /// this component, `self_node` is the actual element, and `schema_node` is the root `<schema>`
    /// element.
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        self_node: Node,
        schema_node: Node,
    );
}

pub(super) struct MappingContext<'a, 'input: 'a> {
    pub components: ConstructionComponentTable,
    pub resolver: ComponentResolver,

    pub top_level_refs: TopLevelElements<'a, 'input>,
    in_progress_top_level: HashSet<DynamicRef>,

    schema_node: Node<'a, 'input>,
}

impl<'a, 'input: 'a> MappingContext<'a, 'input> {
    pub(super) fn new(schema: Node<'a, 'input>) -> Self {
        let mut context = MappingContext {
            components: ConstructionComponentTable::new(),
            resolver: ComponentResolver::new(),
            top_level_refs: TopLevelElements::default(),
            in_progress_top_level: HashSet::new(),
            schema_node: schema,
        };
        super::builtins::register_builtins(&mut context);
        context
    }

    /// Wrapper for [`ComponentResolver::register()`]
    pub(super) fn register<R>(&mut self, value: R)
    where
        R: RefNamed + Copy,
        LookupTables: Lookup<R>,
    {
        self.resolver.register(value, &self.components)
    }

    fn ensure_top_level_is_present<C>(&mut self, ref_: Ref<C>, node: Node)
    where
        C: Component + TopLevelMappable + 'static,
        ComponentTraits: HasArenaContainer<C>,
    {
        let dynref: DynamicRef = ref_.into();

        if self.in_progress_top_level.contains(&dynref) {
            panic!("Invalid circular dependency detected!");
        }

        if !self.components.is_present(ref_) {
            self.in_progress_top_level.insert(dynref);

            C::map_from_top_level_xml(self, ref_, node, self.schema_node);
            assert!(self.components.is_present(ref_));

            let was_removed = self.in_progress_top_level.remove(&dynref);
            assert!(was_removed);
        }
    }

    pub(super) fn request<C>(&mut self, ref_: Ref<C>) -> &C
    where
        C: Component + TopLevelMappable + 'static,
        ComponentTraits: HasArenaContainer<C>,
        TopLevelElements<'a, 'input>: TopLevel<'a, 'input, C>,
    {
        let node = self.top_level_refs.get_node_by_ref(ref_);
        if let Some(node) = node {
            self.ensure_top_level_is_present(ref_, node);
        }
        ref_.get(&self.components)
    }

    pub(super) fn request_ref_by_node<C>(&mut self, node: Node) -> Ref<C>
    where
        C: Component + TopLevelMappable + 'static,
        ComponentTraits: HasArenaContainer<C>,
        TopLevelElements<'a, 'input>: TopLevel<'a, 'input, C>,
    {
        let ref_: Ref<C> = self.top_level_refs.get_ref_by_node_id(node.id());
        self.ensure_top_level_is_present(ref_, node);
        ref_
    }
}
