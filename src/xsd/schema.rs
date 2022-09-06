use super::{
    annotation::Annotation,
    attribute_decl::AttributeDeclaration,
    attribute_group_def::AttributeGroupDefinition,
    components::{Component, ComponentTraits, HasArenaContainer, Lookup, LookupTables, NamedXml},
    element_decl::ElementDeclaration,
    identity_constraint_def::IdentityConstraintDefinition,
    mapping_context::{TopLevel, TopLevelElements},
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
    pub type_definitions: Set<TypeDefinition>,
    pub attribute_declarations: Set<Ref<AttributeDeclaration>>,
    pub element_declarations: Set<Ref<ElementDeclaration>>,
    pub attribute_group_definitions: Set<Ref<AttributeGroupDefinition>>,
    pub model_group_definitions: Set<Ref<ModelGroupDefinition>>,
    pub notation_declarations: Set<Ref<NotationDeclaration>>,
    pub identity_constraint_definitions: Set<Ref<IdentityConstraintDefinition>>,
}

impl Schema {
    pub(super) fn map_from_xml<'a, 'input: 'a, 'p>(
        context: &mut MappingContext<'a, 'input, 'p>,
        schema: Node<'a, 'input>,
    ) -> Self {
        assert_eq!(schema.tag_name().name(), "schema");

        fn reserve_top_level<'a, 'input: 'a, 'p, C>(
            context: &mut MappingContext<'a, 'input, 'p>,
            node: Node<'a, 'input>,
            schema: Node,
        ) where
            C: Component + NamedXml,
            ComponentTraits: HasArenaContainer<C>,
            LookupTables: Lookup<Ref<C>>,
            TopLevelElements<'a, 'input>: TopLevel<'a, 'input, C>,
        {
            let name = C::get_name_from_xml(node, schema);
            let ref_ = context.reserve::<C>();
            context.register_with_name(name, ref_);
            context.top_level_refs.insert(node, ref_);
        }

        for top_level_element in schema.children().filter(|e| e.is_element()) {
            match top_level_element.tag_name().name() {
                SimpleTypeDefinition::TAG_NAME => {
                    // TODO unnamed top level allowed?
                    let name =
                        SimpleTypeDefinition::name_from_xml(top_level_element, schema).unwrap();
                    let std_ref = context.reserve();
                    context.register_with_name(name, TypeDefinition::Simple(std_ref));
                    context.top_level_refs.insert(top_level_element, std_ref);
                }
                ComplexTypeDefinition::TAG_NAME => {
                    let name =
                        ComplexTypeDefinition::name_from_xml(top_level_element, schema).unwrap();
                    let ctd_ref = context.reserve();
                    context.register_with_name(name, TypeDefinition::Complex(ctd_ref));
                    context.top_level_refs.insert(top_level_element, ctd_ref);
                }
                AttributeDeclaration::TAG_NAME => {
                    reserve_top_level::<AttributeDeclaration>(context, top_level_element, schema);
                }
                ElementDeclaration::TAG_NAME => {
                    reserve_top_level::<ElementDeclaration>(context, top_level_element, schema);
                }
                AttributeGroupDefinition::TAG_NAME => {
                    reserve_top_level::<AttributeGroupDefinition>(
                        context,
                        top_level_element,
                        schema,
                    );
                }
                ModelGroupDefinition::TAG_NAME => {
                    reserve_top_level::<ModelGroupDefinition>(context, top_level_element, schema);
                }
                NotationDeclaration::TAG_NAME => {
                    reserve_top_level::<NotationDeclaration>(context, top_level_element, schema);
                }
                IdentityConstraintDefinition::KEY_TAG_NAME
                | IdentityConstraintDefinition::KEYREF_TAG_NAME
                | IdentityConstraintDefinition::UNIQUE_TAG_NAME => {
                    reserve_top_level::<IdentityConstraintDefinition>(
                        context,
                        top_level_element,
                        schema,
                    );
                }

                Annotation::TAG_NAME => {}
                _ => panic!(
                    "Unknown top level element {}",
                    top_level_element.tag_name().name()
                ),
            }
        }

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
            let simple_type_def = context.request_ref_by_node(simple_type);
            type_definitions.push(TypeDefinition::Simple(simple_type_def));
        }
        for complex_type in schema
            .children()
            .filter(|e| e.tag_name().name() == "complexType")
        {
            let complex_type_def = context.request_ref_by_node(complex_type);
            type_definitions.push(TypeDefinition::Complex(complex_type_def));
        }

        // {attribute declarations}
        //   The (top-level) attribute declarations corresponding to all the <attribute> element
        //   information items in the [children], if any, plus any declarations brought in via
        //   <include>, <override>, <redefine>, and <import>.
        let attribute_declarations = schema
            .children()
            .filter(|e| e.tag_name().name() == "attribute")
            .map(|attribute| context.request_ref_by_node(attribute))
            .collect::<Sequence<_>>();

        // {element declarations}
        //   The (top-level) element declarations corresponding to all the <element> element
        //   information items in the [children], if any, plus any declarations brought in via
        //   <include>, <override>, <redefine>, and <import>.
        let element_declarations = schema
            .children()
            .filter(|e| e.tag_name().name() == "element")
            .map(|element| context.request_ref_by_node(element))
            .collect::<Sequence<_>>();

        // {attribute group definitions}
        //   The attribute group definitions corresponding to all the <attributeGroup> element
        //   information items in the [children], if any, plus any definitions brought in via
        //   <include>, <override>, <redefine>, and <import>.
        let attribute_group_definitions = schema
            .children()
            .filter(|e| e.tag_name().name() == "attributeGroup")
            .map(|attribute_group| context.request_ref_by_node(attribute_group))
            .collect::<Sequence<_>>();

        // {model group definitions}
        //   The model group definitions corresponding to all the <group> element information items
        //   in the [children], if any, plus any definitions brought in via <include>, <redefine>
        //   and <import>.
        let model_group_definitions = schema
            .children()
            .filter(|e| e.tag_name().name() == "group")
            .map(|group| context.request_ref_by_node(group))
            .collect::<Sequence<_>>();

        // {notation declarations}
        //   The notation declarations corresponding to all the <notation> element information
        //   items in the [children], if any, plus any declarations brought in via <include>,
        //   <override>, <redefine>, and <import>.
        let notation_declarations = schema
            .children()
            .filter(|e| e.tag_name().name() == "notation")
            .map(|notation| context.request_ref_by_node(notation))
            .collect::<Sequence<_>>();

        // {identity-constraint definitions}
        //   The identity-constraint definitions corresponding to all the <key>, <keyref>, and
        //   <unique> element information items anywhere within the [children], if any, plus any
        //   definitions brought in via <include>, <override>, <redefine>, and <import>.
        let identity_constraint_definitions = schema
            .children()
            .filter(|e| ["key", "keyref", "unique"].contains(&e.tag_name().name()))
            .map(|icd| context.request_ref_by_node(icd))
            .collect::<Sequence<_>>();

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
