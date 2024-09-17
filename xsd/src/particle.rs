use super::{
    annotation::Annotation,
    components::{Component, ComponentTable},
    element_decl,
    error::XsdError,
    model_group::Compositor,
    shared::Term,
    values::actual_value,
    xstypes::{QName, Sequence},
    ElementDeclaration, MappingContext, ModelGroup, ModelGroupDefinition, Ref, Wildcard,
};
use roxmltree::Node;

/// Schema Component: Particle, a kind of Component (§3.9)
#[derive(Clone, Debug)]
pub struct Particle {
    pub min_occurs: u64, // TODO nonNegativeInteger
    pub max_occurs: MaxOccurs,
    pub term: Term,
    pub(crate) annotations: Option<Sequence<Ref<Annotation>>>,
}

#[derive(Clone, Debug)]
pub enum MaxOccurs {
    Unbounded,
    Count(u64), // TODO NonZeroU64
}

impl MaxOccurs {
    pub(crate) fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Unbounded, _) | (_, Self::Unbounded) => Self::Unbounded,
            (Self::Count(a), Self::Count(b)) => Self::Count(a + b),
        }
    }

    pub(crate) fn mul(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Unbounded, _) | (_, Self::Unbounded) => Self::Unbounded,
            (Self::Count(a), Self::Count(b)) => Self::Count(a * b),
        }
    }

    pub(crate) fn max(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Unbounded, _) | (_, Self::Unbounded) => Self::Unbounded,
            (Self::Count(a), Self::Count(b)) => Self::Count((*a).max(*b)),
        }
    }
}

impl Particle {
    /// Schema Component Constraint: Particle Emptiable
    ///
    /// <https://www.w3.org/TR/xmlschema11-1/#cos-group-emptiable>
    pub fn is_emptiable(&self, components: &impl ComponentTable) -> bool {
        // [Definition:]  For a particle to be emptiable one or more of the following is true:
        // 1 Its {min occurs} is 0.
        // 2 Its {term} is a group and the minimum part of the effective total range of that group
        //   [...] is 0.
        self.min_occurs == 0
            || (self.term.is_model_group() && self.effective_total_range(components).0 == 0)
    }

    /// Schema Component Constraint: Effective Total Range
    ///
    /// # Panics
    /// Panics if the particle's term is not a model group.
    pub fn effective_total_range(&self, components: &impl ComponentTable) -> (u64, MaxOccurs) {
        let Term::ModelGroup(group) = self.term else {
            panic!("effective_total_range needs term to be a model group");
        };
        let group = group.get(components);

        match group.compositor {
            Compositor::All | Compositor::Sequence => {
                // Pt. 1, 3.8.6.5 Effective Total Range (all and sequence)
                let mut min_acc = 0;
                let mut max_acc = MaxOccurs::Count(0);
                for particle in group.particles.iter() {
                    let particle = particle.get(components);
                    let (min, max) = if particle.term.is_model_group() {
                        particle.effective_total_range(components)
                    } else {
                        (particle.min_occurs, particle.max_occurs.clone())
                    };
                    min_acc += min;
                    max_acc = max_acc.add(&max);
                }
                (self.min_occurs * min_acc, self.max_occurs.mul(&max_acc))
            }
            Compositor::Choice => {
                // Pt. 2, 3.8.6.6 Effective Total Range (choice)
                let mut min_acc = 0;
                let mut max_acc = MaxOccurs::Count(0);
                for particle in group.particles.iter() {
                    let particle = particle.get(components);
                    let (min, max) = if particle.term.is_model_group() {
                        particle.effective_total_range(components)
                    } else {
                        (particle.min_occurs, particle.max_occurs.clone())
                    };
                    min_acc = min_acc.min(min);
                    max_acc = max_acc.max(&max);
                }
                (self.min_occurs * min_acc, self.max_occurs.mul(&max_acc))
            }
        }
    }

    /// Map according to XML Representation of Model Group Schema Components (§3.8.2),
    /// XML Mapping Summary for Model Group Schema Component
    pub(super) fn map_from_xml_model_group_term(
        context: &mut MappingContext,
        particle: Node,
        schema: Node,
        element_parent: element_decl::ScopeParent,
    ) -> Result<Ref<ModelGroup>, XsdError> {
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
                "element" => Some(ElementDeclaration::map_from_xml_local(
                    context,
                    child,
                    schema,
                    element_parent,
                )),
                _ => None,
            })
            .collect::<Result<Vec<_>, _>>()?;

        // {annotations}
        //   The ·annotation mapping· of the <all>, <choice>, or <sequence> element, whichever is
        //   present, as defined in XML Representation of Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, particle);

        Ok(context.create(ModelGroup {
            compositor,
            particles,
            annotations,
        }))
    }

    /// Mapper for Model groups `<all>`, `<sequence>`, and `<choice>`, see XML Representation of Model
    /// Group Schema Components (§3.8.2), XML Mapping Summary for Particle Schema Component
    pub(super) fn map_from_xml_model_group(
        context: &mut MappingContext,
        particle: Node,
        schema: Node,
        element_parent: element_decl::ScopeParent,
    ) -> Result<Ref<Self>, XsdError> {
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

        // {term}
        //   [see `map_from_xml_model_group_term()` above.]
        let model_group =
            Self::map_from_xml_model_group_term(context, particle, schema, element_parent)?;
        let term = Term::ModelGroup(model_group);

        // {annotations}
        //   The same annotations as the {annotations} of the model group.
        // NOTE: These are provided on-demand by the `annotations` method.
        let annotations = None;

        Ok(context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        }))
    }

    /// Mapper for Group references `<group>`, see XML Representation of Model Group Definition
    /// Schema Components (§3.7.2)
    pub(super) fn map_from_xml_group_reference(
        context: &mut MappingContext,
        group: Node,
    ) -> Result<Ref<Particle>, XsdError> {
        assert_eq!(group.tag_name().name(), "group");
        // FIXME: minOccurs=maxOccurs=0 shouldn't create anything

        // The ·actual value· of the minOccurs [attribute], if present, otherwise 1.
        let min_occurs = group
            .attribute("minOccurs")
            .map(|v| actual_value::<u64>(v, group))
            .unwrap_or(1);

        // unbounded, if the maxOccurs [attribute] equals unbounded, otherwise the ·actual value·
        // of the maxOccurs [attribute], if present, otherwise 1.
        let max_occurs = group
            .attribute("maxOccurs")
            .map(|v| {
                if v == "unbounded" {
                    MaxOccurs::Unbounded
                } else {
                    MaxOccurs::Count(actual_value::<u64>(v, group))
                }
            })
            .unwrap_or(MaxOccurs::Count(1));

        // {term}: The {model group} of the model group definition ·resolved· to by the ·actual value· of the ref [attribute]
        // TODO: handle missing ref?
        let ref_ = actual_value::<QName>(group.attribute("ref").unwrap(), group);
        let ref_model_group_definition: Ref<ModelGroupDefinition> = context.resolve(&ref_).unwrap(); // TODO
        let term = Term::ModelGroup(context.request(ref_model_group_definition)?.model_group);

        // The ·annotation mapping· of the <group> element, as defined in XML Representation of
        // Annotation Schema Components (§3.15.2).
        let annotations = Some(Annotation::xml_element_annotation_mapping(context, group));

        Ok(context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        }))
    }

    // TODO anyAttribute

    /// Mapper for Wildcard `<any>`, see XML Representation of Wildcard Schema Components (§3.10.2)
    pub(super) fn map_from_xml_wildcard_any(
        context: &mut MappingContext,
        any: Node,
        schema: Node,
    ) -> Result<Ref<Self>, XsdError> {
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
        // NOTE: These are provided on-demand by the `annotations` method.
        let annotations = None;

        Ok(context.create(Particle {
            min_occurs,
            max_occurs,
            term,
            annotations,
        }))
    }

    pub fn annotations<'a>(&'a self, components: &'a impl ComponentTable) -> &'a [Ref<Annotation>] {
        self.annotations
            .as_deref()
            .unwrap_or_else(|| self.term.annotations(components))
    }
}

impl Component for Particle {
    const DISPLAY_NAME: &'static str = "Particle";
}
