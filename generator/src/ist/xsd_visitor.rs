use dt_xsd::{
    AttributeDeclaration, AttributeGroupDefinition, AttributeUse, ComplexTypeDefinition,
    ElementDeclaration, ModelGroup, Particle, Ref, Schema, SimpleTypeDefinition, Wildcard,
};

pub trait XsdVisitor {
    type Result;

    fn schema(&mut self, schema: &Schema) -> Self::Result;

    fn complex_type_definition(
        &mut self,
        complex_type_ref: Ref<ComplexTypeDefinition>,
    ) -> Self::Result;

    fn simple_type_definition(
        &mut self,
        simple_type_ref: Ref<SimpleTypeDefinition>,
    ) -> Self::Result;

    fn particle(&mut self, particle_ref: Ref<Particle>) -> Self::Result;

    fn model_group(&mut self, model_group_ref: Ref<ModelGroup>) -> Self::Result;

    fn element_declaration(&mut self, element_ref: Ref<ElementDeclaration>) -> Self::Result;

    fn attribute_use(&mut self, attribute_ref: Ref<AttributeUse>) -> Self::Result;

    fn attribute_declaration(&mut self, attribute_ref: Ref<AttributeDeclaration>) -> Self::Result;

    fn attribute_group_definition(
        &mut self,
        attribute_group_ref: Ref<AttributeGroupDefinition>,
    ) -> Self::Result;

    fn wildcard(&mut self, wildcard_ref: Ref<Wildcard>) -> Self::Result;
}
