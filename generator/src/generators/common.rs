use std::collections::HashSet;

use dt_xsd::{
    ComplexTypeDefinition, ElementDeclaration, Ref, SchemaComponentTable, SimpleTypeDefinition,
};

pub(super) trait ComponentVisitor: Sized {
    type ComplexTypeValue;
    fn visit_complex_type(
        &mut self,
        context: &mut GeneratorContext,
        complex_type: Ref<ComplexTypeDefinition>,
    ) -> Self::ComplexTypeValue;

    type SimpleTypeValue;
    fn visit_simple_type(
        &mut self,
        context: &mut GeneratorContext,
        simple_type: Ref<SimpleTypeDefinition>,
    ) -> Self::SimpleTypeValue;

    type ElementDeclarationValue;
    fn visit_element_declaration(
        &mut self,
        context: &mut GeneratorContext,
        element_declaration: Ref<ElementDeclaration>,
    ) -> Self::ElementDeclarationValue;
}

pub(super) struct GeneratorContext<'a> {
    pub(super) table: &'a SchemaComponentTable,
    pub(super) visited_complex_types: HashSet<Ref<ComplexTypeDefinition>>,
    pub(super) visited_simple_types: HashSet<Ref<SimpleTypeDefinition>>,
}

impl<'a> GeneratorContext<'a> {
    pub(super) fn new(table: &'a SchemaComponentTable) -> Self {
        Self {
            table,
            visited_complex_types: HashSet::new(),
            visited_simple_types: HashSet::new(),
        }
    }
}
