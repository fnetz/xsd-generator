use super::{
    annotation::Annotation,
    assertion::Assertion,
    attribute_decl,
    attribute_use::AttributeUse,
    builtins::XS_ANY_TYPE_NAME,
    components::{Component, Named},
    element_decl::ElementDeclaration,
    mapping_context::TopLevelMappable,
    model_group::Compositor,
    particle::{MaxOccurs, Particle},
    shared::TypeDefinition,
    simple_type_def::SimpleTypeDefinition,
    values::actual_value,
    wildcard::{self, Wildcard},
    xstypes::{AnyURI, NCName, QName, Sequence, Set},
    AttributeDeclaration, MappingContext, ModelGroup, Ref, Term,
};
use roxmltree::Node;

/// Schema Component: Complex Type Definition, a kind of Type Definition (§3.4)
#[derive(Clone, Debug)]
pub struct ComplexTypeDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: Option<NCName>,
    pub target_namespace: Option<AnyURI>,
    pub base_type_definition: TypeDefinition,
    pub final_: Set<DerivationMethod>,
    pub context: Option<Context>,
    pub derivation_method: Option<DerivationMethod>,
    pub abstract_: bool,
    pub attribute_uses: Set<Ref<AttributeUse>>,
    pub attribute_wildcard: Option<Ref<Wildcard>>,
    pub content_type: ContentType,
    pub prohibited_substitutions: Set<DerivationMethod>,
    pub assertions: Sequence<Ref<Assertion>>,
}

#[derive(Clone, Debug)]
pub enum Context {
    Element(Ref<ElementDeclaration>),
    ComplexType(Ref<ComplexTypeDefinition>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DerivationMethod {
    Extension,
    Restriction,
}

/// Property Record: Content Type (§3.4)
#[derive(Clone, Debug)]
pub struct ContentType {
    pub variety: ContentTypeVariety,
    pub particle: Option<Ref<Particle>>,
    pub open_content: Option<OpenContent>,
    pub simple_type_definition: Option<Ref<SimpleTypeDefinition>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ContentTypeVariety {
    Empty,
    Simple,
    ElementOnly,
    Mixed,
}

/// Property Record: Open Content
#[derive(Clone, Debug)]
pub struct OpenContent {
    pub mode: OpenContentMode,
    pub wildcard: Ref<Wildcard>,
}

#[derive(Clone, Debug)]
pub enum OpenContentMode {
    Interleave,
    Suffix,
}

impl ComplexTypeDefinition {
    pub const TAG_NAME: &'static str = "complexType";

    pub(super) fn name_from_xml(complex_type: Node, schema: Node) -> Option<QName> {
        // {name}
        //   The ·actual value· of the name [attribute] if present, otherwise ·absent·.
        let name = complex_type
            .attribute("name")
            .map(|v| actual_value::<String>(v, complex_type));

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, complex_type));

        name.map(|name| QName::with_optional_namespace(target_namespace, name))
    }

    pub(super) fn map_from_xml(
        context: &mut MappingContext,
        complex_type: Node,
        schema: Node,
        ancestor_element: Option<Ref<ElementDeclaration>>,
        tlref: Option<Ref<Self>>,
    ) -> Ref<Self> {
        let complex_type_ref = tlref.unwrap_or_else(|| context.components.reserve::<Self>());

        if let Some(simple_content) = complex_type
            .children()
            .find(|c| c.tag_name().name() == "simpleContent")
        {
            Self::map_with_simple_content(complex_type, simple_content, schema)
        } else if let Some(complex_content) = complex_type
            .children()
            .find(|c| c.tag_name().name() == "complexContent")
        {
            Self::map_with_explicit_complex_content(
                context,
                complex_type_ref,
                complex_type,
                complex_content,
                schema,
                ancestor_element,
            )
        } else {
            Self::map_with_implicit_complex_content(
                context,
                complex_type_ref,
                complex_type,
                schema,
                ancestor_element,
            )
        }

        assert!(
            context.components.is_present(complex_type_ref),
            "ComplexTypeDefinition mapper failed to populate ref"
        );
        complex_type_ref
    }

    fn map_with_simple_content(_complex_type: Node, _simple_content: Node, _schema: Node) {
        todo!("Complex type def with simple content")
    }

    fn map_with_explicit_complex_content(
        context: &mut MappingContext,
        complex_type_ref: Ref<Self>,
        complex_type: Node,
        complex_content: Node,
        schema: Node,
        ancestor_element: Option<Ref<ElementDeclaration>>,
    ) {
        let content = complex_content
            .children()
            .find(|c| ["restriction", "extension"].contains(&c.tag_name().name()))
            .unwrap();

        // {base type definition}
        //   The type definition ·resolved· to by the ·actual value· of the base [attribute]
        let base_type_definition = content
            .attribute("base")
            .map(|base| actual_value::<QName>(base, content))
            .map(|n| context.resolver.resolve(&n))
            .unwrap();

        // {derivation method}
        //   If the <restriction> alternative is chosen, then restriction, otherwise (the
        //   <extension> alternative is chosen) extension.
        let derivation_method = match content.tag_name().name() {
            "restriction" => DerivationMethod::Restriction,
            "extension" => DerivationMethod::Extension,
            _ => unreachable!(),
        };

        let content_type = ContentType::map_complex(
            context,
            complex_type_ref,
            complex_type,
            None,
            schema,
            derivation_method,
        );

        let common = Self::map_common(context, complex_type, schema, ancestor_element);

        let attribute_uses =
            Self::map_attribute_uses_property(context, complex_type_ref, complex_type, schema);

        // TODO attribute wildcard

        context.components.insert(
            complex_type_ref,
            Self {
                base_type_definition,
                derivation_method: Some(derivation_method),
                content_type,
                attribute_uses,
                ..common
            },
        );
    }

    fn map_with_implicit_complex_content(
        context: &mut MappingContext,
        complex_type_ref: Ref<Self>,
        complex_type: Node,
        schema: Node,
        ancestor_element: Option<Ref<ElementDeclaration>>,
    ) {
        // {base type definition} ·xs:anyType·
        let base_type_definition = context.resolver.resolve(&XS_ANY_TYPE_NAME);

        // {derivation method}    restriction
        let derivation_method = Some(DerivationMethod::Restriction);

        let content_type = ContentType::map_complex(
            context,
            complex_type_ref,
            complex_type,
            None,
            schema,
            derivation_method.unwrap(),
        );

        let common = Self::map_common(context, complex_type, schema, ancestor_element);

        let attribute_uses =
            Self::map_attribute_uses_property(context, complex_type_ref, complex_type, schema);

        // TODO attribute wildcard

        context.components.insert(
            complex_type_ref,
            Self {
                base_type_definition,
                derivation_method,
                content_type,
                attribute_uses,
                ..common
            },
        );
    }

    fn map_common(
        mapping_context: &mut MappingContext,
        complex_type: Node,
        schema: Node,
        ancestor_element: Option<Ref<ElementDeclaration>>,
    ) -> Self {
        // {name}
        //   The ·actual value· of the name [attribute] if present, otherwise ·absent·.
        let name = complex_type
            .attribute("name")
            .map(|v| actual_value::<String>(v, complex_type));

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, complex_type));

        // {abstract}
        //   The ·actual value· of the abstract [attribute], if present, otherwise false.
        let abstract_ = complex_type
            .attribute("abstract")
            .map(|v| actual_value::<bool>(v, complex_type))
            .unwrap_or(false);

        // TODO impl, doc
        let prohibited_substitutions = Set::new();

        // TODO same
        let final_ = Set::new();

        // {context}
        //   If the name [attribute] is present, then ·absent·, otherwise (among the ancestor
        //   element information items there will be a nearest <element>), the Element Declaration
        //   corresponding to the nearest <element> information item among the the ancestor element
        //   information items.
        let context = if complex_type.has_attribute("name") {
            None
        } else {
            let ancestor_element = ancestor_element.expect(
                "Expected an unnamed complex type definition to have an ancestor <element>",
            );
            Some(Context::Element(ancestor_element))
        };

        // {assertions}
        //   A sequence whose members are Assertions drawn from the following sources, in order:
        //   1 The {assertions} of the {base type definition}.
        //   2 Assertions corresponding to all the <assert> element information items among the
        //     [children] of <complexType>, <restriction> and <extension>, if any, in document
        //     order.
        // TODO
        let assertions = Sequence::new();

        // {annotations}
        //   The ·annotation mapping· of the set of elements containing the <complexType>, the
        //   <openContent> [child], if present, the <attributeGroup> [children], if present, the
        //   <simpleContent> and <complexContent> [children], if present, and their <restriction>
        //   and <extension> [children], if present, and their <openContent> and <attributeGroup>
        //   [children], if present, as defined in XML Representation of Annotation Schema
        //   Components (§3.15.2).
        let mut annot_elements = vec![complex_type];
        complex_type
            .children()
            // TODO children of simple/complexContent
            .filter(|e| {
                [
                    "openContent",
                    "attributeGroup",
                    "simpleContent",
                    "complexContent",
                ]
                .contains(&e.tag_name().name())
            })
            .for_each(|e| annot_elements.push(e));
        let annotations =
            Annotation::xml_element_set_annotation_mapping(mapping_context, &annot_elements);

        Self {
            annotations,
            name,
            target_namespace,
            final_,
            context,
            abstract_,
            prohibited_substitutions,
            assertions,

            // Populated in the specific mapping implementations
            // TODO restructure
            base_type_definition: mapping_context.resolver.resolve(&XS_ANY_TYPE_NAME), // TODO !!
            derivation_method: None,
            content_type: ContentType {
                variety: ContentTypeVariety::Empty,
                open_content: None,
                particle: None,
                simple_type_definition: None,
            },
            attribute_uses: Set::new(),
            attribute_wildcard: None,
        }
    }

    /// Maps the {attribute uses} property
    fn map_attribute_uses_property(
        context: &mut MappingContext,
        complex_type_ref: Ref<Self>,
        complex_type: Node,
        schema: Node,
    ) -> Vec<Ref<AttributeUse>> {
        // If the <schema> ancestor has a defaultAttributes attribute, and the <complexType>
        // element does not have defaultAttributesApply = false, then the {attribute uses} property
        // is computed as if there were an <attributeGroup> [child] with empty content and a ref
        // [attribute] whose ·actual value· is the same as that of the defaultAttributes
        // [attribute] appearing after any other <attributeGroup> [children]. Otherwise proceed as
        // if there were no such <attributeGroup> [child].
        if schema.has_attribute("defaultAttributes")
            && complex_type
                .attribute("defaultAttributesApply")
                .map(|v| actual_value::<bool>(v, complex_type))
                != Some(false)
        {
            todo!("default attributes")
        } else {
            // Then the value is a union of sets of attribute uses as follows
            let mut attribute_uses = Set::new();

            // 1 The set of attribute uses corresponding to the <attribute> [children], if any.
            complex_type
                .children()
                .filter(|c| c.tag_name().name() == "attribute")
                .map(|attribute| {
                    AttributeDeclaration::map_from_xml_local(
                        context,
                        attribute,
                        schema,
                        attribute_decl::ScopeParent::ComplexType(complex_type_ref),
                    )
                })
                .for_each(|(_attrib_decl, attrib_use)| attribute_uses.push(attrib_use));

            // 2 The {attribute uses} of the attribute groups ·resolved· to by the ·actual value·s
            //   of the ref [attribute] of the <attributeGroup> [children], if any.
            // TODO

            // 3 The attribute uses "inherited" from the {base type definition} T, as described by
            //   the appropriate case among the following:
            //  3.1 If T is a complex type definition and {derivation method} = extension, then the
            //    attribute uses in T.{attribute uses} are inherited.
            //  3.2 If T is a complex type definition and {derivation method} = restriction, then
            //    the attribute uses in T.{attribute uses} are inherited, with the exception of
            //    those with an {attribute declaration} whose expanded name is one of the
            //    following:
            //   3.2.1 the expanded name of the {attribute declaration} of an attribute use which
            //     has already been included in the set, following the rules in clause 1 or clause
            //     2 above;
            //   3.2.2 the expanded name of the {attribute declaration} of what would have been an
            //     attribute use corresponding to an <attribute> [child], if the <attribute> had
            //     not had use = prohibited. Note: This sub-clause handles the case where the base
            //     type definition T allows the attribute in question, but the restriction
            //     prohibits it.
            //  3.3 otherwise no attribute use is inherited.
            // TODO

            attribute_uses
        }
    }
}

impl ContentType {
    fn map_complex(
        context: &mut MappingContext,
        complex_type_ref: Ref<ComplexTypeDefinition>,
        complex_type: Node,
        complex_content: Option<Node>,
        schema: Node,
        derivation_method: DerivationMethod,
    ) -> Self {
        // When the mapping rule below refers to "the [children]", ...
        let children_elem = if let Some(complex_content) = complex_content {
            // ... for a <complexType> source declaration with a <complexContent> child, the
            // [children] of <extension> or <restriction> (whichever appears as a child of
            // <complexContent>) are meant
            complex_content
                .children()
                .find(|c| ["extension", "restriction"].contains(&c.tag_name().name()))
                .unwrap()
        } else {
            // If no <complexContent> is present, then the [children] of the <complexType> source
            // declaration itself are meant
            complex_type
        };

        // 1 Let the effective mixed be the appropriate case among the following:
        let effective_mixed =
            if let Some(mixed) = complex_content.and_then(|cc| cc.attribute("mixed")) {
                // 1.1 If the mixed [attribute] is present on <complexContent>, then its ·actual value·;
                actual_value::<bool>(mixed, complex_type)
            } else if let Some(mixed) = complex_type.attribute("mixed") {
                // 1.2 If the mixed [attribute] is present on <complexType>, then its ·actual value·;
                actual_value::<bool>(mixed, complex_type)
            } else {
                // 1.3 otherwise false.
                false
            };

        // TODO maxOccurs unbounded

        // 2 Let the explicit content be the appropriate case among the following:
        // 2.1 If at least one of the following is true
        let explicit_content: Option<Ref<Particle>> = if
        // 2.1.1 There is no <group>, <all>, <choice> or <sequence> among the [children];
        !children_elem.children().any(|c| ["group", "all", "choice", "sequence"].contains(&c.tag_name().name())) ||
                // 2.1.2 There is an <all> or <sequence> among the [children] with no [children] of its own excluding <annotation>;
                children_elem.children().any(|c| ["all", "sequence"].contains(&c.tag_name().name()) && !c.children().any(|c| c.tag_name().name() != Annotation::TAG_NAME)) ||
                // 2.1.3 There is among the [children] a <choice> element whose minOccurs [attribute] has the ·actual value· 0 and which has no [children] of its own except for <annotation>;
                children_elem.children().any(|c| c.tag_name().name() == "choice" && c.attribute("minOccurs").map(|v| actual_value::<u64>(v, complex_type)) == Some(0) && !c.children().any(|c| c.tag_name().name() != Annotation::TAG_NAME)) ||
                // 2.1.4 The <group>, <all>, <choice> or <sequence> element among the [children] has a maxOccurs [attribute] with an ·actual value· of 0;
                children_elem.children().find(|c| ["group", "all", "choice", "sequence"].contains(&c.tag_name().name())).map(|c| c.attribute("maxOccurs").filter(|m| *m != "unbounded").map(|v| actual_value::<u64>(v, complex_type)) == Some(0)) == Some(true)
        {
            // then empty
            None
        } else {
            // 2.2 otherwise the particle corresponding to the <all>, <choice>, <group> or <sequence> among the [children].
            Some(
                children_elem
                    .children()
                    .find_map(|c| match c.tag_name().name() {
                        "all" | "choice" | "sequence" => Some(Particle::map_from_xml_model_group(
                            context,
                            c,
                            schema,
                            complex_type_ref,
                        )),
                        "group" => Some(Particle::map_from_xml_group_reference(context, c)),
                        _ => None,
                    }),
            )
            .unwrap()
        };

        // 3 Let the effective content be the appropriate case among the following:
        let effective_content = if let Some(explicit_content) = explicit_content {
            // 3.2 otherwise the ·explicit content·.
            Some(explicit_content)
        } else {
            // 3.1 If the ·explicit content· is empty , then the appropriate case among the following:
            if effective_mixed {
                // 3.1.1 If the ·effective mixed· is true, then a particle whose properties are as
                //   follows:
                // {min occurs} 1
                // {max occurs} 1
                // {term}       a model group whose {compositor} is sequence and whose {particles}
                //              is empty.
                let term = Term::ModelGroup(context.components.create(ModelGroup {
                    compositor: Compositor::Sequence,
                    particles: Sequence::new(),
                    annotations: Sequence::new(),
                }));
                Some(context.components.create(Particle {
                    min_occurs: 1,
                    max_occurs: MaxOccurs::Count(1),
                    term,
                    annotations: Sequence::new(),
                }))
            } else {
                // 3.1.2 otherwise empty.
                None
            }
        };

        // TODO derivation method == None?
        // 4 Let the explicit content type be the appropriate case among the following:
        let explicit_content_type = if derivation_method == DerivationMethod::Restriction {
            // 4.1 If {derivation method} = restriction, then the appropriate case among the
            //   following:
            if let Some(effective_content) = effective_content {
                // 4.1.2 otherwise a Content Type as follows:
                // {variety}                 mixed if the ·effective mixed· is true, otherwise
                //                           element-only
                // {particle}                The ·effective content·
                // {open content}           ·absent·
                // {simple type definition} ·absent·
                ContentType {
                    variety: if effective_mixed {
                        ContentTypeVariety::Mixed
                    } else {
                        ContentTypeVariety::ElementOnly
                    },
                    particle: Some(effective_content),
                    open_content: None,
                    simple_type_definition: None,
                }
            } else {
                // 4.1.1 If the ·effective content· is empty , then a Content Type as follows:
                // {variety}                empty
                // {particle}               ·absent·
                // {open content}           ·absent·
                // {simple type definition} ·absent·
                ContentType {
                    variety: ContentTypeVariety::Empty,
                    particle: None,
                    open_content: None,
                    simple_type_definition: None,
                }
            }
        } else {
            // 4.2 If {derivation method} = extension, then the appropriate case among the following:

            // 4.2.1 If the {base type definition} is a simple type definition or is a complex type
            //   definition whose {content type}.{variety} = empty or simple, then a Content Type
            //   as per clause 4.1.1 and clause 4.1.2 above;
            // 4.2.2 If the {base type definition} is a complex type definition whose {content
            //   type}.{variety} = element-only or mixed and the ·effective content· is empty, then
            //   {base type definition}.{content type};
            // 4.2.3 otherwise a Content Type as follows:
            todo!()
        };

        // 5 Let the wildcard element be the appropriate case among the following:
        let wildcard_element = if let Some(open_content) = children_elem
            .children()
            .find(|c| c.tag_name().name() == "openContent")
        {
            // 5.1 If the <openContent> [child] is present , then the <openContent> [child].
            Some(open_content)
            // 5.2 If the <openContent> [child] is not present, the <schema> ancestor has a <defaultOpenContent> [child], and one of the following is true
        } else if let Some(default_open_content) = schema
            .children()
            .find(|c| c.tag_name().name() == "defaultOpenContent")
        {
            // 5.2.1 the ·explicit content type· has {variety} ≠ empty
            // 5.2.2 the ·explicit content type· has {variety} = empty and the <defaultOpenContent>
            //   element has appliesToEmpty = true
            if explicit_content_type.variety != ContentTypeVariety::Empty
                || (explicit_content_type.variety == ContentTypeVariety::Empty
                    && default_open_content
                        .attribute("appliesToEmpty")
                        .map(|v| actual_value::<bool>(v, complex_type))
                        == Some(true))
            {
                // then the <defaultOpenContent> [child] of the <schema>.
                Some(default_open_content)
            } else {
                // 5.3 otherwise ·absent·.
                None
            }
        } else {
            // 5.3 otherwise ·absent·.
            None
        };

        // 6 Then the value of the property is the appropriate case among the following:
        if wildcard_element
            .map(|e| {
                e.attribute("mode")
                    .map(|v| actual_value::<&str>(v, complex_type))
                    == Some("none")
            })
            .unwrap_or(true)
        {
            // 6.1 If the ·wildcard element· is ·absent· or is present and has mode = 'none' , then the ·explicit content type·.
            explicit_content_type
        } else {
            // The wildcard element must be present
            let wildcard_element = wildcard_element.unwrap();

            // 6.2 otherwise
            //   {variety}    The {variety} of the ·explicit content type· if it's not empty;
            //                otherwise element-only.
            let variety = if explicit_content_type.variety != ContentTypeVariety::Empty {
                explicit_content_type.variety
            } else {
                ContentTypeVariety::ElementOnly
            };

            //   {particle}  The {particle} of the ·explicit content type· if the {variety} of the
            //               ·explicit content type· is not empty; otherwise a Particle as follows:
            let particle = if explicit_content_type.variety != ContentTypeVariety::Empty {
                explicit_content_type.particle
            } else {
                // {min occurs} 1
                // {max occurs} 1
                // {term}       a model group whose {compositor} is sequence and whose
                //              {particles} is empty.
                let term = Term::ModelGroup(context.components.create(ModelGroup {
                    compositor: Compositor::Sequence,
                    particles: Sequence::new(),
                    annotations: Sequence::new(),
                }));
                Some(context.components.create(Particle {
                    min_occurs: 1,
                    max_occurs: MaxOccurs::Count(1),
                    term,
                    annotations: Sequence::new(),
                }))
            };

            //  {open content} An Open Content as follows:
            let open_content = {
                // {mode} The ·actual value· of the mode [attribute] of the ·wildcard element·, if
                //        present, otherwise interleave.
                let mode = wildcard_element
                    .attribute("mode")
                    .and_then(|v| match v {
                        "interleave" => Some(OpenContentMode::Interleave),
                        "suffix" => Some(OpenContentMode::Suffix),
                        _ => unreachable!(),
                    })
                    .unwrap_or(OpenContentMode::Interleave);

                // {wildcard} Let W be the wildcard corresponding to the <any> [child] of the
                //            ·wildcard element·. If the {open content} of the ·explicit content
                //            type· is ·absent·, then W; otherwise a wildcard whose {process
                //            contents} and {annotations} are those of W, and whose {namespace
                //            constraint} is the wildcard union of the {namespace constraint} of W
                //            and of {open content}.{wildcard} of the ·explicit content type·, as
                //            defined in Attribute Wildcard Union (§3.10.6.3).
                // TODO
                let _w = wildcard_element
                    .children()
                    .find(|c| c.tag_name().name() == "any")
                    .unwrap();
                let wildcard = context.components.create(Wildcard {
                    namespace_constraint: wildcard::NamespaceConstraint {
                        variety: wildcard::NamespaceConstraintVariety::Any,
                        namespaces: Set::new(),
                        disallowed_names: Set::new(),
                    },
                    process_contents: wildcard::ProcessContents::Strict,
                    annotations: Sequence::new(),
                });

                Some(OpenContent { mode, wildcard })
            };

            // {simple type definition} ·absent·
            let simple_type_definition = None;

            Self {
                variety,
                particle,
                open_content,
                simple_type_definition,
            }
        }
    }

    fn map_simple() -> Self {
        todo!()
    }
}

impl Component for ComplexTypeDefinition {
    const DISPLAY_NAME: &'static str = "ComplexTypeDefinition";
}

impl Named for ComplexTypeDefinition {
    fn name(&self) -> Option<QName> {
        self.name.as_ref().map(|local_name| {
            QName::with_optional_namespace(self.target_namespace.as_ref(), local_name)
        })
    }
}

impl TopLevelMappable for ComplexTypeDefinition {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        complex_type: Node,
        schema: Node,
    ) {
        Self::map_from_xml(context, complex_type, schema, None, Some(self_ref));
    }
}
