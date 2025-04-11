use crate::{
    ModelGroupDefinition, Ref, annotation::Annotation, components::Component, particle::Particle,
    xstypes::Sequence,
};

/// Schema Component: Model Group, a kind of Term (§3.8)
#[derive(Clone, Debug)]
pub struct ModelGroup {
    pub annotations: Sequence<Ref<Annotation>>,
    pub compositor: Compositor,
    pub particles: Sequence<Ref<Particle>>,

    /// The parent model group, if there is one. Not part of the XSD specification, but helpful for
    /// code generation.
    pub parent: Option<Ref<ModelGroupDefinition>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Compositor {
    All,
    Choice,
    Sequence,
}

impl Component for ModelGroup {
    const DISPLAY_NAME: &'static str = "ModelGroup";
}
