use super::{
    annotation::Annotation,
    attribute_decl::AttributeDeclaration,
    attribute_group_def::AttributeGroupDefinition,
    element_decl::ElementDeclaration,
    identity_constraint_def::IdentityConstraintDefinition,
    model_group_def::ModelGroupDefinition,
    notation_decl::NotationDeclaration,
    shared::TypeDefinition,
    xstypes::{Sequence, Set},
    ComplexTypeDefinition, MappingContext, Ref, SimpleTypeDefinition,
};
use roxmltree::Node;

/// Schema Component: Schema, a kind of Annotated Component (§3.17)
#[derive(Clone, Debug)]
pub struct Schema {
    pub annotations: Sequence<Ref<Annotation>>,
    pub type_definitions: Set<Ref<TypeDefinition>>,
    pub attribute_declarations: Set<Ref<AttributeDeclaration>>,
    pub element_declarations: Set<Ref<ElementDeclaration>>,
    pub attribute_group_definitions: Set<Ref<AttributeGroupDefinition>>,
    pub model_group_definitions: Set<Ref<ModelGroupDefinition>>,
    pub notation_declarations: Set<Ref<NotationDeclaration>>,
    pub identity_constraint_definitions: Set<Ref<IdentityConstraintDefinition>>,
}

impl Schema {
    pub fn map_from_xml(context: &mut MappingContext, schema: Node) -> Self {
        assert_eq!(schema.tag_name().name(), "schema");

        // {type definitions}
        //   The simple and complex type definitions corresponding to all the <simpleType> and
        //   <complexType> element information items in the [children], if any, plus any
        //   definitions brought in via <include> (see Assembling a schema for a single target
        //   namespace from multiple schema definition documents (<include>) (§4.2.3)), <override>
        //   (see Overriding component definitions (<override>) (§4.2.5)), <redefine> (see
        //   Including modified component definitions (<redefine>) (§4.2.4)), and <import> (see
        //   References to schema components across namespaces (<import>) (§4.2.6)).
        let mut type_definitions = Sequence::new();
        for simple_type in schema
            .children()
            .filter(|e| e.tag_name().name() == "simpleType")
        {
            let simple_type_def = SimpleTypeDefinition::map_from_xml(context, simple_type, schema);
            type_definitions.push(
                context
                    .components
                    .create(TypeDefinition::Simple(simple_type_def)),
            );
        }
        for complex_type in schema
            .children()
            .filter(|e| e.tag_name().name() == "complexType")
        {
            let complex_type_def =
                ComplexTypeDefinition::map_from_xml(context, complex_type, schema);
            type_definitions.push(
                context
                    .components
                    .create(TypeDefinition::Complex(complex_type_def)),
            );
        }

        // {attribute declarations}
        //   The (top-level) attribute declarations corresponding to all the <attribute> element
        //   information items in the [children], if any, plus any declarations brought in via
        //   <include>, <override>, <redefine>, and <import>.
        let attribute_declarations = schema
            .children()
            .filter(|e| e.tag_name().name() == "attribute")
            .map(|attribute| AttributeDeclaration::map_from_xml_global(context, attribute, schema))
            .collect::<Sequence<_>>();

        // {element declarations}
        //   The (top-level) element declarations corresponding to all the <element> element
        //   information items in the [children], if any, plus any declarations brought in via
        //   <include>, <override>, <redefine>, and <import>.
        let element_declarations = schema
            .children()
            .filter(|e| e.tag_name().name() == "element")
            .map(|element| ElementDeclaration::map_from_xml_top_level(context, element, schema))
            .collect::<Sequence<_>>();

        // {attribute group definitions}
        //   The attribute group definitions corresponding to all the <attributeGroup> element
        //   information items in the [children], if any, plus any definitions brought in via
        //   <include>, <override>, <redefine>, and <import>.
        let attribute_group_definitions = schema
            .children()
            .filter(|e| e.tag_name().name() == "attributeGroup")
            .map(|attribute_group| {
                AttributeGroupDefinition::map_from_xml(context, attribute_group, schema)
            })
            .collect::<Sequence<_>>();

        // {model group definitions}
        //   The model group definitions corresponding to all the <group> element information items
        //   in the [children], if any, plus any definitions brought in via <include>, <redefine>
        //   and <import>.
        let model_group_definitions = schema
            .children()
            .filter(|e| e.tag_name().name() == "group")
            .map(|group| ModelGroupDefinition::map_from_xml(context, group, schema))
            .collect::<Sequence<_>>();

        // {notation declarations}
        //   The notation declarations corresponding to all the <notation> element information
        //   items in the [children], if any, plus any declarations brought in via <include>,
        //   <override>, <redefine>, and <import>.
        let notation_declarations = schema
            .children()
            .filter(|e| e.tag_name().name() == "notation")
            .map(|notation| NotationDeclaration::map_from_xml(context, notation, schema))
            .collect::<Sequence<_>>();

        // {identity-constraint definitions}
        //   The identity-constraint definitions corresponding to all the <key>, <keyref>, and
        //   <unique> element information items anywhere within the [children], if any, plus any
        //   definitions brought in via <include>, <override>, <redefine>, and <import>.
        // TODO
        let identity_constraint_definitions = Sequence::new();

        // {annotations}
        //   The ·annotation mapping· of the set of elements containing the <schema> and all the
        //   <include>, <redefine>, <override>, <import>, and <defaultOpenContent> [children], if
        //   any, as defined in XML Representation of Annotation Schema Components (§3.15.2).
        let mut annot_elements = vec![schema];
        schema
            .children()
            .filter(|e| {
                [
                    "include",
                    "redefine",
                    "override",
                    "import",
                    "defaultOpenContent",
                ]
                .contains(&e.tag_name().name())
            })
            .for_each(|e| annot_elements.push(e));
        let annotations = Annotation::xml_element_set_annotation_mapping(context, &annot_elements);

        Self {
            annotations,
            type_definitions,
            attribute_declarations,
            element_declarations,
            attribute_group_definitions,
            model_group_definitions,
            notation_declarations,
            identity_constraint_definitions,
        }
    }
}
