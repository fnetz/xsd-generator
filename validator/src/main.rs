mod cli;

use clap::Parser;
use dt_xsd::{
    complex_type_def::{ContentType, ContentTypeVariety, OpenContent, OpenContentMode},
    components::{ComponentTable, Named},
    shared::ValueConstraintVariety,
    state_machine::{Dfa, Transition},
    ComplexTypeDefinition, ElementDeclaration, Particle, SchemaComponentTable,
    SimpleTypeDefinition, TypeDefinition,
};
use quick_xml::{
    events::{attributes::Attribute, BytesStart, Event},
    name::ResolveResult,
    NsReader,
};

fn attribute_name_equal<R>(
    reader: &NsReader<R>,
    attr: Attribute,
    namespace_name: &str,
    local_name: &str,
) -> bool {
    let (ns, local) = reader.resolve_attribute(attr.key);
    let namespace = match ns {
        ResolveResult::Unbound => None,
        ResolveResult::Bound(prefix) => Some(prefix.into_inner()),
        ResolveResult::Unknown(u) => {
            panic!("Unknown namespace {:?}", u);
        }
    };
    let local = local.into_inner();
    namespace.as_deref() == Some(namespace_name.as_bytes()) && local == local_name.as_bytes()
}

// TODO: deduplicate
pub const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// 3.9.4.2 Element Sequence Locally Valid (Particle)
fn element_sequence_locally_valid_particle(
    e: &roxmltree::Node,
    s: &[roxmltree::Node],
    particle: &Particle,
    open_content: &Option<OpenContent>,
    components: &SchemaComponentTable,
) {
}

/// 3.4.4.3 Element Sequence Locally Valid (Complex Content)
fn element_sequence_locally_valid_complex_content(
    e: &roxmltree::Node,
    s: &[roxmltree::Node],
    particle: &Particle,
    open_content: &Option<OpenContent>,
    components: &SchemaComponentTable,
) -> bool {
    // For a sequence S (possibly empty) of element information items to be locally ·valid· with
    // respect to a Content Type CT, the appropriate case among the following must be true:

    match open_content {
        None => {
            // 1 If CT.{open content} is ·absent·, then S is ·valid· with respect to CT.{particle},
            //   as defined in Element Sequence Locally Valid (Particle) (§3.9.4.2).
            element_sequence_locally_valid_particle(e, s, particle, open_content, components)
        }
        Some(open_content) => match open_content.mode {
            OpenContentMode::Interleave => {
                // 2 If CT.{open content}.{mode} = suffix , then S can be represented as two
                //   subsequences S1 and S2 (either can be empty) such that all of the following
                //   are true:
                todo!()
            }
            OpenContentMode::Suffix => {
                // 3 otherwise (CT.{open content}.{mode} = interleave) S can be represented as two
                //   subsequences S1 and S2 (either can be empty) such that all of the following
                //   are true:
                todo!()
            }
        },
    }

    todo!()
}

/// 3.4.4.2 Element Locally Valid (Complex Type)
fn element_locally_valid_complex_type(
    e: &roxmltree::Node,
    e_is_nilled: bool,
    t: &ComplexTypeDefinition,
    components: &SchemaComponentTable,
) -> bool {
    // For an element information item E to be locally ·valid· with respect to a complex type
    // definition T all of the following must be true:
    if !e_is_nilled {
        // 1 If E is not ·nilled·, then all of the following are true:
        match &t.content_type {
            ContentType::Empty => {
                // 1.1 If T.{content type}.{variety} = empty, then E has no character or element
                //   information item [children].
                if e.has_children() {
                    return false;
                }
            }
            ContentType::Simple {
                simple_type_definition,
            } => {
                // 1.2 If T.{content type}.{variety} = simple, then E has no element information
                //   item [children], and the ·initial value· of E is ·valid· with respect to
                //   T.{content type}.{simple type definition} as defined by String Valid
                //   (§3.16.4).
                if e.children().any(|c| c.is_element()) {
                    return false;
                }

                // (...) the initial value of an element information item is the string composed
                // of, in order, the [character code] of each character information item in the
                // [children] of that element information item. (Part 1, 3.1.4)
                let initial_value: String = e
                    .children()
                    .filter(|c| c.is_text())
                    .map(|c| c.text().unwrap())
                    .collect();

                if !string_valid(&initial_value, simple_type_definition.get(components)) {
                    return false;
                }
            }
            ContentType::ElementOnly {
                particle,
                open_content,
            }
            | ContentType::Mixed {
                particle,
                open_content,
            } => {
                if t.content_type.variety() == ContentTypeVariety::ElementOnly {
                    // 1.3 If T.{content type}.{variety} = element-only, then E has no character
                    //   information item [children] other than those whose [character code] is
                    //   defined as a white space in [XML 1.1].
                    // TODO: whitespace, refer to generator utility lib
                    if e.children()
                        .any(|c| c.is_text() && !c.text().unwrap().trim().is_empty())
                    {
                        return false;
                    }
                }

                // 1.4 If T.{content type}.{variety} = element-only or T.{content
                //   type}.{variety} = mixed, then the sequence of element information items in
                //   E.[children], if any, taken in order, is ·valid· with respect to
                //   T.{content type}, as defined in Element Sequence Locally Valid (Complex
                //   Content) (§3.4.4.3).
                let s = e.children().filter(|c| c.is_element()).collect::<Vec<_>>();
                let particle = particle.get(components);
                if !element_sequence_locally_valid_complex_content(
                    e,
                    &s,
                    particle,
                    open_content,
                    components,
                ) {
                    return false;
                }
            }
        }
    }

    // 2 For each attribute information item A in E.[attributes] excepting those named xsi:type,
    //   xsi:nil, xsi:schemaLocation, or xsi:noNamespaceSchemaLocation (see Built-in Attribute
    //   Declarations (§3.2.7)), the appropriate case among the following is true:

    // 3 For each attribute use U in T.{attribute uses}, if U.{required} = true, then U.{attribute
    //   declaration} has the same expanded name as one of the attribute information items in
    //   E.[attributes].

    // 4 For each ·defaulted attribute· A belonging to E, the {lexical form} of A's ·effective
    //   value constraint· is ·valid· with respect to A.{attribute declaration}.{type definition}
    //   as defined by String Valid (§3.16.4).

    // 5 For each element information item in E.[children] and each attribute information item in
    //   E.[attributes], if neither the ·governing type definition· nor the ·locally declared type·
    //   is ·absent·, then the ·governing type definition· is the same as, or is ·validly
    //   substitutable· for, the ·locally declared type·, ·without limitation·.

    // 6 E is ·valid· with respect to each of the assertions in T.{assertions} as per Assertion
    //   Satisfied (§3.13.4.1).
    // TODO

    todo!()
}

fn string_valid(string: &str, t: &SimpleTypeDefinition) -> bool {
    todo!()
}

fn element_locally_valid_type(
    e: &roxmltree::Node,
    t: Option<TypeDefinition>,
    e_is_nilled: bool,
    components: &SchemaComponentTable,
) -> bool {
    // For an element information item E to be locally ·valid· with respect to a type definition T all of the following must be true:

    // 1 T is not ·absent·;
    let Some(t) = t else {
        return false;
    };

    // 3 The appropriate case among the following is true:
    match t {
        TypeDefinition::Simple(t) => {
            let t = t.get(components);

            // 3.1 If T is a simple type definition, then all of the following are true:

            //   3.1.1 E.[attributes] is empty, except for attributes named xsi:type, xsi:nil,
            //     xsi:schemaLocation, or xsi:noNamespaceSchemaLocation.
            if e.attributes().any(|a| {
                a.namespace() != Some(XSI_NAMESPACE)
                    || a.name() != "type"
                    || a.name() != "nil"
                    || a.name() != "schemaLocation"
                    || a.name() != "noNamespaceSchemaLocation"
            }) {
                return false;
            }

            // 3.1.2 E has no element information item [children].
            if e.children().any(|c| c.is_element()) {
                return false;
            }

            // 3.1.3 If E is not ·nilled·, then the ·initial value· is ·valid· with respect to T as
            //   defined by String Valid (§3.16.4).
            if !e_is_nilled {
                // (...) the initial value of an element information item is the string composed
                // of, in order, the [character code] of each character information item in the
                // [children] of that element information item. (Part 1, 3.1.4)
                let initial_value: String = e
                    .children()
                    .filter(|c| c.is_text())
                    .map(|c| c.text().unwrap())
                    .collect();

                string_valid(&initial_value, t)
            } else {
                true
            }
        }
        TypeDefinition::Complex(t) => {
            let t = t.get(components);

            // 2 If T is a complex type definition, then T.{abstract} = false.
            if t.abstract_ {
                return false;
            }

            // 3.2 If T is a complex type definition, then E is locally ·valid· with respect to T
            //   as per Element Locally Valid (Complex Type) (§3.4.4.2);
            element_locally_valid_complex_type()
        }
    }
}

fn selected_type_definition(_e: &roxmltree::Node, d: &ElementDeclaration) -> TypeDefinition {
    // The selected type definition S of an element information item E is a type definition
    // associated with E in the following way. Let D be the ·governing element declaration· of E.
    // Then:
    if let Some(_type_table) = d.type_table.as_ref() {
        // 1 If D has a {type table}, then S is the type ·conditionally selected· for E by D.{type
        //   table}.
        todo!()
    } else {
        // 2 If D has no {type table}, then S is D.{type definition}.
        d.type_definition
    }
}

fn governing_type_definition(
    e: &roxmltree::Node,
    governing_element_declaration: Option<&ElementDeclaration>,
) -> Option<TypeDefinition> {
    // The governing type definition of an element information item E, in a given schema-validity
    // ·assessment· episode, is the first of the following which applies:

    // 1 An ·instance-specified type definition· which ·overrides· a type definition stipulated by
    //   the processor (see Assessing Schema-Validity (§5.2)).
    // 2 A type definition stipulated by the processor (see Assessing Schema-Validity (§5.2)).
    // NOTE: doesn't apply (yet)

    // 3 An ·instance-specified type definition· which ·overrides· the ·selected type definition·
    //   of E.
    // TODO

    // 4 The ·selected type definition· of E.
    //   [from "selected type definition"] If E has no ·governing element declaration·, then E has
    //   no selected type definition.
    if let Some(d) = governing_element_declaration {
        return Some(selected_type_definition(e, d));
    }

    // 5 The value ·absent· if E is ·skipped·.
    // TODO

    // 6 An ·instance-specified type definition· which ·overrides· the ·locally declared type·.
    // TODO

    // 7 The ·locally declared type·.
    // TODO

    // 8 An ·instance-specified type definition·.
    // TODO

    // If none of these applies, there is no ·governing type definition· (or, in equivalent words,
    // it is ·absent·).
    None
}

fn element_locally_valid_element<R>(
    e: &roxmltree::Node,
    d: Option<&ElementDeclaration>,
    components: &SchemaComponentTable,
) -> bool {
    // 1 D is not ·absent· and E and D have the same expanded name.
    let Some(d) = d else {
        return false;
    };
    if e.tag_name().namespace() != d.target_namespace.as_deref() || e.tag_name().name() != d.name {
        return false;
    }

    // 2 D.{abstract} = false.
    if d.abstract_ {
        return false;
    }

    // 3 One of the following is true:
    let xsi_nil = e
        .attributes()
        .find(|a| a.namespace() == Some(XSI_NAMESPACE) && a.name() == "nil");
    let is_nilled = if !d.nillable {
        // 3.1 D.{nillable} = false, and E has no xsi:nil attribute.
        if xsi_nil.is_some() {
            return false;
        }
        false
    } else {
        // 3.2 D.{nillable} = true and one of the following is true
        //   3.2.1 E has no xsi:nil attribute information item.
        if let Some(xsi_nil) = xsi_nil {
            // TODO: normalization
            if xsi_nil.value() == "false" {
                // 3.2.2 E has xsi:nil = false.
                false
            } else if xsi_nil.value() == "false" {
                // 3.2.3 E has xsi:nil = true (that is, E is ·nilled·), and all of the following
                //   are true:

                //   3.2.3.1 E has no character or element information item [children].
                if e.has_children() {
                    return false;
                }

                //   3.2.3.2 D has no {value constraint} with {variety} = fixed.
                if d.value_constraint
                    .as_ref()
                    .map_or(false, |vc| vc.variety == ValueConstraintVariety::Fixed)
                {
                    return false;
                }

                true
            } else {
                return false;
            }
        } else {
            false
        }
    };

    // 4 If E has an ·instance-specified type definition· T, then T ·overrides· the ·selected type
    //   definition· of E.
    // NOTE: this step is already covered by the definition of "governing type definition"

    let governing_type_definition = governing_type_definition(e, Some(d));

    debug_assert!(
        governing_type_definition.is_some(),
        "The governing type definition must be present, \
        as the governing element declaration is present",
    );

    // 5 The appropriate case among the following is true:
    if d.value_constraint.is_some() && !e.has_children() && !is_nilled {
        // 5.1 If D has a {value constraint}, and E has neither element nor character [children],
        //   and E is not ·nilled· with respect to D , then all of the following are true:
        let _value_constraint = d.value_constraint.as_ref().unwrap();

        //   5.1.1 If E's ·governing type definition· is an ·instance-specified type definition·,
        //     then D.{value constraint} is a valid default for the ·governing type definition· as
        //     defined in Element Default Valid (Immediate) (§3.3.6.2).
        // TODO

        //   5.1.2 The element information item with D.{value constraint}.{lexical form} used as
        //     its ·normalized value· is locally ·valid· with respect to the ·governing type
        //     definition· as defined by Element Locally Valid (Type) (§3.3.4.4).
        // TODO
    } else {
        // 5.2 If D has no {value constraint}, or E has either element or character [children], or
        //   E is ·nilled· with respect to D, then all of the following are true:

        //   5.2.1 E is locally ·valid· with respect to the ·governing type definition· as defined
        //     by Element Locally Valid (Type) (§3.3.4.4).
        if !element_locally_valid_type(e, governing_type_definition, is_nilled, components) {
            return false;
        }

        if let Some(value_constraint) = d.value_constraint.as_ref() {
            if value_constraint.variety == ValueConstraintVariety::Fixed && !is_nilled {
                // 5.2.2 If D.{value constraint}.{variety} = fixed and E is not ·nilled· with
                //   respect to D, then all of the following are true:

                //   5.2.2.1 E has no element information item [children].
                if e.has_children() {
                    return false;
                }

                //   5.2.2.2 The appropriate case among the following is true:
                if false {
                    // 5.2.2.2.1 If E's ·governing type definition· is a Complex Type Definition
                    //   with {content type}.{variety} = mixed , then the ·initial value· of E
                    //   matches D.{value constraint}.{lexical form}.
                } else {
                    // 5.2.2.2.2 If E's ·governing type definition· is a Simple Type Definition or
                    //   a Complex Type Definition with {content type}.{variety} = simple, then the
                    //   ·actual value· of E is equal or identical to D.{value constraint}.{value}.
                }
            }
        }
    }

    // 6 E is ·valid· with respect to each of the {identity-constraint definitions} as per
    //   Identity-constraint Satisfied (§3.11.4).
    // TODO

    // 7 If E is the ·validation root·, then it is ·valid· per Validation Root Valid (ID/IDREF)
    //   (§3.3.4.5).
    // TODO

    true
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
    )
    .unwrap();

    // for complex in components.complex_type_definitions.as_ref() {
    //     let Some(state_machine) = dt_xsd::state_machine::create_state_machine(complex, &components)
    //     else {
    //         continue;
    //     };
    //     let upa = dt_xsd::state_machine::verify_unique_particle_attribution_satisfied(
    //         &state_machine,
    //         &components,
    //     );
    //     println!("UPA: {:?}", upa);
    // }

    let mut reader = NsReader::from_file(&cli.input).unwrap();

    let mut buffer = Vec::new();
    let mut stack = Vec::<EvalContext>::new();

    struct EvalContextComplex<'a> {
        type_def: &'a ComplexTypeDefinition,
        state_machine: Dfa,
        current_state: u32,
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

                    let mut new_state = None;
                    let mut matching_element_decl = None;
                    // Since the UPA constraint has been validated above, there can only be zero or
                    // one matching transition.
                    for (label, to) in context.state_machine.get_transitions(context.current_state)
                    {
                        match label {
                            Transition::ElementDeclaration(label) => {
                                let element = label.get(&components);
                                if element.target_namespace.as_deref() == namespace
                                    && element.name == local
                                {
                                    new_state = Some(*to);
                                    matching_element_decl = Some(element);
                                    println!("Transition to {:?} ({to})", element.name);
                                    break;
                                }
                            }
                            Transition::Wildcard(label) => {
                                let _wildcard = label.get(&components);
                                // TODO: implement
                            }
                        }
                    }
                    context.current_state = new_state.expect("No matching transition");
                    let matching_element_decl = matching_element_decl.expect("No matching element");

                    stack.push(EvalContext {
                        element: matching_element_decl,
                        type_,
                    });
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
                                    // TODO: cache
                                    let state_machine = dt_xsd::state_machine::create_state_machine(
                                        particle,
                                        &components,
                                    );
                                    let upa = dt_xsd::state_machine::verify_upa_satisfied(
                                        &state_machine,
                                        &components,
                                    );
                                    assert!(upa);
                                    EvalContextType::Complex(EvalContextComplex {
                                        type_def: c,
                                        current_state: state_machine.start_state.unwrap(),
                                        state_machine,
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
                        if context.state_machine.is_end_state(context.current_state) {
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
