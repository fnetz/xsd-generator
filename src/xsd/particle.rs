use super::{
    annotation::Annotation, components::Component, element_decl, model_group::Compositor,
    shared::Term, values::actual_value, xstypes::Sequence, ComplexTypeDefinition,
    ElementDeclaration, MappingContext, ModelGroup, Ref, Wildcard,
};
use roxmltree::Node;

/// Schema Component: Particle, a kind of Component (§3.9)
#[derive(Clone, Debug)]
pub struct Particle {
    pub min_occurs: u64, // TODO nonNegativeInteger
    pub max_occurs: MaxOccurs,
    pub term: Term,
    pub annotations: Sequence<Ref<Annotation>>,
}

#[derive(Clone, Debug)]
pub enum MaxOccurs {
    Unbounded,
    Count(u64), // TODO NonZeroU64
}

impl Particle {
    pub(super) fn map_from_xml_local_element(
        context: &mut MappingContext,
        particle: Node,
        schema: Node,
        element_parent: Ref<ComplexTypeDefinition>,
    ) -> Ref<Self> {
        assert_eq!(particle.tag_name().name(), "element");

        // {min occurs}
        //   The ·actual value· of the minOccurs [attribute], if present, otherwise 1.
        let min_occurs = particle
            .attribute("minOccurs")
            .map(|min_occurs| actual_value::<u64>(min_occurs, particle))
            .unwrap_or(1);

        // {max occurs}
        //   unbounded, if the maxOccurs [attribute] equals unbounded, otherwise the ·actual value·
        //   of the maxOccurs [attribute], if present, otherwise 1.
        let max_occurs = particle
            .attribute("maxOccurs")
            .map(|max_occurs| {
                if max_occurs == "unbounded" {
                    MaxOccurs::Unbounded
                } else {
                    MaxOccurs::Count(actual_value::<u64>(max_occurs, particle))
                }
            })
            .unwrap_or(MaxOccurs::Count(1));

        // {term}
        //   A (local) element declaration as given below.
        let element = ElementDeclaration::map_from_xml_local(
            context,
            particle,
            schema,
            element_decl::ScopeParent::ComplexType(element_parent),
        );
        let term = Term::ElementDeclaration(element);

        // {annotations}
        //   The same annotations as the {annotations} of the {term}.
        let annotations = match term {
            Term::ElementDeclaration(element) => {
                element.get(context.components()).annotations.clone()
            }
            _ => unreachable!(),
        };

        context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        })
    }

    /// Mapper for Model groups <all>, <sequence>, and <choice>, see XML Representation of Model
    /// Group Schema Components (§3.8.2)
    pub(super) fn map_from_xml_model_group(
        context: &mut MappingContext,
        particle: Node,
        schema: Node,
        element_parent: Ref<ComplexTypeDefinition>,
    ) -> Ref<Self> {
        assert!(matches!(
            particle.tag_name().name(),
            "all" | "choice" | "sequence"
        ));

        // {min occurs}
        //   The ·actual value· of the minOccurs [attribute], if present, otherwise 1.
        let min_occurs = particle
            .attribute("minOccurs")
            .map(|min_occurs| actual_value::<u64>(min_occurs, particle))
            .unwrap_or(1);

        // {max occurs}
        //   unbounded, if the maxOccurs [attribute] equals unbounded, otherwise the ·actual value·
        //   of the maxOccurs [attribute], if present, otherwise 1.
        let max_occurs = particle
            .attribute("maxOccurs")
            .map(|max_occurs| {
                if max_occurs == "unbounded" {
                    MaxOccurs::Unbounded
                } else {
                    MaxOccurs::Count(actual_value::<u64>(max_occurs, particle))
                }
            })
            .unwrap_or(MaxOccurs::Count(1));

        // {annotations}
        //   The ·annotation mapping· of the <all>, <choice>, or <sequence> element, whichever
        //   is present, as defined in XML Representation of Annotation Schema Components
        //   (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, particle);

        // {term} A model group as given below.
        let term = Term::ModelGroup({
            // {compositor}
            //   One of all, choice, sequence depending on the element information item.
            let compositor = match particle.tag_name().name() {
                "all" => Compositor::All,
                "choice" => Compositor::Choice,
                "sequence" => Compositor::Sequence,
                _ => unreachable!(),
            };
            // {particles}
            //   A sequence of particles corresponding to all the <all>, <choice>, <sequence>,
            //   <any>, <group> or <element> items among the [children], in order.
            let particles = particle
                .children()
                .filter_map(|child| match child.tag_name().name() {
                    "all" | "choice" | "sequence" => Some(Self::map_from_xml_model_group(
                        context,
                        child,
                        schema,
                        element_parent,
                    )),
                    "any" => Some(Particle::map_from_xml_wildcard_any(context, child, schema)),
                    "group" => Some(Particle::map_from_xml_group_reference(context, child)),
                    "element" => Some(Particle::map_from_xml_local_element(
                        context,
                        child,
                        schema,
                        element_parent,
                    )),
                    _ => None,
                })
                .collect();

            context.create(ModelGroup {
                compositor,
                particles,
                annotations: annotations.clone(),
            })
        });

        // {annotations}
        //   The same annotations as the {annotations} of the model group.
        // -- created above --

        context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        })
    }

    /// Mapper for Group references <group>, see XML Representation of Model Group Definition
    /// Schema Components (§3.7.2)
    pub(super) fn map_from_xml_group_reference(
        _context: &mut MappingContext,
        group: Node,
    ) -> Ref<Particle> {
        assert_eq!(group.tag_name().name(), "group");
        todo!()
    }

    // TODO anyAttribute

    /// Mapper for Wildcard <any>, see XML Representation of Wildcard Schema Components (§3.10.2)
    pub(super) fn map_from_xml_wildcard_any(
        context: &mut MappingContext,
        any: Node,
        schema: Node,
    ) -> Ref<Self> {
        // TODO handle minOccurs=maxOccurs=0
        assert_eq!(any.tag_name().name(), "any");

        let wildcard = Wildcard::map_from_xml_any(context, any, schema);

        // The ·actual value· of the minOccurs [attribute], if present, otherwise 1.
        let min_occurs = any
            .attribute("minOccurs")
            .map(|v| actual_value::<u64>(v, any))
            .unwrap_or(1);

        // unbounded, if maxOccurs = unbounded, otherwise the ·actual value· of the maxOccurs
        // [attribute], if present, otherwise 1.
        let max_occurs = any
            .attribute("maxOccurs")
            .map(|v| {
                if v == "unbounded" {
                    MaxOccurs::Unbounded
                } else {
                    MaxOccurs::Count(actual_value::<u64>(v, any))
                }
            })
            .unwrap_or(MaxOccurs::Count(1));

        // A wildcard as above.
        let term = Term::Wildcard(wildcard);

        // {annotations} The same annotations as the {annotations} of the wildcard.
        let annotations = wildcard.get(context.components()).annotations.clone();

        context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        })
    }
}

impl Component for Particle {
    const DISPLAY_NAME: &'static str = "Particle";
}
