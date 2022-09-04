use super::{
    annotation::Annotation, components::Component, particle::Particle, xstypes::Sequence, Ref,
};

/// Schema Component: Model Group, a kind of Term (§3.8)
#[derive(Clone, Debug)]
pub struct ModelGroup {
    pub annotations: Sequence<Ref<Annotation>>,
    pub compositor: Compositor,
    pub particles: Sequence<Ref<Particle>>,
}

#[derive(Clone, Debug)]
pub enum Compositor {
    All,
    Choice,
    Sequence,
}

impl Component for ModelGroup {
    const DISPLAY_NAME: &'static str = "ModelGroup";
}
