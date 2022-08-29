use super::{
    annotation::Annotation, particle::Particle, xstypes::Sequence, Ref, RefVisitor, RefsVisitable,
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

impl RefsVisitable for ModelGroup {
    fn visit_refs(&mut self, visitor: &mut impl RefVisitor) {
        self.annotations
            .iter_mut()
            .for_each(|annot| visitor.visit_ref(annot));
        self.particles
            .iter_mut()
            .for_each(|particle| visitor.visit_ref(particle));
    }
}
