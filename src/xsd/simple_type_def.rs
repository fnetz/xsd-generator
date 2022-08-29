use super::{
    annotation::Annotation,
    attribute_decl::AttributeDeclaration,
    complex_type_def::ComplexTypeDefinition,
    components::MappingContext,
    constraining_facet::ConstrainingFacet,
    element_decl::ElementDeclaration,
    fundamental_facet::FundamentalFacet,
    shared::TypeDefinition,
    xstypes::{AnyURI, NCName, Sequence, Set},
    Ref, RefVisitor, RefsVisitable,
};
use roxmltree::Node;

/// Simple Type Definition, a kind of [Type Definition](super::shared::TypeDefinition), §3.16
#[derive(Clone, Debug)]
pub struct SimpleTypeDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: Option<NCName>,
    pub target_namespace: Option<AnyURI>,
    pub final_: Set<DerivationMethod>,
    pub context: Option<Context>,
    // TODO Option for root?
    pub base_type_definition: Option<Ref<TypeDefinition>>,
    pub facets: Set<ConstrainingFacet>,
    pub fundamental_facets: Set<FundamentalFacet>,
    pub variety: Option<Variety>,
    pub primitive_type_definition: Option<Ref<SimpleTypeDefinition>>,
    pub item_type_definition: Option<Ref<SimpleTypeDefinition>>,
    pub member_type_definitions: Option<Sequence<Ref<SimpleTypeDefinition>>>,
}

#[derive(Clone, Debug)]
pub enum DerivationMethod {
    Extension,
    Restriction,
    List,
    Union,
}

#[derive(Clone, Debug)]
pub enum Context {
    Attribute(Ref<AttributeDeclaration>),
    Element(Ref<ElementDeclaration>),
    ComplexType(Ref<ComplexTypeDefinition>),
    SimpleType(Ref<SimpleTypeDefinition>),
}

#[derive(Copy, Clone, Debug)]
pub enum Variety {
    Atomic,
    List,
    Union,
}

impl SimpleTypeDefinition {
    pub fn map_from_xml(
        _context: &mut MappingContext,
        _simple_type: Node,
        _schema: Node,
    ) -> Ref<Self> {
        todo!()
    }
}

impl RefsVisitable for SimpleTypeDefinition {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
        self.context.as_mut().map(|context| match context {
            Context::Attribute(attr) => visitor.visit_ref(attr),
            Context::Element(element) => visitor.visit_ref(element),
            Context::ComplexType(complex_type) => visitor.visit_ref(complex_type),
            Context::SimpleType(simple_type) => visitor.visit_ref(simple_type),
        });
        self.base_type_definition
            .as_mut()
            .map(|base| visitor.visit_ref(base));
        self.member_type_definitions
            .as_mut()
            .map(|types| types.iter_mut().for_each(|type_| visitor.visit_ref(type_)));
        self.primitive_type_definition
            .as_mut()
            .map(|type_| visitor.visit_ref(type_));
        self.item_type_definition
            .as_mut()
            .map(|type_| visitor.visit_ref(type_));
        self.member_type_definitions
            .as_mut()
            .map(|types| types.iter_mut().for_each(|type_| visitor.visit_ref(type_)));
    }
}
