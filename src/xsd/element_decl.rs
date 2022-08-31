use super::{
    annotation::Annotation,
    complex_type_def::{self, ComplexTypeDefinition},
    identity_constraint_def::IdentityConstraintDefinition,
    model_group_def::ModelGroupDefinition,
    shared::{self, TypeDefinition},
    type_alternative::TypeAlternative,
    values::actual_value,
    xstypes::{AnyURI, NCName, QName, Sequence, Set},
    MappingContext, Ref, RefVisitor, RefsVisitable, Resolution, SimpleTypeDefinition,
};
use roxmltree::Node;

/// Schema Component: Element Declaration, a kind of [Term](super::shared::Term) (§3.3)
#[derive(Clone, Debug)]
pub struct ElementDeclaration {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub type_definition: Ref<TypeDefinition>,
    pub type_table: Option<TypeTable>,
    pub scope: Scope,
    pub value_constraint: Option<ValueConstraint>,
    pub nillable: bool,
    pub identity_constraint_definitions: Set<Ref<IdentityConstraintDefinition>>,
    pub substitution_group_affiliations: Set<Ref<ElementDeclaration>>,
    pub substitution_group_exclusions: Set<GroupExlusion>,
    pub disallowed_substitutions: Set<SubstitutionMethods>,
    pub abstract_: bool,
}

pub type GroupExlusion = complex_type_def::DerivationMethod;

#[derive(Clone, Debug)]
pub enum SubstitutionMethods {
    Substitution,
    Extension,
    Restriction,
}

/// Property Record: Type Table (§3.3)
#[derive(Clone, Debug)]
pub struct TypeTable {
    pub alternatives: Sequence<Ref<TypeAlternative>>,
    pub default_type_definition: Ref<TypeAlternative>,
}

/// Property Record: Scope (§3.3)
#[derive(Clone, Debug)]
pub struct Scope {
    pub variety: ScopeVariety,
    pub parent: Option<ScopeParent>,
}

pub use shared::ScopeVariety;

#[derive(Clone, Debug)]
pub enum ScopeParent {
    ComplexType(Ref<ComplexTypeDefinition>),
    Group(Ref<ModelGroupDefinition>),
}

/// Property Record: Value Constraint (§3.3)
pub type ValueConstraint = shared::ValueConstraint;

impl ElementDeclaration {
    fn map_from_xml_common(context: &mut MappingContext, element: Node, schema: Node) -> Self {
        // {name}
        //   The ·actual value· of the name [attribute].
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
                let simple_type_def =
                    SimpleTypeDefinition::map_from_xml(context, simple_type, schema);
                context
                    .components
                    .create(TypeDefinition::Simple(simple_type_def))
            })
            .or_else(|| {
                element
                    .children()
                    .find(|c| c.tag_name().name() == "complexType")
                    .map(|complex_type| {
                        let complex_type_def =
                            ComplexTypeDefinition::map_from_xml(context, complex_type, schema);
                        context
                            .components
                            .create(TypeDefinition::Complex(complex_type_def))
                    })
            })
            .or_else(|| {
                element.attribute("type").map(|type_| {
                    context.components.resolve_type_def(
                        &actual_value::<QName>(type_, element),
                        Resolution::Deferred,
                    )
                })
            })
            .or_else(|| {
                element
                    .attribute("substitutionGroup")
                    .map(|v| actual_value::<Vec<QName>>(v, element))
                    .and_then(|v| v.first().cloned())
                    .map(|name| {
                        context
                            .components
                            .resolve_element_declaration(&name, Resolution::Deferred)
                    })
                    .map(|_element_decl| todo!("element_decl.type_definition"))
            })
            .unwrap_or_else(|| todo!("xs:anyType"));

        // {type table}
        //   A Type Table corresponding to the <alternative> element information
        //   items among the [children], if any, as follows, otherwise ·absent·.
        let alternative_elements = element
            .children()
            .filter(|c| c.tag_name().name() == "alternative")
            .collect::<Vec<_>>();

        let type_table = if !alternative_elements.is_empty() {
            // {alternatives}
            //   A sequence of Type Alternatives, each corresponding, in order, to
            //   one of the <alternative> elements which have a test [attribute].
            let alternatives = alternative_elements
                .iter()
                .filter(|a| a.has_attribute("test"))
                .map(|&a| TypeAlternative::map_from_xml(a, schema))
                .collect::<Sequence<_>>();

            // {default type definition}
            //   Depends upon the final <alternative> element among the
            //   [children].
            let final_alternative = *alternative_elements.last().unwrap();

            //   If it has no test [attribute], the final <alternative> maps to
            //   the {default type definition}; if it does have a test
            //   attribute, it is covered by the rule for {alternatives} and the
            //   {default type definition} is taken from the declared type of
            //   the Element Declaration. So the value of the {default type
            //   definition} is given by the appropriate case among the
            //   following:
            let default_type_definition = if !final_alternative.has_attribute("test") {
                // 1 If the <alternative> has no test [attribute], then a Type
                //   Alternative corresponding to the <alternative>.
                TypeAlternative::map_from_xml(final_alternative, schema)
            } else {
                // 2 otherwise (the <alternative> has a test) a Type Alternative
                //   with the following properties:
                //   {test}
                //     ·absent·.
                //   {type definition}
                //     the {type definition} property of the parent Element Declaration.
                //   {annotations}
                //     the empty sequence.
                context.components.create(TypeAlternative {
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
        //   The ·actual value· of the nillable [attribute], if present,
        //   otherwise false.
        let nillable = element
            .attribute("nillable")
            .map(|v| actual_value::<bool>(v, element))
            .unwrap_or(false);

        // {value constraint}
        //   If there is a default or a fixed [attribute], then a Value
        //   Constraint as follows, otherwise ·absent·. [Definition:]  Use the
        //   name effective simple type definition for the declared {type
        //   definition}, if it is a simple type definition, or, if {type
        //   definition}.{content type}.{variety} = simple, for {type
        //   definition}.{content type}.{simple type definition}, or else for
        //   the built-in string simple type definition).
        // TODO
        let value_constraint = None;

        // {identity- constraint definitions}
        //   A set consisting of the identity-constraint-definitions
        //   corresponding to all the <key>, <unique> and <keyref> element
        //   information items in the [children], if any, otherwise the empty
        //   set.
        // TODO
        let identity_constraint_definitions = Set::new();

        // {substitution group affiliations}
        //   A set of the element declarations ·resolved· to by the items in the
        //   ·actual value· of the substitutionGroup [attribute], if present,
        //   otherwise the empty set.
        // TODO
        let substitution_group_affiliations = Set::new();

        // TODO impl, doc
        let disallowed_substitutions = Set::new();

        // TODO same
        let substitution_group_exclusions = Set::new();

        // {abstract}
        //   The ·actual value· of the abstract [attribute], if present,
        //   otherwise false.
        let abstract_ = element
            .attribute("abstract")
            .map(|v| actual_value::<bool>(v, element))
            .unwrap_or(false);

        // {annotations}
        //   The ·annotation mapping· of the <element> element and any of its
        //   <unique>, <key> and <keyref> [children] with a ref [attribute], as
        //   defined in XML Representation of Annotation Schema Components
        //   (§3.15.2).
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
            scope: Scope {
                variety: ScopeVariety::Global,
                parent: None,
            },
        }
    }

    pub fn map_from_xml_top_level(
        context: &mut MappingContext,
        element: Node,
        schema: Node,
    ) -> Ref<Self> {
        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the parent
        //   <schema> element information item, or ·absent· if there is none.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, element));

        // {scope}
        //   A Scope as follows:
        let scope = Scope {
            // {variety}
            //   global
            variety: ScopeVariety::Global,
            // {parent}
            //   ·absent·
            parent: None,
        };

        let common = Self::map_from_xml_common(context, element, schema);

        context.components.create(Self {
            target_namespace,
            scope,
            ..common
        })
    }

    pub fn map_from_xml_local(
        context: &mut MappingContext,
        element: Node,
        schema: Node,
    ) -> Ref<Self> {
        // {target namespace}
        //   The appropriate case among the following:
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
        let scope = Scope {
            // {variety} local
            variety: ScopeVariety::Local,
            // {parent}  If the <element> element information item has <complexType> as an
            //           ancestor, the Complex Type Definition corresponding to that item,
            //           otherwise (the <element> element information item is within a named
            //           <group> element information item), the Model Group Definition
            //           corresponding to that item.
            parent: None, // TODO
        };

        let common = Self::map_from_xml_common(context, element, schema);

        context.components.create(Self {
            target_namespace,
            scope,
            ..common
        })
    }
}

impl RefsVisitable for ElementDeclaration {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
        visitor.visit_ref(&mut self.type_definition);
        if let Some(ref mut type_table) = self.type_table {
            type_table
                .alternatives
                .iter_mut()
                .for_each(|alternative| visitor.visit_ref(alternative));
            visitor.visit_ref(&mut type_table.default_type_definition);
        }
        if let Some(ref mut scope_parent) = self.scope.parent {
            match scope_parent {
                ScopeParent::ComplexType(ref mut complex_type) => {
                    visitor.visit_ref(complex_type);
                }
                ScopeParent::Group(ref mut group) => {
                    visitor.visit_ref(group);
                }
            }
        }
        self.identity_constraint_definitions.iter_mut().for_each(
            |identity_constraint_definition| {
                visitor.visit_ref(identity_constraint_definition);
            },
        );
        self.substitution_group_affiliations.iter_mut().for_each(
            |substitution_group_affiliation| {
                visitor.visit_ref(substitution_group_affiliation);
            },
        );
    }
}
