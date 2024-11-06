use super::{
    annotation::Annotation,
    attribute_decl::AttributeDeclaration,
    attribute_group_def::AttributeGroupDefinition,
    components::{
        Component, ComponentTable, ComponentTraits, HasArenaContainer, Lookup, LookupTables,
        NamedXml,
    },
    element_decl::ElementDeclaration,
    identity_constraint_def::IdentityConstraintDefinition,
    import::Import,
    mapping_context::{RootContext, TopLevel, TopLevelElements},
    model_group_def::ModelGroupDefinition,
    notation_decl::NotationDeclaration,
    shared::TypeDefinition,
    values::actual_value,
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

    /// This property is not required by the XSD specification, but is used to
    /// store the original target namespace of the schema.
    pub target_namespace: Option<String>,
}

impl Schema {
    pub fn map_from_xml(
        root_context: &mut RootContext,
        schema: Node,
    ) -> Result<Self, crate::error::XsdError> {
        assert_eq!(schema.tag_name().name(), "schema");

        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|s| actual_value(s, schema));

        let mut context = MappingContext::new(root_context, schema);

        let mut type_definitions = Set::new();
        let mut attribute_declarations = Set::new();
        let mut element_declarations = Set::new();
        let mut attribute_group_definitions = Set::new();
        let mut model_group_definitions = Set::new();
        let mut notation_declarations = Set::new();
        let mut identity_constraint_definitions = Set::new();

        for import in schema
            .children()
            .filter(|c| c.tag_name().name() == Import::TAG_NAME)
        {
            let import = Import::map_from_xml(import, schema)?;
            let child_schema = context.root_mut().resolve_import(&import);

            // NOTE: Import failure is not an error, but there should be a way to emit a
            // notification (and corresponding errors) to the user.

            if let Some(child_schema) = child_schema {
                type_definitions.extend(child_schema.type_definitions);
                attribute_declarations.extend(child_schema.attribute_declarations);
                element_declarations.extend(child_schema.element_declarations);
                attribute_group_definitions.extend(child_schema.attribute_group_definitions);
                model_group_definitions.extend(child_schema.model_group_definitions);
                notation_declarations.extend(child_schema.notation_declarations);
                identity_constraint_definitions
                    .extend(child_schema.identity_constraint_definitions);
            } else {
                eprintln!("Failed to resolve import: {:?}", import);
            }
        }

        fn reserve_top_level<'a, 'input: 'a, C>(
            context: &mut MappingContext<'a, '_, 'input, '_>,
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
                    let name = SimpleTypeDefinition::name_from_xml(top_level_element, schema)
                        .ok_or(crate::error::XsdError::UnnamedTopLevelElement)?;
                    let std_ref = context.reserve();
                    context.register_with_name(name, TypeDefinition::Simple(std_ref));
                    context.top_level_refs.insert(top_level_element, std_ref);
                }
                ComplexTypeDefinition::TAG_NAME => {
                    let name = ComplexTypeDefinition::name_from_xml(top_level_element, schema)
                        .ok_or(crate::error::XsdError::UnnamedTopLevelElement)?;
                    let ctd_ref = context.reserve();
                    context.register_with_name(name, TypeDefinition::Complex(ctd_ref));
                    context.top_level_refs.insert(top_level_element, ctd_ref);
                }
                AttributeDeclaration::TAG_NAME => {
                    reserve_top_level::<AttributeDeclaration>(
                        &mut context,
                        top_level_element,
                        schema,
                    );
                }
                ElementDeclaration::TAG_NAME => {
                    reserve_top_level::<ElementDeclaration>(
                        &mut context,
                        top_level_element,
                        schema,
                    );
                }
                AttributeGroupDefinition::TAG_NAME => {
                    reserve_top_level::<AttributeGroupDefinition>(
                        &mut context,
                        top_level_element,
                        schema,
                    );
                }
                ModelGroupDefinition::TAG_NAME => {
                    reserve_top_level::<ModelGroupDefinition>(
                        &mut context,
                        top_level_element,
                        schema,
                    );
                }
                NotationDeclaration::TAG_NAME => {
                    reserve_top_level::<NotationDeclaration>(
                        &mut context,
                        top_level_element,
                        schema,
                    );
                }
                IdentityConstraintDefinition::KEY_TAG_NAME
                | IdentityConstraintDefinition::KEYREF_TAG_NAME
                | IdentityConstraintDefinition::UNIQUE_TAG_NAME => {
                    reserve_top_level::<IdentityConstraintDefinition>(
                        &mut context,
                        top_level_element,
                        schema,
                    );
                }

                // These tags don't directly contribute top-level components
                Annotation::TAG_NAME | Import::TAG_NAME => {}

                _ => {
                    return Err(crate::error::XsdError::UnknownTopLevelElement(
                        top_level_element.tag_name().name().into(),
                    ))
                }
            }

            top_level_element.descendants().for_each(|e| {
                // "The identity-constraint definitions corresponding to all the <key>, <keyref>,
                // and <unique> element information items *anywhere within* the [children], if any
                // [...]" - Spec pt.1, 3.17.2 XML Representation of Schema Components,
                // {identity-constraint definitions} representation
                if IdentityConstraintDefinition::TAG_NAMES.contains(&e.tag_name().name()) {
                    reserve_top_level::<IdentityConstraintDefinition>(&mut context, e, schema);
                }
            });
        }

        // {type definitions}
        //   The simple and complex type definitions corresponding to all the <simpleType> and
        //   <complexType> element information items in the [children], if any, plus any
        //   definitions brought in via <include> (see Assembling a schema for a single target
        //   namespace from multiple schema definition documents (<include>) (§4.2.3)), <override>
        //   (see Overriding component definitions (<override>) (§4.2.5)), <redefine> (see
        //   Including modified component definitions (<redefine>) (§4.2.4)), and <import> (see
        //   References to schema components across namespaces (<import>) (§4.2.6)).
        for simple_type in schema
            .children()
            .filter(|e| e.tag_name().name() == SimpleTypeDefinition::TAG_NAME)
        {
            let simple_type_def = context.request_ref_by_node(simple_type)?;
            type_definitions.push(TypeDefinition::Simple(simple_type_def));
        }
        for complex_type in schema
            .children()
            .filter(|e| e.tag_name().name() == ComplexTypeDefinition::TAG_NAME)
        {
            let complex_type_def = context.request_ref_by_node(complex_type)?;
            type_definitions.push(TypeDefinition::Complex(complex_type_def));
        }

        // {attribute declarations}
        //   The (top-level) attribute declarations corresponding to all the <attribute> element
        //   information items in the [children], if any, plus any declarations brought in via
        //   <include>, <override>, <redefine>, and <import>.
        for attribute_decl in schema
            .children()
            .filter(|e| e.tag_name().name() == AttributeDeclaration::TAG_NAME)
        {
            let attribute_decl = context.request_ref_by_node(attribute_decl)?;
            attribute_declarations.push(attribute_decl);
        }

        // {element declarations}
        //   The (top-level) element declarations corresponding to all the <element> element
        //   information items in the [children], if any, plus any declarations brought in via
        //   <include>, <override>, <redefine>, and <import>.
        for element_decl in schema
            .children()
            .filter(|e| e.tag_name().name() == ElementDeclaration::TAG_NAME)
        {
            let element_decl = context.request_ref_by_node(element_decl)?;
            element_declarations.push(element_decl);
        }

        // {attribute group definitions}
        //   The attribute group definitions corresponding to all the <attributeGroup> element
        //   information items in the [children], if any, plus any definitions brought in via
        //   <include>, <override>, <redefine>, and <import>.
        for attribute_group_def in schema
            .children()
            .filter(|e| e.tag_name().name() == AttributeGroupDefinition::TAG_NAME)
        {
            let attribute_group_def = context.request_ref_by_node(attribute_group_def)?;
            attribute_group_definitions.push(attribute_group_def);
        }

        // {model group definitions}
        //   The model group definitions corresponding to all the <group> element information items
        //   in the [children], if any, plus any definitions brought in via <include>, <redefine>
        //   and <import>.
        for model_group_def in schema
            .children()
            .filter(|e| e.tag_name().name() == ModelGroupDefinition::TAG_NAME)
        {
            let model_group_def = context.request_ref_by_node(model_group_def)?;
            model_group_definitions.push(model_group_def);
        }

        // {notation declarations}
        //   The notation declarations corresponding to all the <notation> element information
        //   items in the [children], if any, plus any declarations brought in via <include>,
        //   <override>, <redefine>, and <import>.
        for notation_decl in schema
            .children()
            .filter(|e| e.tag_name().name() == NotationDeclaration::TAG_NAME)
        {
            let notation_decl = context.request_ref_by_node(notation_decl)?;
            notation_declarations.push(notation_decl);
        }

        // {identity-constraint definitions}
        //   The identity-constraint definitions corresponding to all the <key>, <keyref>, and
        //   <unique> element information items anywhere within the [children], if any, plus any
        //   definitions brought in via <include>, <override>, <redefine>, and <import>.
        for icd in schema
            .descendants()
            .filter(|e| IdentityConstraintDefinition::TAG_NAMES.contains(&e.tag_name().name()))
        {
            let icd = context.request_ref_by_node(icd)?;
            identity_constraint_definitions.push(icd);
        }

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
        let annotations =
            Annotation::xml_element_set_annotation_mapping(&mut context, &annot_elements);

        Ok(Self {
            annotations,
            type_definitions,
            attribute_declarations,
            element_declarations,
            attribute_group_definitions,
            model_group_definitions,
            notation_declarations,
            identity_constraint_definitions,

            target_namespace,
        })
    }

    pub fn find_element_by_name(
        &self,
        namespace_uri: Option<&str>,
        local_name: &str,
        components: &impl ComponentTable,
    ) -> Option<Ref<ElementDeclaration>> {
        self.element_declarations
            .iter()
            .find(|ed| {
                let ed = ed.get(components);
                ed.target_namespace.as_deref() == namespace_uri && ed.name == local_name
            })
            .copied()
    }
}

// TODO: impl AnnotatedComponent
