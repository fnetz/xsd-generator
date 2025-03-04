//! https://www.cogsci.ed.ac.uk/~ht/XML_Europe_2003.html
use crate::{
    ElementDeclaration, Particle, Ref, SchemaComponentTable, Term, Wildcard,
    model_group::Compositor,
};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Transition {
    ElementDeclaration(Ref<ElementDeclaration>),
    Wildcard(Ref<Wildcard>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EpsilonOr<T> {
    Epsilon,
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
    end_states: BTreeSet<u32>,
    transitions: Vec<Vec<(u32, EpsilonOr<Transition>)>>,
}

impl EpsilonNfa {
    fn create_state(&mut self) -> u32 {
        let state = self.transitions.len() as u32;
        self.transitions.push(Vec::new());
        state
    }

    fn add_element_transition(&mut self, from: u32, to: u32, label: Ref<ElementDeclaration>) {
        self.transitions[from as usize].push((to, Transition::ElementDeclaration(label).into()));
    }

    fn add_wildcard_transition(&mut self, from: u32, to: u32, label: Ref<Wildcard>) {
        self.transitions[from as usize].push((to, Transition::Wildcard(label).into()));
    }

    fn add_epsilon_transition(&mut self, from: u32, to: u32) {
        self.transitions[from as usize].push((to, EpsilonOr::Epsilon));
    }

    fn set_starting_state(&mut self, state: u32) {
        self.starting_state = Some(state);
    }

    fn add_end_state(&mut self, state: u32) {
        self.end_states.insert(state);
    }

    fn get_transitions(&self, state: u32) -> &[(u32, EpsilonOr<Transition>)] {
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
                    .filter(move |(to, _)| *to == state)
                    .map(move |(_, transition)| (from as u32, *transition))
            })
    }

    fn print_dot(&self) {
        println!("digraph fsm {{");
        for state in 0..self.transitions.len() as u32 {
            if self.end_states.contains(&state) {
                println!("  {} [shape=doublecircle];", state);
            } else {
                println!("  {} [shape=circle];", state);
            }
        }
        for (from, tos) in self.transitions.iter().enumerate() {
            for (to, label) in tos {
                match label {
                    EpsilonOr::Transition(Transition::ElementDeclaration(label)) => {
                        println!("  {} -> {} [label=\"{:?}\"];", from, to, label);
                    }
                    EpsilonOr::Transition(Transition::Wildcard(label)) => {
                        println!("  {} -> {} [label=\"{:?}\"];", from, to, label);
                    }
                    EpsilonOr::Epsilon => {
                        println!("  {} -> {} [label=\"ε\"];", from, to);
                    }
                }
            }
        }
        if let Some(starting_state) = self.starting_state {
            println!("  s [shape=point];");
            println!("  s -> {};", starting_state,);
        }
        println!("}}");
    }

    fn compute_epsilon_closure(&self) -> BTreeMap<u32, BTreeSet<u32>> {
        let mut e = BTreeMap::new();
        for state in 0..self.transitions.len() as u32 {
            e.insert(state, BTreeSet::from([state]));
        }
        let mut queue = BTreeSet::from_iter(0..self.transitions.len() as u32);
        while let Some(state) = queue.pop_first() {
            let mut t = BTreeSet::from([state]);
            for (to, label) in self.get_transitions(state) {
                if label.is_epsilon() {
                    t.extend(e[to].iter());
                }
            }
            if t != e[&state] {
                e.insert(state, t);
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

fn t_t(term: &Term, sm: &mut EpsilonNfa, s: u32, components: &SchemaComponentTable) -> u32 {
    match term {
        Term::ElementDeclaration(element) => {
            let b = sm.create_state();
            sm.add_element_transition(b, s, *element);
            b
        }
        Term::ModelGroup(group) => {
            let group = group.get(components);
            match group.compositor {
                Compositor::All => unimplemented!(),
                Compositor::Choice => todo!(),
                Compositor::Sequence => {
                    let mut n = s;
                    for particle in group.particles.iter().rev() {
                        n = t_p(particle.get(components), sm, n, components);
                    }
                    n
                }
            }
        }
        Term::Wildcard(wildcard) => {
            let b = sm.create_state();
            sm.add_wildcard_transition(b, s, *wildcard);
            b
        }
    }
}

fn t_p(particle: &Particle, sm: &mut EpsilonNfa, s: u32, components: &SchemaComponentTable) -> u32 {
    let mut n = s;
    match particle.max_occurs {
        crate::particle::MaxOccurs::Unbounded => {
            let t = sm.create_state();
            let b = t_t(&particle.term, sm, t, components);
            sm.add_epsilon_transition(t, b);
            sm.add_epsilon_transition(b, n);
            n = b;
        }
        crate::particle::MaxOccurs::Count(max_occurs) => {
            for _ in 0..(max_occurs - particle.min_occurs) {
                let b = t_t(&particle.term, sm, n, components);
                sm.add_epsilon_transition(b, s);
                n = b;
            }
        }
    }

    for _ in 0..particle.min_occurs {
        n = t_t(&particle.term, sm, n, components);
    }

    n
}

#[derive(Default)]
pub struct Dfa {
    pub start_state: Option<u32>,
    end_states: BTreeSet<u32>,
    transitions: Vec<HashMap<Transition, u32>>,
}

impl Dfa {
    fn create_state(&mut self) -> u32 {
        let state = self.transitions.len() as u32;
        self.transitions.push(HashMap::new());
        state
    }

    fn set_starting_state(&mut self, state: u32) {
        self.start_state = Some(state);
    }

    fn add_transition(&mut self, from: u32, to: u32, label: Transition) -> Option<u32> {
        self.transitions[from as usize].insert(label, to)
    }

    pub fn get_transitions(&self, state: u32) -> &HashMap<Transition, u32> {
        &self.transitions[state as usize]
    }

    pub fn is_end_state(&self, state: u32) -> bool {
        self.end_states.contains(&state)
    }
}

struct LabeledDfa<L: std::hash::Hash> {
    dfa: Dfa,
    states_by_label: HashMap<L, u32>,
}

impl<L: std::hash::Hash + std::cmp::Eq> LabeledDfa<L> {
    fn new() -> Self {
        Self {
            dfa: Dfa::default(),
            states_by_label: HashMap::new(),
        }
    }

    fn create_state(&mut self, label: L) -> u32 {
        let state = self.dfa.create_state();
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

    fn get_or_create(&mut self, label: L) -> (u32, bool) {
        match self.get(&label) {
            Some(&state) => (state, true),
            None => (self.create_state(label), false),
        }
    }

    fn add_transition(&mut self, from: &L, to: &L, label: Transition) -> Option<u32> {
        let from = self.states_by_label[from];
        let to = self.states_by_label[to];
        self.dfa.add_transition(from, to, label)
    }

    fn into_inner(self) -> Dfa {
        self.dfa
    }

    fn print_dot(&self)
    where
        L: std::fmt::Debug,
    {
        println!("digraph dfa {{");
        for (state, index) in &self.states_by_label {
            let shape = if self.dfa.end_states.contains(index) {
                "doublecircle"
            } else {
                "circle"
            };
            println!("  {index} [shape={shape}, label=\"{state:?}\"];");
        }
        for (from, tos) in self.dfa.transitions.iter().enumerate() {
            for (label, to) in tos {
                print!("  {from} -> {to} [label=\"");
                match label {
                    Transition::ElementDeclaration(label) => print!("{label:?}"),
                    Transition::Wildcard(label) => print!("{label:?}"),
                }
                println!("\"];");
            }
        }
        if let Some(starting_state) = self.dfa.start_state {
            println!("  s [shape=point];");
            println!("  s -> {};", starting_state,);
        }
        println!("}}");
    }
}

pub fn create_state_machine(particle: &Particle, components: &SchemaComponentTable) -> Dfa {
    let mut fsm = EpsilonNfa::default();
    let s = fsm.create_state();
    fsm.add_end_state(s);
    let starting_state = t_p(particle, &mut fsm, s, components);
    fsm.set_starting_state(starting_state);
    fsm.print_dot();

    let epsilon_closure = fsm.compute_epsilon_closure();
    println!("{:#?}", epsilon_closure);

    let mut pending_states = BTreeSet::new();
    let starting_state = Rc::new(epsilon_closure[&starting_state].clone());
    pending_states.insert(Rc::clone(&starting_state));

    type DState = Rc<BTreeSet<u32>>;

    let mut new_dfa = LabeledDfa::<DState>::new();

    let start = new_dfa.create_state(starting_state);
    new_dfa.dfa.set_starting_state(start);

    while let Some(d_state) = pending_states.pop_first() {
        let mut out_transitions = HashMap::<Transition, BTreeSet<u32>>::new();
        for n_state in d_state.iter().copied() {
            for (to, label) in fsm.get_transitions(n_state) {
                match label {
                    EpsilonOr::Transition(label) => {
                        out_transitions
                            .entry(*label)
                            .or_default()
                            .extend(&epsilon_closure[to]);
                    }
                    EpsilonOr::Epsilon => {}
                }
            }
        }

        for (label, out_state) in out_transitions {
            let out_state = Rc::new(out_state);
            let (_, exists) = new_dfa.get_or_create(Rc::clone(&out_state));
            if !exists {
                pending_states.insert(Rc::clone(&out_state));
            }

            new_dfa.add_transition(&d_state, &out_state, label);
        }
    }

    for (state, index) in &new_dfa.states_by_label {
        let is_end_state = state.iter().any(|&state| fsm.end_states.contains(&state));
        if is_end_state {
            new_dfa.dfa.end_states.insert(*index);
        }
    }

    new_dfa.print_dot();

    new_dfa.into_inner()
}

/// Checks if the Unique Particle Attribution (UPA) constraint is satisfied.
pub fn verify_upa_satisfied(dfa: &Dfa, components: &SchemaComponentTable) -> bool {
    // See Algorithm 2 from https://www.cogsci.ed.ac.uk/~ht/XML_Europe_2003.html [1] and W3C XML
    // Schema Definition Language (XSD) 1.1 Part 1, Appendix J [2]

    // Steps 1-2 of [1] are performed in the construction of the DFA.

    // 3. M2 violates the UPA if it is non-deterministic ignoring term identity, that is, if there
    //    is any state in M2 which has two outgoing edges such that any of the following hold: [1]
    for transitions in &dfa.transitions {
        // NOTE: This is O(n^2) for now, but usually the number of transitions is small enough.
        for (ti_a, transition_a) in transitions.keys().enumerate() {
            for transition_b in transitions.keys().skip(ti_a + 1) {
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
                }
            }
        }
    }
    true
}
