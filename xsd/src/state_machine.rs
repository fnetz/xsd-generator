//! https://www.cogsci.ed.ac.uk/~ht/XML_Europe_2003.html
use crate::{
    ElementDeclaration, Particle, Ref, SchemaComponentTable, Term, Wildcard,
    model_group::Compositor,
};
use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Transition {
    ElementDeclaration(Ref<ElementDeclaration>),
    Wildcard(Ref<Wildcard>),
    Eof,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    BeginParticle(Ref<Particle>),
    EndParticle(Ref<Particle>),
    BeginTerm,
    EndTerm,
    StoreSlot(usize),
    Terminate,

    EndParticleAndStore(Ref<Particle>, usize),
    EndParticleAndTerminate(Ref<Particle>),
    // CommitTerm,
}

impl Action {
    fn perform_on_stacky(&self, stack: &mut Vec<StackEntry>) {
        match self {
            Action::BeginParticle(p) => {
                stack.push(*p);
            }
            Action::EndParticle(p) => {
                let top = stack.pop().expect("stack underflow");
                assert_eq!(top, *p);
            }
            Action::BeginTerm => {
                // stack.push(StackEntry::Term);
            }
            Action::EndTerm => {
                // let top = stack.pop().expect("stack underflow");
                // assert_eq!(top, StackEntry::Term);
            }
            Action::StoreSlot(_) => {
                assert!(!stack.is_empty());
            }
            Action::Terminate => {
                assert!(stack.is_empty());
            }

            Action::EndParticleAndStore(p, s) => {
                Action::EndParticle(*p).perform_on_stacky(stack);
                Action::StoreSlot(*s).perform_on_stacky(stack);
            }
            Action::EndParticleAndTerminate(p) => {
                Action::EndParticle(*p).perform_on_stacky(stack);
                Action::Terminate.perform_on_stacky(stack);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Action2 {
    BeginParticleAndTerm(Ref<Particle>),
    EndTermParticleAndStore(Ref<Particle>, usize),
    EndTermParticleAndTerminate(Ref<Particle>),
    StoreEmpty(Ref<Particle>, usize),
    TerminateEmpty(Ref<Particle>),
    CommitTerm,
}

impl Action2 {
    fn perform_on_stack(&self, stack: &mut Vec<StackEntry>) {
        match self {
            Action2::BeginParticleAndTerm(p) => {
                Action::BeginParticle(*p).perform_on_stacky(stack);
                Action::BeginTerm.perform_on_stacky(stack);
            }
            Action2::EndTermParticleAndStore(p, s) => {
                Action::EndTerm.perform_on_stacky(stack);
                Action::EndParticleAndStore(*p, *s).perform_on_stacky(stack);
            }
            Action2::EndTermParticleAndTerminate(p) => {
                Action::EndTerm.perform_on_stacky(stack);
                Action::EndParticleAndTerminate(*p).perform_on_stacky(stack);
            }
            Action2::StoreEmpty(p, s) => {
                Action::BeginParticle(*p).perform_on_stacky(stack);
                Action::EndParticle(*p).perform_on_stacky(stack);
                Action::StoreSlot(*s).perform_on_stacky(stack);
            }
            Action2::TerminateEmpty(p) => {
                Action::BeginParticle(*p).perform_on_stacky(stack);
                Action::EndParticle(*p).perform_on_stacky(stack);
                Action::Terminate.perform_on_stacky(stack);
            }
            Action2::CommitTerm => {
                Action::EndTerm.perform_on_stacky(stack);
                Action::BeginTerm.perform_on_stacky(stack);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EpsilonOr<T> {
    Epsilon,
    Action(Action),
    Transition(T),
}

impl<T> EpsilonOr<T> {
    fn is_epsilon(&self) -> bool {
        matches!(self, EpsilonOr::Epsilon)
    }
}

impl From<Transition> for EpsilonOr<Transition> {
    fn from(transition: Transition) -> Self {
        EpsilonOr::Transition(transition)
    }
}

#[derive(Default)]
pub struct EpsilonNfa {
    starting_state: Option<u32>,
    end_state: Option<u32>,
    transitions: Vec<Vec<(u32, EpsilonOr<Transition>, ContextTreeIndex)>>,
}

impl EpsilonNfa {
    fn create_state(&mut self) -> u32 {
        let state = self.transitions.len() as u32;
        self.transitions.push(Vec::new());
        state
    }

    fn add_element_transition(
        &mut self,
        from: u32,
        to: u32,
        label: Ref<ElementDeclaration>,
        context: ContextTreeIndex,
    ) {
        self.transitions[from as usize].push((
            to,
            Transition::ElementDeclaration(label).into(),
            context,
        ));
    }

    fn add_wildcard_transition(
        &mut self,
        from: u32,
        to: u32,
        label: Ref<Wildcard>,
        context: ContextTreeIndex,
    ) {
        self.transitions[from as usize].push((to, Transition::Wildcard(label).into(), context));
    }

    fn add_epsilon_transition(&mut self, from: u32, to: u32) {
        self.transitions[from as usize].push((to, EpsilonOr::Epsilon, 0));
    }

    fn add_action_transition(&mut self, from: u32, to: u32, action: Action) {
        self.transitions[from as usize].push((to, EpsilonOr::Action(action), 0));
    }

    fn set_starting_state(&mut self, state: u32) {
        assert!(self.starting_state.is_none());
        assert!(state < self.state_count() as u32);
        self.starting_state = Some(state);
    }

    fn set_end_state(&mut self, state: u32) {
        assert!(self.end_state.is_none());
        assert!(state < self.state_count() as u32);
        self.end_state = Some(state);
    }

    fn get_transitions(&self, state: u32) -> &[(u32, EpsilonOr<Transition>, ContextTreeIndex)] {
        &self.transitions[state as usize]
    }

    fn get_transitions_to(
        &self,
        state: u32,
    ) -> impl Iterator<Item = (u32, EpsilonOr<Transition>)> + '_ {
        self.transitions
            .iter()
            .enumerate()
            .flat_map(move |(from, tos)| {
                tos.iter()
                    .filter(move |(to, _, _)| *to == state)
                    .map(move |(_, transition, _)| (from as u32, *transition))
            })
    }

    #[allow(dead_code)]
    fn print_dot(&self) {
        eprintln!("digraph fsm {{");
        for state in 0..self.transitions.len() as u32 {
            if Some(state) == self.end_state {
                eprintln!("  {} [shape=doublecircle];", state);
            } else {
                eprintln!("  {} [shape=circle];", state);
            }
        }
        for (from, tos) in self.transitions.iter().enumerate() {
            for (to, label, _) in tos {
                match label {
                    EpsilonOr::Transition(Transition::ElementDeclaration(label)) => {
                        eprintln!("  {} -> {} [label=\"{:?}\"];", from, to, label);
                    }
                    EpsilonOr::Transition(Transition::Wildcard(label)) => {
                        eprintln!("  {} -> {} [label=\"{:?}\"];", from, to, label);
                    }
                    EpsilonOr::Transition(Transition::Eof) => {
                        eprintln!("  {} -> {} [label=\"EOF\"];", from, to);
                    }
                    EpsilonOr::Epsilon => {
                        eprintln!("  {} -> {} [label=\"ε\"];", from, to);
                    }
                    EpsilonOr::Action(a) => {
                        eprintln!("  {} -> {} [label=\"φ {:?}\"];", from, to, a);
                    }
                }
            }
        }
        if let Some(starting_state) = self.starting_state {
            eprintln!("  s [shape=point];");
            eprintln!("  s -> {};", starting_state,);
        }
        eprintln!("}}");
    }

    fn state_count(&self) -> usize {
        self.transitions.len()
    }

    fn compute_epsilon_closure(&self) -> Vec<BTreeSet<u32>> {
        // let mut e = BTreeMap::new();
        let mut e = vec![BTreeSet::new(); self.state_count()];
        for state in 0..self.transitions.len() as u32 {
            // e.insert(state, BTreeMap::from([(state, None)]));
            e[state as usize].insert(state);
        }
        let mut queue = BTreeSet::from_iter(0..self.transitions.len() as u32);
        while let Some(state) = queue.pop_first() {
            let mut t = BTreeSet::from([state]);
            for (to, label, _) in self.get_transitions(state) {
                if label.is_epsilon() {
                    t.extend(e[*to as usize].iter());
                }
            }
            if t != e[state as usize] {
                // e.insert(state, t);
                e[state as usize] = t;
                queue.extend(
                    self.get_transitions_to(state)
                        .filter(|&(_, t)| t.is_epsilon())
                        .map(|(from, _)| from),
                );
            }
        }
        e
    }
}

struct Link<'a> {
    next: Option<&'a Link<'a>>,
    action: Action,
}

impl<'a> Link<'a> {
    fn collapse(link: Option<&Self>) -> Vec<Action2> {
        let mut path = Vec::new();
        let mut hold = None;
        let mut link = link;
        while let Some(l) = link {
            if let Some(held) = hold.take() {
                hold = match l.action {
                    Action::BeginParticle(p) => match held {
                        Action::BeginTerm => {
                            path.push(Action2::BeginParticleAndTerm(p));
                            None
                        }
                        Action::EndParticleAndStore(p, s) => {
                            path.push(Action2::StoreEmpty(p, s));
                            None
                        }
                        Action::EndParticleAndTerminate(p) => {
                            path.push(Action2::TerminateEmpty(p));
                            None
                        }
                        _ => unreachable!(
                            "BeginParticle must be followed by BeginTerm/EndParticle+Store/EndParticle+Terminate"
                        ),
                    },
                    Action::EndParticle(p) => match held {
                        Action::StoreSlot(s) => Some(Action::EndParticleAndStore(p, s)),
                        Action::Terminate => Some(Action::EndParticleAndTerminate(p)),
                        _ => unreachable!("EndParticle must be followed by Store/Terminate"),
                    },
                    Action::BeginTerm => todo!(),
                    Action::EndTerm => match held {
                        Action::BeginTerm => {
                            path.push(Action2::CommitTerm);
                            None
                        }
                        Action::EndParticleAndStore(p, s) => {
                            path.push(Action2::EndTermParticleAndStore(p, s));
                            None
                        }
                        Action::EndParticleAndTerminate(p) => {
                            path.push(Action2::EndTermParticleAndTerminate(p));
                            None
                        }
                        _ => unreachable!(
                            "EndTerm must be followed by BeginTerm/EndParticle+Store/EndParticle+Terminate"
                        ),
                    },
                    Action::StoreSlot(_) => todo!(),
                    Action::Terminate => todo!(),
                    Action::EndParticleAndStore(_, _) => todo!(),
                    Action::EndParticleAndTerminate(_) => todo!(),
                };
            } else {
                hold = Some(l.action);
            }

            link = l.next;
        }
        if let Some(held) = hold {
            todo!("held: {:?}", held);
        }
        path.reverse();
        path
    }
}

fn visit_state(
    state: u32,
    // base: u32,
    // nfa: &EpsilonNfa,
    // dfa: &mut LabeledDfa<Vec<Epsilon>, u32>,
    // visited: &mut BTreeSet<u32>,
    link: Option<&Link>,
    gctx: &mut TestCtx,
    lctx: &mut TestCtxLocal,
) {
    if !lctx.visited.insert(state) {
        return;
    }

    if Some(state) == gctx.nfa.end_state {
        // gctx.dfa.get_or_create(lctx.base, ());
        let path = Link::collapse(link);

        let mut stack = lctx.base_stack.clone();

        for action in &path {
            action.perform_on_stack(&mut stack);
        }

        assert!(stack.is_empty());

        let (e, _) = gctx.dfa.get_or_create(state, stack);
        gctx.dfa.dfa.end_states.insert(e);

        gctx.dfa
            .add_transition(&lctx.base, &state, Transition::Eof, path);
    }

    for (to, label, _) in gctx.nfa.get_transitions(state) {
        match label {
            EpsilonOr::Epsilon => {
                visit_state(*to, link, gctx, lctx);
            }
            EpsilonOr::Action(action) => {
                let link = Link {
                    next: link,
                    action: *action,
                };
                visit_state(*to, Some(&link), gctx, lctx);
            }
            EpsilonOr::Transition(t) => {
                let path = Link::collapse(link);

                let mut stack = lctx.base_stack.clone();

                for action in &path {
                    action.perform_on_stack(&mut stack);
                }

                // gctx.dfa.get_or_create(lctx.base);
                let (_, to_visited) = gctx.dfa.get_or_create(*to, stack.clone());
                gctx.dfa.add_transition(&lctx.base, to, *t, path);

                if !to_visited {
                    gctx.to_visit.push((*to, stack));
                }
            }
        }
    }
}

// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
// pub enum StackEntry {
//     Particle(Ref<Particle>),
// }
pub type StackEntry = Ref<Particle>;

struct TestCtx<'a> {
    nfa: &'a EpsilonNfa,
    dfa: LabeledDfa<Vec<StackEntry>, Vec<Action2>, u32>,
    to_visit: Vec<(u32, Vec<StackEntry>)>,
}

struct TestCtxLocal {
    base: u32,
    base_stack: Vec<StackEntry>,
    visited: BTreeSet<u32>,
}

fn path_dfs_test(sm: &EpsilonNfa) -> Dfa<Vec<StackEntry>, Vec<Action2>> {
    let start = sm.starting_state.unwrap();

    let mut ctx = TestCtx {
        nfa: sm,
        dfa: LabeledDfa::new(),
        to_visit: vec![(start, vec![])],
    };

    let s = ctx.dfa.create_state(start, vec![]);
    ctx.dfa.dfa.set_starting_state(s);

    while let Some((state, stack)) = ctx.to_visit.pop() {
        eprintln!("state: {state}, stack: {stack:?}");
        let mut lctx = TestCtxLocal {
            base: state,
            base_stack: stack,
            visited: BTreeSet::new(),
        };
        visit_state(state, None, &mut ctx, &mut lctx);
    }

    ctx.dfa.print_dot();

    ctx.dfa.into_inner()
}

fn t_t(
    term: &Term,
    sm: &mut EpsilonNfa,
    s: u32,
    components: &SchemaComponentTable,
    ctxt: &mut ContextTree,
) -> u32 {
    let s_t = sm.create_state();
    sm.add_action_transition(s_t, s, Action::EndTerm);

    let b = match term {
        Term::ElementDeclaration(element) => {
            let b = sm.create_state();
            sm.add_element_transition(b, s_t, *element, ctxt.current.unwrap());
            // TODO: Substitution groups
            b
        }
        Term::ModelGroup(group) => {
            let group = group.get(components);
            match group.compositor {
                Compositor::All => unimplemented!(),
                Compositor::Choice => {
                    let b = sm.create_state();

                    for (index, particle) in group.particles.iter().copied().enumerate() {
                        let c = sm.create_state();
                        sm.add_action_transition(c, s_t, Action::StoreSlot(index));
                        let ps = t_p(particle, sm, c, components, ctxt);
                        sm.add_epsilon_transition(b, ps);
                    }

                    b
                }
                Compositor::Sequence => {
                    let mut n = s_t;
                    for (index, particle) in group.particles.iter().copied().enumerate().rev() {
                        let o = sm.create_state();
                        sm.add_action_transition(o, n, Action::StoreSlot(index));
                        n = t_p(particle, sm, o, components, ctxt);
                    }
                    n
                }
            }
        }
        Term::Wildcard(wildcard) => {
            let b = sm.create_state();
            sm.add_wildcard_transition(b, s_t, *wildcard, ctxt.current.unwrap());
            b
        }
    };

    let n_t = sm.create_state();
    sm.add_action_transition(n_t, b, Action::BeginTerm);
    n_t
}

fn t_p(
    particle: Ref<Particle>,
    sm: &mut EpsilonNfa,
    s: u32,
    components: &SchemaComponentTable,
    ctxt: &mut ContextTree,
    // index: usize,
) -> u32 {
    ctxt.push(particle);

    let s_p = sm.create_state();
    sm.add_action_transition(s_p, s, Action::EndParticle(particle));

    let part = particle.get(components);

    let mut n = s_p;
    match part.max_occurs {
        crate::particle::MaxOccurs::Unbounded => {
            let t = sm.create_state();
            let b = t_t(&part.term, sm, t, components, ctxt);
            sm.add_epsilon_transition(t, b);
            sm.add_epsilon_transition(b, n);
            n = b;
        }
        crate::particle::MaxOccurs::Count(max_occurs) => {
            for _ in 0..(max_occurs - part.min_occurs) {
                let b = t_t(&part.term, sm, n, components, ctxt);
                sm.add_epsilon_transition(b, s_p);
                n = b;
            }
        }
    }

    for _ in 0..part.min_occurs {
        n = t_t(&part.term, sm, n, components, ctxt);
    }

    let n_p = sm.create_state();
    sm.add_action_transition(n_p, n, Action::BeginParticle(particle));

    ctxt.pop();
    n_p
}

type ContextTreeIndex = u32;

struct ContextItem {
    particle: Ref<Particle>,
    parent: Option<ContextTreeIndex>,
}

struct ContextTree {
    items: Vec<ContextItem>,
    current: Option<ContextTreeIndex>,
}

impl ContextTree {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            current: None,
        }
    }

    fn push(&mut self, particle: Ref<Particle>) -> ContextTreeIndex {
        let parent = self.current;
        let index = self.items.len() as ContextTreeIndex;
        self.items.push(ContextItem { particle, parent });
        self.current = Some(index);
        index
    }

    /// # Panics
    /// Panics if the current context is empty.
    fn pop(&mut self) {
        let current = self.current.unwrap();
        self.current = self.items[current as usize].parent;
    }
}

pub struct DfaState<StateC, TransC> {
    pub context: StateC,
    pub transitions: HashMap<Transition, (u32, TransC)>,
}

impl<S, T> DfaState<S, T> {
    fn new(context: S) -> Self {
        Self {
            context,
            transitions: HashMap::new(),
        }
    }
}

pub struct Dfa<StateC, TransC> {
    pub start_state: Option<u32>,
    pub end_states: BTreeSet<u32>,
    pub states: Vec<DfaState<StateC, TransC>>,
}

impl<S, T> Default for Dfa<S, T> {
    fn default() -> Self {
        Self {
            start_state: None,
            end_states: BTreeSet::default(),
            states: Vec::default(),
        }
    }
}

impl<StateC, TransC> Dfa<StateC, TransC> {
    fn create_state(&mut self, sc: StateC) -> u32 {
        let state = self.states.len() as u32;
        self.states.push(DfaState::new(sc));
        state
    }

    fn set_starting_state(&mut self, state: u32) {
        self.start_state = Some(state);
    }

    fn add_transition(
        &mut self,
        from: u32,
        to: u32,
        label: Transition,
        context: TransC,
    ) -> Option<u32> {
        self.states[from as usize]
            .transitions
            .insert(label, (to, context))
            .map(|(to, _)| to)
    }

    pub fn get_transitions(&self, state: u32) -> &HashMap<Transition, (u32, TransC)> {
        &self.states[state as usize].transitions
    }

    pub fn is_end_state(&self, state: u32) -> bool {
        self.end_states.contains(&state)
    }
}

struct LabeledDfa<StateC, TransC, L: std::hash::Hash> {
    dfa: Dfa<StateC, TransC>,
    states_by_label: HashMap<L, u32>,
}

impl<SC, TC: Default, L: std::hash::Hash + std::cmp::Eq> LabeledDfa<SC, TC, L> {
    fn new() -> Self {
        Self {
            dfa: Dfa::default(),
            states_by_label: HashMap::new(),
        }
    }

    fn create_state(&mut self, label: L, context: SC) -> u32 {
        let state = self.dfa.create_state(context);
        self.states_by_label.insert(label, state);
        state
    }

    fn get<Q>(&self, label: &Q) -> Option<&u32>
    where
        Q: std::hash::Hash + Eq,
        L: Borrow<Q>,
    {
        self.states_by_label.get(label)
    }

    fn get_or_create(&mut self, label: L, context: SC) -> (u32, bool) {
        // TODO: check equal
        match self.get(&label) {
            Some(&state) => (state, true),
            None => (self.create_state(label, context), false),
        }
    }

    fn add_transition(&mut self, from: &L, to: &L, label: Transition, context: TC) -> Option<u32> {
        let from = self.states_by_label[from];
        let to = self.states_by_label[to];
        self.dfa.add_transition(from, to, label, context)
    }

    fn into_inner(self) -> Dfa<SC, TC> {
        self.dfa
    }

    #[allow(dead_code)]
    fn print_dot(&self)
    where
        SC: std::fmt::Debug,
        TC: std::fmt::Debug,
        L: std::fmt::Debug,
    {
        eprintln!("digraph dfa {{");
        for (state, index) in &self.states_by_label {
            let shape = if self.dfa.end_states.contains(index) {
                "doublecircle"
            } else {
                "ellipse"
            };
            let context = &self.dfa.states[*index as usize].context;
            eprintln!("  {index} [shape={shape}, label=\"{state:?}\\n({context:?})\"];");
        }
        for (from, tos) in self.dfa.states.iter().enumerate() {
            for (label, (to, context)) in &tos.transitions {
                eprint!("  {from} -> {to} [label=\"");
                match label {
                    Transition::ElementDeclaration(label) => eprint!("{label:?}"),
                    Transition::Wildcard(label) => eprint!("{label:?}"),
                    Transition::Eof => eprint!("EOF"),
                }
                eprintln!("\\n({context:?})\"];");
            }
        }
        if let Some(starting_state) = self.dfa.start_state {
            eprintln!("  s [shape=point];");
            eprintln!("  s -> {};", starting_state,);
        }
        eprintln!("}}");
    }
}

pub fn create_state_machine(
    particle: Ref<Particle>,
    components: &SchemaComponentTable,
) -> Dfa<Vec<StackEntry>, Vec<Action2>> {
    let mut ctxt = ContextTree::new();

    let mut fsm = EpsilonNfa::default();
    let s = fsm.create_state();
    fsm.set_end_state(s);
    let s2 = fsm.create_state();
    fsm.add_action_transition(s2, s, Action::Terminate);
    let starting_state = t_p(particle, &mut fsm, s2, components, &mut ctxt);
    fsm.set_starting_state(starting_state);
    fsm.print_dot();

    return path_dfs_test(&fsm);

    for (i, item) in ctxt.items.iter().enumerate() {
        eprintln!("{}: {:?} - {:?}", i, item.particle, item.parent);
    }

    let epsilon_closure = fsm.compute_epsilon_closure();
    // println!("{:#?}", epsilon_closure);

    let mut pending_states = BTreeSet::new();
    let starting_state = Rc::new(epsilon_closure[starting_state as usize].clone());
    pending_states.insert(Rc::clone(&starting_state));

    type DState = Rc<BTreeSet<u32>>;

    let mut new_dfa = LabeledDfa::<(), u32, DState>::new();

    let start = new_dfa.create_state(starting_state, ());
    new_dfa.dfa.set_starting_state(start);

    while let Some(d_state) = pending_states.pop_first() {
        let mut out_transitions = HashMap::<Transition, (u32, BTreeSet<u32>)>::new();
        for n_state in d_state.iter().copied() {
            for (to, label, context) in fsm.get_transitions(n_state) {
                match label {
                    EpsilonOr::Transition(label) => {
                        let (p, s) = out_transitions
                            .entry(*label)
                            .or_insert_with(|| (*context, BTreeSet::new()));
                        s.extend(&epsilon_closure[*to as usize]);
                        if p != context {
                            // TODO: See Note at end of 3.9.4.3 Element Sequence Accepted (Particle)
                            fsm.print_dot();
                            eprintln!("n_state: {n_state}");
                            eprintln!("to: {to}");
                            eprintln!("context: {context}");
                            eprintln!("p: {p}");
                            panic!("Paths differ");
                        }
                    }
                    EpsilonOr::Epsilon => {}
                    _ => todo!(),
                }
            }
        }

        for (label, (path, out_state)) in out_transitions {
            let out_state = Rc::new(out_state);
            let (_, exists) = new_dfa.get_or_create(Rc::clone(&out_state), ());
            if !exists {
                pending_states.insert(Rc::clone(&out_state));
            }

            new_dfa.add_transition(&d_state, &out_state, label, path);
        }
    }

    for (state, index) in &new_dfa.states_by_label {
        let is_end_state = state.iter().any(|&state| fsm.end_state == Some(state));
        if is_end_state {
            new_dfa.dfa.end_states.insert(*index);
        }
    }

    new_dfa.print_dot();

    // new_dfa.into_inner()
    todo!()
}

/// Checks if the Unique Particle Attribution (UPA) constraint is satisfied.
pub fn verify_upa_satisfied(dfa: &Dfa<(), u32>, components: &SchemaComponentTable) -> bool {
    // See Algorithm 2 from https://www.cogsci.ed.ac.uk/~ht/XML_Europe_2003.html [1] and W3C XML
    // Schema Definition Language (XSD) 1.1 Part 1, Appendix J [2]

    // Steps 1-2 of [1] are performed in the construction of the DFA.

    // 3. M2 violates the UPA if it is non-deterministic ignoring term identity, that is, if there
    //    is any state in M2 which has two outgoing edges such that any of the following hold: [1]
    for state in &dfa.states {
        // NOTE: This is O(n^2) for now, but usually the number of transitions is small enough.
        for (ti_a, transition_a) in state.transitions.keys().enumerate() {
            for transition_b in state.transitions.keys().skip(ti_a + 1) {
                match (transition_a, transition_b) {
                    (Transition::ElementDeclaration(e_a), Transition::ElementDeclaration(e_b)) => {
                        // 1. Their labels are both element declarations with the same {local name}
                        //    and {namespace name}. [1]
                        // TODO: Substitution groups [2]
                        let e_a = e_a.get(components);
                        let e_b = e_b.get(components);
                        if e_a.name == e_b.name && e_a.target_namespace == e_b.target_namespace {
                            return false;
                        }
                    }
                    (Transition::Wildcard(w_a), Transition::Wildcard(w_b)) => {
                        // 2. Their labels are both wildcards whose ranges overlap. [1]
                        let _w_a = w_a.get(components);
                        let _w_b = w_b.get(components);
                        // TODO
                    }
                    (Transition::ElementDeclaration(e), Transition::Wildcard(w))
                    | (Transition::Wildcard(w), Transition::ElementDeclaration(e)) => {
                        // 3. Their labels are a wildcard and an element declaration and the {namespace name}
                        //    of the element declaration is in the range of the wildcard. [1]
                        let _w = w.get(components);
                        let _e = e.get(components);
                        // TODO
                    }
                    (_, Transition::Eof) | (Transition::Eof, _) => todo!(),
                }
            }
        }
    }
    true
}
