mod cli;

use std::collections::{BTreeMap, BTreeSet};

use clap::Parser;
use dt_xsd::{
    complex_type_def::ContentType, components::Named, model_group::Compositor,
    ComplexTypeDefinition, ElementDeclaration, Particle, Ref, SchemaComponentTable,
    SimpleTypeDefinition, Term, TypeDefinition, Wildcard,
};
use quick_xml::{events::Event, name::ResolveResult, NsReader};

// https://www.cogsci.ed.ac.uk/~ht/XML_Europe_2003.html

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Transition {
    ElementDeclaration(Ref<ElementDeclaration>),
    Wildcard(Ref<Wildcard>),
    Epsilon,
}

#[derive(Default)]
struct Fsm {
    states: u32,
    starting_state: Option<u32>,
    end_states: BTreeSet<u32>,
    // transitions: Vec<(u32, u32, Transition)>,
    transitions: BTreeMap<u32, Vec<(u32, Transition)>>,
}

impl Fsm {
    fn create_state(&mut self) -> u32 {
        let state = self.states;
        self.states += 1;
        state
    }

    fn add_element_transition(&mut self, from: u32, to: u32, label: Ref<ElementDeclaration>) {
        self.transitions
            .entry(from)
            .or_default()
            .push((to, Transition::ElementDeclaration(label)));
    }

    fn add_wildcard_transition(&mut self, from: u32, to: u32, label: Ref<Wildcard>) {
        self.transitions
            .entry(from)
            .or_default()
            .push((to, Transition::Wildcard(label)));
    }

    fn add_epsilon_transition(&mut self, from: u32, to: u32) {
        self.transitions
            .entry(from)
            .or_default()
            .push((to, Transition::Epsilon));
    }

    fn set_starting_state(&mut self, state: u32) {
        self.starting_state = Some(state);
    }

    fn add_end_state(&mut self, state: u32) {
        self.end_states.insert(state);
    }

    fn get_transitions(&self, state: u32) -> &[(u32, Transition)] {
        self.transitions
            .get(&state)
            .map(|t| t.as_slice())
            .unwrap_or(&[])
    }

    fn get_transitions_to(&self, state: u32) -> impl Iterator<Item = (u32, Transition)> + '_ {
        self.transitions.iter().flat_map(move |(from, tos)| {
            tos.iter()
                .filter(move |(to, _)| *to == state)
                .map(|(_, transition)| (*from, *transition))
        })
    }

    fn print_dot(&self) {
        println!("digraph fsm {{");
        for state in 0..self.states {
            if self.end_states.contains(&state) {
                println!("  {} [shape=doublecircle];", state);
            } else {
                println!("  {} [shape=circle];", state);
            }
        }
        for (from, tos) in &self.transitions {
            for (to, label) in tos {
                match label {
                    Transition::ElementDeclaration(label) => {
                        println!("  {} -> {} [label=\"{:?}\"];", from, to, label);
                    }
                    Transition::Wildcard(label) => {
                        println!("  {} -> {} [label=\"{:?}\"];", from, to, label);
                    }
                    Transition::Epsilon => {
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
        for state in 0..self.states {
            e.insert(state, BTreeSet::from([state]));
        }
        let mut queue = BTreeSet::from_iter(0..self.states);
        while let Some(state) = queue.pop_first() {
            let mut t = BTreeSet::from([state]);
            for (to, label) in self.get_transitions(state) {
                if matches!(label, Transition::Epsilon) {
                    t.extend(e[to].iter());
                }
            }
            if t != e[&state] {
                e.insert(state, t);
                queue.extend(
                    self.get_transitions_to(state)
                        .filter(|&(_, t)| t == Transition::Epsilon)
                        .map(|(from, _)| from),
                );
            }
        }
        e
    }
}

fn t_t(term: &Term, sm: &mut Fsm, s: u32, components: &SchemaComponentTable) -> u32 {
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

fn t_p(particle: &Particle, sm: &mut Fsm, s: u32, components: &SchemaComponentTable) -> u32 {
    let mut n = s;
    match particle.max_occurs {
        dt_xsd::particle::MaxOccurs::Unbounded => {
            let t = sm.create_state();
            let b = t_t(&particle.term, sm, t, components);
            sm.add_epsilon_transition(t, b);
            sm.add_epsilon_transition(b, n);
            n = b;
        }
        dt_xsd::particle::MaxOccurs::Count(max_occurs) => {
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

fn main() {
    let cli = cli::Cli::parse();

    let xsd = std::fs::read_to_string(cli.schema).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
        ..Default::default()
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = dt_xsd::read_schema(
        xsd,
        match cli.builtin_overwrite {
            cli::BuiltinOverwriteAction::Deny => dt_xsd::BuiltinOverwriteAction::Deny,
            cli::BuiltinOverwriteAction::Warn => dt_xsd::BuiltinOverwriteAction::Warn,
            cli::BuiltinOverwriteAction::Allow => dt_xsd::BuiltinOverwriteAction::Allow,
        },
        match cli.register_builtins {
            cli::RegisterBuiltins::Yes => dt_xsd::RegisterBuiltins::Yes,
            cli::RegisterBuiltins::No => dt_xsd::RegisterBuiltins::No,
        },
        &[],
    );

    for complex in components.complex_type_definitions.as_ref() {
        match complex.content_type {
            ContentType::ElementOnly { particle, .. } | ContentType::Mixed { particle, .. } => {
                let particle = particle.get(&components);
                let mut fsm = Fsm::default();
                let s = fsm.create_state();
                fsm.add_end_state(s);
                let starting_state = t_p(particle, &mut fsm, s, &components);
                fsm.set_starting_state(starting_state);
                fsm.print_dot();

                println!("{:#?}", fsm.compute_epsilon_closure());
            }
            _ => {}
        }
    }

    let mut reader = NsReader::from_file(&cli.input).unwrap();

    let mut buffer = Vec::new();
    let mut stack = Vec::<EvalContext>::new();

    struct EvalContextComplex<'a> {
        type_def: &'a ComplexTypeDefinition,
        state_machine: Fsm,
        epsilon_closure: BTreeMap<u32, BTreeSet<u32>>,
        current_states: BTreeSet<u32>,
    }
    enum EvalContextType<'a> {
        Empty,
        Simple { type_def: &'a SimpleTypeDefinition },
        Complex(EvalContextComplex<'a>),
    }

    struct EvalContext<'a> {
        element: &'a ElementDeclaration,
        type_: EvalContextType<'a>,
    }

    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .expect("Error reading event");
        match event {
            Event::Eof => break,
            Event::Start(tag) => {
                println!("START {:?}", tag);
                let (ns, local) = reader.resolve_element(tag.name());
                let namespace = match ns {
                    ResolveResult::Unbound => None,
                    ResolveResult::Bound(prefix) => {
                        Some(std::str::from_utf8(prefix.into_inner()).unwrap())
                    }
                    ResolveResult::Unknown(u) => {
                        panic!("Unknown namespace {:?}", u);
                    }
                };
                let local = std::str::from_utf8(local.into_inner()).unwrap();

                // TODO: xsi:type
                let element = schema.find_element_by_name(namespace, local, &components);
                if let Some(element) = element {
                    println!(
                        "Element found by name {:?}",
                        element.get(&components).name().unwrap()
                    );
                } else {
                    println!("Element not found by name");
                }

                if let Some(top) = stack.last_mut() {
                    let context = match top.type_ {
                        EvalContextType::Simple { .. } | EvalContextType::Empty => {
                            panic!("Simple or empty type on top of stack");
                        }
                        EvalContextType::Complex(ref mut context) => context,
                    };

                    let mut new_states = BTreeSet::new();
                    for state in context.current_states.iter().copied() {
                        for (to, label) in context.state_machine.get_transitions(state) {
                            match label {
                                Transition::ElementDeclaration(label) => {
                                    let element = label.get(&components);
                                    if element.target_namespace.as_deref() == namespace
                                        && element.name == local
                                    {
                                        new_states
                                            .extend(context.epsilon_closure[&to].iter().copied());
                                    }
                                }
                                Transition::Wildcard(label) => {
                                    let _wildcard = label.get(&components);
                                    // TODO: implement
                                }
                                Transition::Epsilon => {}
                            }
                        }
                    }
                    context.current_states = new_states;
                } else {
                    // Root element!
                    let element = element.expect("Root element not found by name");
                    let type_def = element.get(&components).type_definition;
                    let type_ = match type_def {
                        TypeDefinition::Simple(s) => {
                            println!("Simple type");
                            EvalContextType::Simple {
                                type_def: s.get(&components),
                            }
                        }
                        TypeDefinition::Complex(c) => {
                            let c = c.get(&components);
                            println!("Complex type {:?}", c.name());
                            match c.content_type {
                                ContentType::ElementOnly { particle, .. }
                                | ContentType::Mixed { particle, .. } => {
                                    // TODO: mixed
                                    // TODO: open content
                                    println!("Element only or Mixed");
                                    let particle = particle.get(&components);
                                    let mut state_machine = Fsm::default();
                                    let s = state_machine.create_state();
                                    state_machine.add_end_state(s);
                                    let starting_state =
                                        t_p(particle, &mut state_machine, s, &components);
                                    state_machine.set_starting_state(starting_state);
                                    state_machine.print_dot();
                                    let epsilon_closure = state_machine.compute_epsilon_closure();
                                    println!("Epsilon closure: {:?}", epsilon_closure);
                                    EvalContextType::Complex(EvalContextComplex {
                                        type_def: c,
                                        state_machine,
                                        current_states: epsilon_closure[&starting_state].clone(),
                                        epsilon_closure,
                                    })
                                }
                                ContentType::Simple {
                                    simple_type_definition,
                                } => EvalContextType::Simple {
                                    type_def: simple_type_definition.get(&components),
                                },
                                ContentType::Empty => EvalContextType::Empty,
                            }
                        }
                    };
                    stack.push(EvalContext {
                        element: element.get(&components),
                        type_,
                    });
                }
            }
            Event::End(tag) => {
                println!("END {:?}", tag);
                let context = stack.pop().expect("Stack underflow");

                let (ns, local) = reader.resolve_element(tag.name());
                let namespace = match ns {
                    ResolveResult::Unbound => None,
                    ResolveResult::Bound(prefix) => {
                        Some(std::str::from_utf8(prefix.into_inner()).unwrap())
                    }
                    ResolveResult::Unknown(u) => {
                        panic!("Unknown namespace {:?}", u);
                    }
                };
                let local = std::str::from_utf8(local.into_inner()).unwrap();
                assert_eq!(context.element.name, local);
                assert_eq!(context.element.target_namespace.as_deref(), namespace);

                match context.type_ {
                    EvalContextType::Simple { .. } | EvalContextType::Empty => {}
                    EvalContextType::Complex(context) => {
                        if context
                            .current_states
                            .intersection(&context.state_machine.end_states)
                            .count()
                            > 0
                        {
                            println!("Valid");
                        } else {
                            println!("Invalid");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
