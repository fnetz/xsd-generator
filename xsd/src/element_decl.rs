use super::simple_type_def::Context as SimpleContext;
use super::{
    annotation::Annotation,
    builtins::{XS_ANY_TYPE_NAME, XS_STRING_NAME},
    complex_type_def::{self, ComplexTypeDefinition, ContentTypeVariety},
    components::{Component, Named, NamedXml},
    identity_constraint_def::IdentityConstraintDefinition,
    mapping_context::TopLevelMappable,
    model_group_def::ModelGroupDefinition,
    particle::MaxOccurs,
    shared::{self, TypeDefinition},
    type_alternative::TypeAlternative,
    values::{actual_value, ActualValue},
    xstypes::{AnyURI, NCName, QName, Sequence, Set},
    MappingContext, Particle, Ref, SimpleTypeDefinition, Term,
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

#[derive(Clone, Debug)]
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

impl ElementDeclaration {
    pub const TAG_NAME: &'static str = "element";

    fn map_from_xml_common(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        element: Node,
        schema: Node,
    ) -> Self {
        // NOTE: For now, get_name_from_xml() can't be used as the common case doesn't handle the
        //       target namespace

        // {name} The ·actual value· of the name [attribute].
        let name = element
            .attribute("name")
            .map(|v| actual_value::<String>(v, element))
            .unwrap();

        // {type definition}
        //   The first of the following that applies:
        //   1 The type definition corresponding to the <simpleType> or
        //     <complexType> element information item in the [children], if
        //     either is present.
        //   2 The type definition ·resolved· to by the ·actual value· of the
        //     type [attribute], if it is present.
        //   3 The declared {type definition} of the Element Declaration
        //     ·resolved· to by the first QName in the ·actual value· of the
        //     substitutionGroup [attribute], if present.
        //   4 ·xs:anyType·.
        let type_definition = element
            .children()
            .find(|c| c.tag_name().name() == "simpleType")
            .map(|simple_type| {
                let simple_type_def = SimpleTypeDefinition::map_from_xml(
                    context,
                    simple_type,
                    schema,
                    None,
                    Some(SimpleContext::Element(self_ref)),
                );
                TypeDefinition::Simple(simple_type_def)
            })
            .or_else(|| {
                element
                    .children()
                    .find(|c| c.tag_name().name() == "complexType")
                    .map(|complex_type| {
                        let complex_type_def = ComplexTypeDefinition::map_from_xml(
                            context,
                            complex_type,
                            schema,
                            Some(self_ref),
                            None,
                        );
                        TypeDefinition::Complex(complex_type_def)
                    })
            })
            .or_else(|| {
                element
                    .attribute("type")
                    .map(|type_| context.resolve(&actual_value::<QName>(type_, element)))
            })
            .or_else(|| {
                element
                    .attribute("substitutionGroup")
                    .map(|v| actual_value::<Vec<QName>>(v, element))
                    .and_then(|v| v.first().cloned())
                    .map(|name| context.resolve::<Ref<ElementDeclaration>>(&name))
                    .map(|element_decl| context.request(element_decl).type_definition)
            })
            .unwrap_or_else(|| context.resolve(&XS_ANY_TYPE_NAME));

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

        // {nillable}
        //   The ·actual value· of the nillable [attribute], if present, otherwise false.
        let nillable = element
            .attribute("nillable")
            .map(|v| actual_value::<bool>(v, element))
            .unwrap_or(false);

        // {value constraint}
        //   If there is a default or a fixed [attribute], then a Value Constraint as follows,
        //   otherwise ·absent·.
        let value_constraint = if element.has_attribute("default") || element.has_attribute("fixed")
        {
            // [Definition:]
            //   Use the name effective simple type definition for the declared {type definition},
            //   if it is a simple type definition, or, if {type definition}.{content type}
            //   .{variety} = simple, for {type definition}.{content type}.{simple type definition},
            //   or else for the built-in string simple type definition).
            // TODO store as the effective type
            let _effective_simple_type_definition =
                if let TypeDefinition::Simple(st) = type_definition {
                    st
                } else {
                    let ct = context.request(type_definition.complex().unwrap());
                    if ct.content_type.variety == ContentTypeVariety::Simple {
                        ct.content_type.simple_type_definition.unwrap()
                    } else {
                        context.resolve(&XS_STRING_NAME)
                    }
                };

            let (variety, value) = if let Some(default) = element.attribute("default") {
                (ValueConstraintVariety::Default, default)
            } else if let Some(fixed) = element.attribute("fixed") {
                (ValueConstraintVariety::Fixed, fixed)
            } else {
                unreachable!()
            };

            Some(ValueConstraint {
                variety,
                value: value.into(),
                lexical_form: value.into(),
            })
        } else {
            None
        };

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
            .collect();

        // {substitution group affiliations}
        //   A set of the element declarations ·resolved· to by the items in the ·actual value· of
        //   the substitutionGroup [attribute], if present, otherwise the empty set.
        let substitution_group_affiliations = element
            .attribute("substitutionGroup")
            .map(|v| actual_value::<Vec<QName>>(v, element))
            .map(|v| v.iter().map(|c| context.resolve(c)).collect())
            .unwrap_or_default();

        // {disallowed substitutions} (see the helper function for explanation)
        let disallowed_substitutions = Self::map_attrib_set_helper(
            "block",
            "blockDefault",
            &[
                SubstitutionMethod::Extension,
                SubstitutionMethod::Restriction,
                SubstitutionMethod::Substitution,
            ],
            element,
            schema,
        );

        // As for {disallowed substitutions} above, but using the final and finalDefault
        // [attributes] in place of the block and blockDefault [attributes] and with the relevant
        // set being {extension, restriction}.
        let substitution_group_exclusions = Self::map_attrib_set_helper(
            "final",
            "finalDefault",
            &[
                complex_type_def::DerivationMethod::Extension,
                complex_type_def::DerivationMethod::Restriction,
            ],
            element,
            schema,
        );

        // {abstract}
        //   The ·actual value· of the abstract [attribute], if present, otherwise false.
        let abstract_ = element
            .attribute("abstract")
            .map(|v| actual_value::<bool>(v, element))
            .unwrap_or(false);

        // {annotations}
        //   The ·annotation mapping· of the <element> element and any of its <unique>, <key> and
        //   <keyref> [children] with a ref [attribute], as defined in XML Representation of
        //   Annotation Schema Components (§3.15.2).
        let mut annot_elements = vec![element];
        element
            .children()
            .filter(|e| ["unique", "key", "keyref"].contains(&e.tag_name().name()))
            .filter(|e| e.has_attribute("ref"))
            .for_each(|e| annot_elements.push(e));
        let annotations = Annotation::xml_element_set_annotation_mapping(context, &annot_elements);

        Self {
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
        }
    }

    /// Maps the [`ElementDeclaration`] from an `<element>` without a `ref` attribute.
    fn map_local_element_decl(
        context: &mut MappingContext,
        element: Node,
        schema: Node,
        parent: ScopeParent,
    ) -> Ref<Self> {
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

        let common = Self::map_from_xml_common(context, self_ref, element, schema);

        context.insert(
            self_ref,
            Self {
                target_namespace,
                scope,
                ..common
            },
        );
        self_ref
    }

    pub(super) fn map_from_xml_local(
        context: &mut MappingContext,
        element: Node,
        schema: Node,
        parent: ScopeParent,
    ) -> Ref<Particle> {
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
            context.resolve(&ref_)
        } else {
            // If the <element> element information item has <complexType> or <group> as an
            // ancestor, and the ref [attribute] is absent, and it does not have
            // minOccurs=maxOccurs=0, then it maps both to a Particle and to a local Element
            // Declaration which is the {term} of that Particle.

            // {term}
            //   A (local) element declaration as given below.
            Self::map_local_element_decl(context, element, schema, parent)
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
        let annotations = match term {
            Term::ElementDeclaration(element) => context.request(element).annotations.clone(),
            _ => unreachable!(),
        };

        context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        })
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
    ) {
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

        let common = Self::map_from_xml_common(context, self_ref, element, schema);

        context.insert(
            self_ref,
            Self {
                target_namespace,
                scope,
                ..common
            },
        );
    }
}
