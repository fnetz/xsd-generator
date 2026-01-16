use crate::{
    MappingContext, Particle, Ref, SimpleTypeDefinition, Term,
    annotation::Annotation,
    builtins::{XS_ANY_TYPE_NAME, XS_STRING_NAME},
    complex_type_def::{self, ComplexTypeDefP0, ComplexTypeDefinition, ContentType},
    components::{Component, Named, NamedXml},
    error::XsdError,
    identity_constraint_def::{IdentityConstraintDefinition, IdentityConstraintDefinitionP0},
    mapping_context::TopLevelMappable,
    model_group_def::ModelGroupDefinition,
    particle::MaxOccurs,
    shared::{self, TypeDefinition},
    simple_type_def::{Context as SimpleContext, SimpleTypeDefP0},
    type_alternative::{TypeAlternative, TypeAlternativeP0},
    values::{ActualValue, actual_value},
    xstypes::{AnyURI, NCName, QName, Sequence, Set},
};
use roxmltree::Node;

/// Schema Component: Element Declaration, a kind of [Term](super::shared::Term) (§3.3)
#[derive(Clone, Debug)]
pub struct ElementDeclaration {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub type_definition: TypeDefinition,
    pub type_table: Option<TypeTable>,
    pub scope: Scope,
    pub value_constraint: Option<ValueConstraint>,
    pub nillable: bool,
    pub identity_constraint_definitions: Set<Ref<IdentityConstraintDefinition>>,
    pub substitution_group_affiliations: Set<Ref<ElementDeclaration>>,
    pub substitution_group_exclusions: Set<GroupExlusion>,
    pub disallowed_substitutions: Set<SubstitutionMethod>,
    pub abstract_: bool,
}

pub type GroupExlusion = complex_type_def::DerivationMethod;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SubstitutionMethod {
    Substitution,
    Extension,
    Restriction,
}

impl ActualValue<'_> for SubstitutionMethod {
    fn convert(src: &str, _parent: Node) -> Self {
        match src {
            "substitution" => Self::Substitution,
            "extension" => Self::Extension,
            "restriction" => Self::Restriction,
            _ => panic!("Invalid value for substitution method"),
        }
    }
}

/// Property Record: Type Table (§3.3)
#[derive(Clone, Debug)]
pub struct TypeTable {
    pub alternatives: Sequence<Ref<TypeAlternative>>,
    pub default_type_definition: Ref<TypeAlternative>,
}

/// Property Record: Scope (§3.3)
pub type Scope = shared::Scope<ScopeParent>;

pub use shared::ScopeVariety;
use shared::ValueConstraintVariety;

#[derive(Copy, Clone, Debug)]
pub enum ScopeParent {
    ComplexType(Ref<ComplexTypeDefinition>),
    Group(Ref<ModelGroupDefinition>),
}

/// Property Record: Value Constraint (§3.3)
pub type ValueConstraint = shared::ValueConstraint;

impl NamedXml for ElementDeclaration {
    fn get_name_from_xml(element: Node, schema: Node) -> QName {
        // {name} The ·actual value· of the name [attribute].
        let name = element
            .attribute("name")
            .map(|v| actual_value::<String>(v, element))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the parent <schema> element
        //   information item, or ·absent· if there is none.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, element));

        QName::with_optional_namespace(target_namespace, name)
    }
}

pub(crate) struct ElementDeclarationPhase0 {
    name: NCName,
    type_definition: ElDeclP0TypeDefinition,
    type_table: Option<ElDeclP0TypeTable>,
    nillable: bool,
    value_constraint: Option<ElDeclP0ValueConstraint>,
    identity_constraint_definitions: Vec<IdentityConstraintDefinitionP0>,
    substitution_group_affiliations: MustResolve<Vec<QName>>,
    disallowed_substitutions: Vec<SubstitutionMethod>,
    substitution_group_exclusions: Vec<complex_type_def::DerivationMethod>,
    abstract_: bool,
    annotations: Vec<Annotation>,
}

pub(crate) enum ElDeclP0TypeDefinition {
    OwnedSimpleType(SimpleTypeDefP0),
    OwnedComplexType(ComplexTypeDefP0),
    ReferencedType(MustResolve<QName>),
    TypeDefinitionOfReferencedElementDeclaration(MustResolve<QName>),
}

pub(crate) struct ElDeclP0TypeTable {
    alternatives: Sequence<TypeAlternativeP0>,
    default_type_definition: TypeAlternativeP0,
}

pub(crate) struct ActualValueWithRespectToEffectiveSimpleType(String);
pub(crate) struct NormalizedValueWithRespectToEffectiveSimpleType(String);
#[repr(transparent)]
pub(crate) struct MustResolve<T>(T);

pub(crate) struct ElDeclP0ValueConstraint {
    variety: ValueConstraintVariety,
    value: ActualValueWithRespectToEffectiveSimpleType,
    lexical_form: NormalizedValueWithRespectToEffectiveSimpleType,
}

impl ElementDeclarationPhase0 {
    /// {name}
    fn map_name(element: Node) -> Result<String, XsdError> {
        // The ·actual value· of the name [attribute].
        Ok(element
            .attribute("name")
            .map(|v| actual_value::<String>(v, element))
            .unwrap())
    }

    /// {type definition}
    fn map_type_definition(
        element: Node,
        schema: Node,
    ) -> Result<ElDeclP0TypeDefinition, XsdError> {
        //   The first of the following that applies:

        //   1 The type definition corresponding to the <simpleType> or
        //     <complexType> element information item in the [children], if
        //     either is present.
        let simple_type_in_children = element
            .children()
            .find(|c| c.tag_name().name() == "simpleType");
        if let Some(simple_type_in_children) = simple_type_in_children {
            return SimpleTypeDefP0::map_from_xml(simple_type_in_children, schema)
                .map(ElDeclP0TypeDefinition::OwnedSimpleType);
        }

        let complex_type_in_children = element
            .children()
            .find(|c| c.tag_name().name() == "complexType");
        if let Some(complex_type_in_children) = complex_type_in_children {
            return ComplexTypeDefP0::map_from_xml(complex_type_in_children)
                .map(ElDeclP0TypeDefinition::OwnedComplexType);
        }

        //   2 The type definition ·resolved· to by the ·actual value· of the
        //     type [attribute], if it is present.
        let type_attrib = element
            .attribute("type")
            .map(|type_| actual_value::<QName>(type_, element));
        if let Some(type_attrib) = type_attrib {
            return Ok(ElDeclP0TypeDefinition::ReferencedType(MustResolve(
                type_attrib,
            )));
        }

        //   3 The declared {type definition} of the Element Declaration
        //     ·resolved· to by the first QName in the ·actual value· of the
        //     substitutionGroup [attribute], if present.
        let first_subst_element_decl = element
            .attribute("substitutionGroup")
            .map(|v| actual_value::<Vec<QName>>(v, element))
            .and_then(|v| v.first().cloned());
        if let Some(first_subst_element_decl) = first_subst_element_decl {
            return Ok(
                ElDeclP0TypeDefinition::TypeDefinitionOfReferencedElementDeclaration(MustResolve(
                    first_subst_element_decl,
                )),
            );
        }

        //   4 ·xs:anyType·.
        Ok(ElDeclP0TypeDefinition::ReferencedType(MustResolve(
            XS_ANY_TYPE_NAME.clone(),
        )))
    }

    /// {type table}
    fn map_type_table(element: Node, schema: Node) -> Result<Option<ElDeclP0TypeTable>, XsdError> {
        //   A Type Table corresponding to the <alternative> element information items among the
        //   [children], if any, as follows, otherwise ·absent·.
        let alternative_elements = element
            .children()
            .filter(|c| c.tag_name().name() == "alternative")
            .collect::<Vec<_>>();

        if !alternative_elements.is_empty() {
            // {alternatives}
            //   A sequence of Type Alternatives, each corresponding, in order, to one of the
            //   <alternative> elements which have a test [attribute].
            let alternatives = alternative_elements
                .iter()
                .filter(|a| a.has_attribute("test"))
                .map(|&a| TypeAlternativeP0::map_from_xml(a, schema))
                .collect::<Result<Sequence<_>, _>>()?;

            // {default type definition}
            //   Depends upon the final <alternative> element among the [children].
            let final_alternative = *alternative_elements.last().unwrap();

            //   If it has no test [attribute], the final <alternative> maps to the {default type
            //   definition}; if it does have a test attribute, it is covered by the rule for
            //   {alternatives} and the {default type definition} is taken from the declared type
            //   of the Element Declaration. So the value of the {default type definition} is given
            //   by the appropriate case among the following:
            let default_type_definition = if !final_alternative.has_attribute("test") {
                // 1 If the <alternative> has no test [attribute], then a Type Alternative
                //   corresponding to the <alternative>.
                TypeAlternativeP0::map_from_xml(final_alternative, schema)?
            } else {
                // 2 otherwise (the <alternative> has a test) a Type Alternative with the following
                //   properties:
                //   {test}             ·absent·.
                //   {type definition}  the {type definition} property of the parent Element
                //                      Declaration.
                //   {annotations}      the empty sequence.
                TypeAlternativeP0::when_alternative_has_test()? // TODO:
            };

            Ok(Some(ElDeclP0TypeTable {
                alternatives,
                default_type_definition,
            }))
        } else {
            Ok(None)
        }
    }

    /// {nillable}
    fn map_nillable(element: Node) -> Result<bool, XsdError> {
        //   The ·actual value· of the nillable [attribute], if present, otherwise false.
        Ok(element
            .attribute("nillable")
            .map(|v| actual_value::<bool>(v, element))
            .unwrap_or(false))
    }

    /// {value constraint}
    fn map_value_constraint(element: Node) -> Result<Option<ElDeclP0ValueConstraint>, XsdError> {
        //   If there is a default or a fixed [attribute], then a Value Constraint as follows,
        //   otherwise ·absent·.
        if element.has_attribute("default") || element.has_attribute("fixed") {
            // [Definition:]
            //   Use the name effective simple type definition for the declared {type definition},
            //   if it is a simple type definition, or, if {type definition}.{content type}
            //   .{variety} = simple, for {type definition}.{content type}.{simple type definition},
            //   or else for the built-in string simple type definition).
            // let _effective_simple_type_definition =
            //     if let TypeDefinition::Simple(st) = type_definition {
            //         st
            //     } else {
            //         let ct = context.request(type_definition.complex().unwrap())?;
            //         if let ContentType::Simple {
            //             simple_type_definition,
            //         } = ct.content_type
            //         {
            //             simple_type_definition
            //         } else {
            //             context.resolve(&XS_STRING_NAME).unwrap() // TODO
            //         }
            //     };

            let (variety, value) = if let Some(default) = element.attribute("default") {
                (ValueConstraintVariety::Default, default)
            } else if let Some(fixed) = element.attribute("fixed") {
                (ValueConstraintVariety::Fixed, fixed)
            } else {
                unreachable!()
            };

            Ok(Some(ElDeclP0ValueConstraint {
                // {variety}: either default or fixed, as appropriate
                variety,
                // {value}: the ·actual value· (with respect to the ·effective simple type definition·)
                //   of the [attribute]
                value: ActualValueWithRespectToEffectiveSimpleType(value.into()),
                // {lexical form}: the ·normalized value· (with respect to the ·effective simple type
                //   definition·) of the [attribute]
                lexical_form: NormalizedValueWithRespectToEffectiveSimpleType(value.into()),
            }))
        } else {
            Ok(None)
        }
    }

    /// {identity-constraint definitions}
    fn map_identity_constraint_definitions(
        element: Node,
        schema: Node,
    ) -> Result<Vec<IdentityConstraintDefinitionP0>, XsdError> {
        //   A set consisting of the identity-constraint-definitions corresponding to all the
        //   <key>, <unique> and <keyref> element information items in the [children], if any,
        //   otherwise the empty set.
        element
            .children()
            .filter(|c| {
                [
                    IdentityConstraintDefinition::KEY_TAG_NAME,
                    IdentityConstraintDefinition::UNIQUE_TAG_NAME,
                    IdentityConstraintDefinition::KEYREF_TAG_NAME,
                ]
                .contains(&c.tag_name().name())
            })
            .map(|icd| IdentityConstraintDefinitionP0::map_from_xml_local(icd, schema))
            .collect::<Result<Vec<_>, _>>()
    }

    /// {substitution group affiliations}
    fn map_substitution_group_affiliations(
        element: Node,
    ) -> Result<MustResolve<Vec<QName>>, XsdError> {
        //   A set of the element declarations ·resolved· to by the items in the ·actual value· of
        //   the substitutionGroup [attribute], if present, otherwise the empty set.
        Ok(MustResolve(
            element
                .attribute("substitutionGroup")
                .map(|v| actual_value::<Vec<QName>>(v, element))
                .unwrap_or_default(),
        ))
    }

    /// {disallowed substitutions}
    fn map_disallowed_substitutions(
        element: Node,
        schema: Node,
    ) -> Result<Vec<SubstitutionMethod>, XsdError> {
        // (see the helper function for explanation)
        Ok(ElementDeclaration::map_attrib_set_helper(
            "block",
            "blockDefault",
            &[
                SubstitutionMethod::Extension,
                SubstitutionMethod::Restriction,
                SubstitutionMethod::Substitution,
            ],
            element,
            schema,
        ))
    }

    /// {substitution group exclusions}
    fn map_substitution_group_exclusions(
        element: Node,
        schema: Node,
    ) -> Result<Vec<complex_type_def::DerivationMethod>, XsdError> {
        // As for {disallowed substitutions} above, but using the final and finalDefault
        // [attributes] in place of the block and blockDefault [attributes] and with the relevant
        // set being {extension, restriction}.
        Ok(ElementDeclaration::map_attrib_set_helper(
            "final",
            "finalDefault",
            &[
                complex_type_def::DerivationMethod::Extension,
                complex_type_def::DerivationMethod::Restriction,
            ],
            element,
            schema,
        ))
    }

    /// {abstract}
    fn map_abstract(element: Node) -> Result<bool, XsdError> {
        //   The ·actual value· of the abstract [attribute], if present, otherwise false.
        Ok(element
            .attribute("abstract")
            .map(|v| actual_value::<bool>(v, element))
            .unwrap_or(false))
    }

    /// {annotations}
    fn map_annotations(element: Node) -> Result<Vec<Annotation>, XsdError> {
        let mut annot_elements = vec![element];
        annot_elements.extend(
            element
                .children()
                .filter(|e| ["unique", "key", "keyref"].contains(&e.tag_name().name()))
                .filter(|e| e.has_attribute("ref")),
        );
        // TODO: map annotations: Annotation::xml_element_set_annotation_mapping(context, &annot_elements);
        eprintln!("TODO: P0 map annotations");
        Ok(vec![])
    }

    fn map_from_xml_common(element: Node, schema: Node) -> Result<Self, XsdError> {
        Ok(ElementDeclarationPhase0 {
            name: Self::map_name(element)?,
            type_definition: Self::map_type_definition(element, schema)?,
            type_table: Self::map_type_table(element, schema)?,
            nillable: Self::map_nillable(element)?,
            value_constraint: Self::map_value_constraint(element)?,
            identity_constraint_definitions: Self::map_identity_constraint_definitions(
                element, schema,
            )?,
            substitution_group_affiliations: Self::map_substitution_group_affiliations(element)?,
            disallowed_substitutions: Self::map_disallowed_substitutions(element, schema)?,
            substitution_group_exclusions: Self::map_substitution_group_exclusions(
                element, schema,
            )?,
            abstract_: Self::map_abstract(element)?,
            annotations: Self::map_annotations(element)?,
        })
    }
}

impl ElementDeclaration {
    pub const TAG_NAME: &'static str = "element";

    fn map_from_xml_common(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        element: Node,
        schema: Node,
    ) -> Result<Self, XsdError> {
        // NOTE: For now, get_name_from_xml() can't be used as the common case doesn't handle the
        //       target namespace

        let this_p0 = ElementDeclarationPhase0::map_from_xml_common(element, schema)?;

        let name = this_p0.name;

        let type_definition: TypeDefinition = match this_p0.type_definition {
            ElDeclP0TypeDefinition::OwnedSimpleType(simple_type_def_p0) => {
                todo!()
            }
            ElDeclP0TypeDefinition::OwnedComplexType(complex_type_def_p0) => todo!(),
            ElDeclP0TypeDefinition::ReferencedType(type_) => {
                // TODO: unwrap
                context.resolve(&type_.0).unwrap()
            }
            ElDeclP0TypeDefinition::TypeDefinitionOfReferencedElementDeclaration(name) => {
                // TODO: unwrap
                let element_decl: Ref<ElementDeclaration> = context.resolve(&name.0).unwrap();

                // TODO: unwrap
                context.request(element_decl).unwrap().type_definition
            }
        };

        // {type table}
        //   A Type Table corresponding to the <alternative> element information items among the
        //   [children], if any, as follows, otherwise ·absent·.
        let alternative_elements = element
            .children()
            .filter(|c| c.tag_name().name() == "alternative")
            .collect::<Vec<_>>();

        let type_table = if !alternative_elements.is_empty() {
            // {alternatives}
            //   A sequence of Type Alternatives, each corresponding, in order, to one of the
            //   <alternative> elements which have a test [attribute].
            let alternatives = alternative_elements
                .iter()
                .filter(|a| a.has_attribute("test"))
                .map(|&a| TypeAlternative::map_from_xml(a, schema))
                .collect::<Sequence<_>>();

            // {default type definition}
            //   Depends upon the final <alternative> element among the [children].
            let final_alternative = *alternative_elements.last().unwrap();

            //   If it has no test [attribute], the final <alternative> maps to the {default type
            //   definition}; if it does have a test attribute, it is covered by the rule for
            //   {alternatives} and the {default type definition} is taken from the declared type
            //   of the Element Declaration. So the value of the {default type definition} is given
            //   by the appropriate case among the following:
            let default_type_definition = if !final_alternative.has_attribute("test") {
                // 1 If the <alternative> has no test [attribute], then a Type Alternative
                //   corresponding to the <alternative>.
                TypeAlternative::map_from_xml(final_alternative, schema)
            } else {
                // 2 otherwise (the <alternative> has a test) a Type Alternative with the following
                //   properties:
                //   {test}             ·absent·.
                //   {type definition}  the {type definition} property of the parent Element
                //                      Declaration.
                //   {annotations}      the empty sequence.
                context.create(TypeAlternative {
                    test: None,
                    type_definition,
                    annotations: Sequence::new(),
                })
            };

            Some(TypeTable {
                alternatives,
                default_type_definition,
            })
        } else {
            None
        };

        let nillable = this_p0.nillable;

        // {value constraint}
        //   If there is a default or a fixed [attribute], then a Value Constraint as follows,
        //   otherwise ·absent·.
        let value_constraint = this_p0.value_constraint.map(|vc| ValueConstraint {
            variety: vc.variety,
            value: vc.value.0,               // TODO: actual value
            lexical_form: vc.lexical_form.0, // TODO: normalized value
        });

        // {identity-constraint definitions}
        //   A set consisting of the identity-constraint-definitions corresponding to all the
        //   <key>, <unique> and <keyref> element information items in the [children], if any,
        //   otherwise the empty set.
        let identity_constraint_definitions = element
            .children()
            .filter(|c| {
                [
                    IdentityConstraintDefinition::KEY_TAG_NAME,
                    IdentityConstraintDefinition::UNIQUE_TAG_NAME,
                    IdentityConstraintDefinition::KEYREF_TAG_NAME,
                ]
                .contains(&c.tag_name().name())
            })
            .map(|icd| IdentityConstraintDefinition::map_from_xml_local(context, icd, schema))
            .collect::<Result<Vec<_>, _>>()?;

        // {substitution group affiliations}
        //   A set of the element declarations ·resolved· to by the items in the ·actual value· of
        //   the substitutionGroup [attribute], if present, otherwise the empty set.
        let substitution_group_affiliations = this_p0
            .substitution_group_affiliations
            .0
            .into_iter()
            .map(|c| context.resolve(&c).unwrap()) // TODO: unwrap
            .collect();

        // {disallowed substitutions} (see the helper function for explanation)
        let disallowed_substitutions = this_p0.disallowed_substitutions;

        // As for {disallowed substitutions} above, but using the final and finalDefault
        // [attributes] in place of the block and blockDefault [attributes] and with the relevant
        // set being {extension, restriction}.
        let substitution_group_exclusions = this_p0.substitution_group_exclusions;

        // {abstract}
        //   The ·actual value· of the abstract [attribute], if present, otherwise false.
        let abstract_ = this_p0.abstract_;

        // {annotations}
        //   The ·annotation mapping· of the <element> element and any of its <unique>, <key> and
        //   <keyref> [children] with a ref [attribute], as defined in XML Representation of
        //   Annotation Schema Components (§3.15.2).
        let annotations = this_p0
            .annotations
            .into_iter()
            .map(|a| context.create(a))
            .collect();

        Ok(Self {
            annotations,
            name,
            type_definition,
            type_table,
            value_constraint,
            nillable,
            identity_constraint_definitions,
            substitution_group_affiliations,
            substitution_group_exclusions,
            disallowed_substitutions,
            abstract_,

            // Populated by the specific implementations below
            target_namespace: None,
            scope: Scope::new_global(),
        })
    }

    /// Maps the [`ElementDeclaration`] from an `<element>` without a `ref` attribute.
    fn map_local_element_decl(
        context: &mut MappingContext,
        element: Node,
        schema: Node,
        parent: ScopeParent,
    ) -> Result<Ref<Self>, XsdError> {
        let self_ref = context.reserve();

        // {target namespace} The appropriate case among the following:
        let target_namespace = if let Some(target_namespace) = schema.attribute("targetNamespace") {
            // 1 If targetNamespace is present , then its ·actual value·.
            Some(actual_value::<String>(target_namespace, element))
        } else {
            // 2 If targetNamespace is not present and one of the following is true
            // 2.1 form = qualified
            // 2.2 form is absent and the <schema> ancestor has elementFormDefault = qualified
            let form = element
                .attribute("form")
                .or_else(|| schema.attribute("elementFormDefault"));
            if form == Some("qualified") {
                // then the ·actual value· of the targetNamespace [attribute] of the ancestor
                // <schema> element information item, or ·absent· if there is none.
                schema
                    .attribute("targetNamespace")
                    .map(|v| actual_value::<String>(v, element))
            } else {
                // 3 otherwise ·absent·.
                None
            }
        };

        // {scope} A Scope as follows:
        //    {variety} local
        //    {parent}  If the <element> element information item has <complexType> as an ancestor,
        //              the Complex Type Definition corresponding to that item, otherwise (the
        //              <element> element information item is within a named <group> element
        //              information item), the Model Group Definition corresponding to that item.
        let scope = Scope::new_local(parent);

        let common = Self::map_from_xml_common(context, self_ref, element, schema)?;

        Ok(context.insert(
            self_ref,
            Self {
                target_namespace,
                scope,
                ..common
            },
        ))
    }

    pub(super) fn map_from_xml_local(
        context: &mut MappingContext,
        element: Node,
        schema: Node,
        parent: ScopeParent,
    ) -> Result<Ref<Particle>, XsdError> {
        assert_eq!(element.tag_name().name(), "element");
        // FIXME: minOccurs=maxOccurs=0 shouldn't create anything

        let element_decl = if let Some(ref_) = element.attribute("ref") {
            // If the <element> element information item has <complexType> or <group> as an
            // ancestor, and the ref [attribute] is present, and it does not have
            // minOccurs=maxOccurs=0, then it maps to a Particle as follows.

            // {term}
            //   The (top-level) element declaration ·resolved· to by the ·actual value· of the ref
            //   [attribute].
            let ref_: QName = actual_value(ref_, element);
            context.resolve(&ref_).unwrap() // TODO
        } else {
            // If the <element> element information item has <complexType> or <group> as an
            // ancestor, and the ref [attribute] is absent, and it does not have
            // minOccurs=maxOccurs=0, then it maps both to a Particle and to a local Element
            // Declaration which is the {term} of that Particle.

            // {term}
            //   A (local) element declaration as given below.
            Self::map_local_element_decl(context, element, schema, parent)?
        };
        let term = Term::ElementDeclaration(element_decl);

        // These properties are common to both ref and non-ref elements:

        // {min occurs}
        //   The ·actual value· of the minOccurs [attribute], if present, otherwise 1.
        let min_occurs = element
            .attribute("minOccurs")
            .map(|min_occurs| actual_value::<u64>(min_occurs, element))
            .unwrap_or(1);

        // {max occurs}
        //   unbounded, if the maxOccurs [attribute] equals unbounded, otherwise the ·actual
        //   value· of the maxOccurs [attribute], if present, otherwise 1.
        let max_occurs = element
            .attribute("maxOccurs")
            .map(|max_occurs| {
                if max_occurs == "unbounded" {
                    MaxOccurs::Unbounded
                } else {
                    MaxOccurs::Count(actual_value::<u64>(max_occurs, element))
                }
            })
            .unwrap_or(MaxOccurs::Count(1));

        // {annotations}
        //   The same annotations as the {annotations} of the {term}.
        // NOTE: These are provided on-demand by the `annotations` method on Particle.
        let annotations = None;

        Ok(context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        }))
    }

    fn map_attrib_set_helper<'a, T: ActualValue<'a> + PartialEq + Copy>(
        local_attrib: &str,
        default_attrib: &str,
        relevant_set: &[T],
        element: Node<'a, 'a>,
        schema: Node<'a, 'a>,
    ) -> Set<T> {
        // Comment text is from {disallowed substitutions}, but this applies to {substitution group
        // exclusions} as well

        // A set depending on the ·actual value· of the block [attribute], if present, otherwise on
        // the ·actual value· of the blockDefault [attribute] of the ancestor <schema> element
        // information item, if present, otherwise on the empty string.
        // Call this the EBV (for effective block value).
        let effective_value = element
            .attribute(local_attrib)
            .or_else(|| schema.attribute(default_attrib))
            .unwrap_or_default();

        // Then the value of this property is the appropriate case among the following:
        if effective_value.is_empty() {
            // 1 If the EBV is the empty string, then the empty set;
            Set::new()
        } else if effective_value == "#all" {
            // 2 If the EBV is #all, then {extension, restriction, substitution};
            relevant_set.to_vec()
        } else {
            // otherwise a set with members drawn from the set above, each being present or absent
            // depending on whether the ·actual value· (which is a list) contains an equivalently
            // named item.
            let effective_block_value = actual_value::<Vec<T>>(effective_value, element);
            relevant_set
                .iter()
                .filter(|m| effective_block_value.contains(m))
                .copied()
                .collect()
        }
    }
}

impl Component for ElementDeclaration {
    const DISPLAY_NAME: &'static str = "ElementDeclaration";
}

impl Named for ElementDeclaration {
    fn name(&self) -> Option<QName> {
        Some(QName::with_optional_namespace(
            self.target_namespace.as_ref(),
            &self.name,
        ))
    }
}

impl TopLevelMappable for ElementDeclaration {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        element: Node,
        schema: Node,
    ) -> Result<(), XsdError> {
        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the parent <schema> element
        //   information item, or ·absent· if there is none.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, element));

        // {scope} A Scope as follows:
        //   {variety} global
        //   {parent}  ·absent·
        let scope = Scope::new_global();

        let common = Self::map_from_xml_common(context, self_ref, element, schema)?;

        context.insert(
            self_ref,
            Self {
                target_namespace,
                scope,
                ..common
            },
        );
        Ok(())
    }
}
