use dt_xsd::{
    ComplexTypeDefinition, ElementDeclaration, Ref, Schema, SchemaComponentTable,
    SimpleTypeDefinition,
};

use super::common::{ComponentVisitor, GeneratorContext};

pub struct TypescriptVisitor;

impl ComponentVisitor for TypescriptVisitor {
    type ComplexTypeValue = ();
    fn visit_complex_type(
        &mut self,
        _context: &mut GeneratorContext,
        _complex_type: Ref<ComplexTypeDefinition>,
    ) -> Self::ComplexTypeValue {
        todo!()
    }

    type SimpleTypeValue = ();
    fn visit_simple_type(
        &mut self,
        _context: &mut GeneratorContext,
        _simple_type: Ref<SimpleTypeDefinition>,
    ) -> Self::SimpleTypeValue {
        todo!()
    }

    type ElementDeclarationValue = ();
    fn visit_element_declaration(
        &mut self,
        _context: &mut GeneratorContext,
        _element_declaration: Ref<ElementDeclaration>,
    ) -> Self::ElementDeclarationValue {
        todo!()
    }
}

pub fn generate(_schema: &Schema, components: &SchemaComponentTable) -> String {
    let _context = GeneratorContext::new(components);
    todo!()
}
