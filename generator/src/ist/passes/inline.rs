use std::{borrow::Cow, collections::HashMap};

use hstr::Atom;

use crate::ist::{
    CompositeType, Member, Quant, Type, TypeBinding, TypeIndex, TypeRef, builder::IstBuilder,
};

#[derive(Debug)]
pub struct InlineSettings {}

fn merge_names_str<'a>(left: Option<&'a str>, right: Option<&'a str>) -> Option<Cow<'a, str>> {
    match (left, right) {
        (Some(left), Some(right)) => Some(format!("{}_{}", left, right).into()),
        (Some(single), None) | (None, Some(single)) => Some(single.into()),
        (None, None) => None,
    }
}

fn merge_names(left: Option<&Atom>, right: Option<&Atom>) -> Option<Atom> {
    merge_names_str(left.map(|v| &**v), right.map(|v| &**v)).map(Atom::new)
}

/// Tries to inline a field into the parent structure, returning true if the field was inlined, and
/// false otherwise.
fn inline_field(
    sup: &CompositeType,
    field: &Member,
    new_fields: &mut Vec<Member>,
    ist: &IstBuilder,
    _settings: &InlineSettings,
) -> bool {
    let TypeRef::Internal(ref field_type) = field.type_ else {
        return false;
    };

    let field_type: &TypeBinding = &ist.types[field_type];
    if !field_type.inline {
        return false;
    }

    let Type::Composite(ref inf) = field_type.type_ else {
        // If the field is not a structure, we can't inline it here
        return false;
    };

    if inf.is_thin() {
        // thin -> has single field
        let sub_field = inf.members.first().unwrap();

        let quant = if sub_field.quant == Quant::default() {
            field.quant
        } else if field.quant == Quant::default() {
            sub_field.quant
        } else {
            // Can't inline if both fields are quantified
            return false;
        };

        let name = match (&field.name, &field_type.name, &sub_field.name) {
            // trivial, no name -> no name
            (None, None, None) => None,

            // mostly trivial, single name -> keep the name
            (Some(single), None, None)
            | (None, Some(single), None)
            | (None, None, Some(single)) => Some(single.clone()),

            // field type AND sub field name -> merge
            // (not that common)
            (None, Some(field_type_name), Some(sub_field_name)) => {
                merge_names(Some(field_type_name), Some(sub_field_name))
            }

            // inlined field name AND sub field name -> merge
            // (also not that common)
            (Some(inlined_field_name), None, Some(sub_field_name)) => {
                merge_names(Some(inlined_field_name), Some(sub_field_name))
            }

            // inlined field name AND field type -> don't inline at all
            // (not inlining is usually better in this case, since this is pretty much the target state anyway)
            (Some(_inlined_field_name), Some(_field_type_name), None | Some(_)) => {
                return false;
            }
        };

        new_fields.push(Member {
            name,
            type_: sub_field.type_.clone(),
            source: field.source.clone().inlined(),
            documentation: field.documentation.clone(),
            quant,
        });
    } else {
        if inf.compositor != sup.compositor {
            // We can't inline a field if the compositor is different
            return false;
        }

        if field.quant != Quant::default() {
            // If the field is already quantified and the substructure is not single-fielded,
            // we can't inline it (yet)
            return false;
        }

        if field.name.is_some() {
            // We don't want to replace a named field with a multi-fielded structure.
            return false;
        }

        new_fields.extend(inf.members.iter().map(|sub_field| Member {
            name: merge_names(field_type.name.as_ref(), sub_field.name.as_ref()),
            type_: sub_field.type_.clone(),
            source: field.source.clone().inlined(),
            documentation: field.documentation.clone(),
            quant: sub_field.quant,
        }));
    }

    true
}

fn try_replace_whole_type(
    sup_index: TypeIndex,
    sup: &CompositeType,
    ist: &IstBuilder,
) -> Option<Type> {
    if !sup.is_thin() {
        return None;
    }
    let single_field = sup.members.first().unwrap();

    if single_field.quant != Quant::default() {
        // We wont't inline a quantified field
        return None;
    }

    if single_field.name.is_some() {
        // We don't want to inline a named field
        return None;
    }

    let TypeRef::Internal(inf) = single_field.type_ else {
        return None;
    };

    if inf == sup_index {
        // We don't want to inline a type into itself
        return None;
    }

    let inf = &ist.types[&inf];

    if !inf.inline || inf.name.is_some() {
        // We only want to replace types with unnamed, inlineable inferior types
        return None;
    }

    Some(inf.type_.clone())
}

pub fn do_inlining_step_on_type(
    k: TypeIndex,
    ist: &mut IstBuilder,
    settings: &InlineSettings,
) -> bool {
    let sup = &ist.types[&k];

    if !sup.type_.children().any(|t| t.wants_inlining(&ist.types)) {
        return false;
    }

    match sup.type_ {
        Type::Composite(ref s) => {
            // First, check if the type is viable for replacement
            if let Some(replacement) = try_replace_whole_type(k, s, ist) {
                ist.types.get_mut(&k).unwrap().type_ = replacement;
                return true;
            }

            let mut new_fields = Vec::new();
            let mut was_modified = false;

            for field in s.members.iter() {
                // Don't inline fields containing the current type
                if field.type_.as_internal() == Some(k) {
                    continue;
                }

                if !inline_field(s, field, &mut new_fields, ist, settings) {
                    new_fields.push(field.clone());
                } else {
                    was_modified = true;
                }
            }

            if was_modified {
                ist.types
                    .get_mut(&k)
                    .unwrap()
                    .type_
                    .as_structure_mut()
                    .unwrap()
                    .members = new_fields;
            } else {
                debug_assert_eq!(new_fields.len(), s.members.len());
            }

            was_modified
        }
        _ => false,
    }
}

fn compute_incoming_references(ist: &IstBuilder) -> HashMap<TypeIndex, Vec<TypeIndex>> {
    let mut incoming_references = HashMap::<TypeIndex, Vec<TypeIndex>>::new();

    for (k, v) in ist.types.iter() {
        for type_ in v.type_.children().filter_map(|t| t.as_internal()) {
            incoming_references.entry(type_).or_default().push(*k);
        }
    }

    incoming_references
}

pub fn perform_inlining(ist: &mut IstBuilder, settings: &InlineSettings) {
    let incoming_references = compute_incoming_references(ist);

    let mut queue = ist.types.keys().cloned().collect::<Vec<_>>();

    while let Some(k) = queue.pop() {
        if do_inlining_step_on_type(k, ist, settings) {
            // If we modified the type, we need to recheck all types that reference it
            if let Some(references) = incoming_references.get(&k) {
                for reference in references {
                    if !queue.contains(reference) {
                        queue.push(*reference);
                    }
                }
            }

            // Also recheck the type itself
            if !queue.contains(&k) {
                queue.push(k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use dt_xsd::Ref;
    use dt_xsd::components::{Component, ComponentTraits, HasArenaContainer};
    use dt_xsd::xstypes::QName;

    use super::*;
    use crate::ist::builder::IstBuilder;
    use crate::ist::{ExternalKind, FieldSource, Visibility};

    #[test]
    fn incoming_references_correct() {
        let mut ist = IstBuilder::new();
        let a = ist.create_type_no_id(
            Type::create_structure(vec![]),
            Some(Atom::new("struct_a")),
            Visibility::Public,
            None,
        );
        let b = ist.create_type_no_id(
            Type::create_structure(vec![Member {
                name: Some(Atom::new("field_b_1")),
                type_: TypeRef::Internal(a),
                source: FieldSource::Term,
                documentation: None,
                quant: Quant::default(),
            }]),
            Some(Atom::new("struct_b")),
            Visibility::Public,
            None,
        );

        let incoming_references = compute_incoming_references(&ist);
        assert_eq!(incoming_references.len(), 1);
        assert_eq!(incoming_references[&a], vec![b]);
    }

    fn do_inlining_step(ist: &mut IstBuilder, settings: &InlineSettings) {
        for k in ist.types.keys().cloned().collect::<Vec<_>>() {
            do_inlining_step_on_type(k, ist, settings);
        }
    }

    #[test]
    fn inline_struct_into_struct() {
        let mut ist = IstBuilder::new();
        let a = ist.create_type_no_id(
            Type::create_structure(vec![Member {
                name: Some(Atom::new("field_a_1")),
                type_: TypeRef::External(
                    QName::without_namespace("dummy_type"),
                    ExternalKind::TypeDefinition,
                ),
                source: FieldSource::Term,
                documentation: None,
                quant: Quant::default(),
            }]),
            None,
            // Some(Name::new("struct_a".into())),
            Visibility::Public,
            None,
        );
        ist.types.get_mut(&a).unwrap().inline = true;

        let _b = ist.create_type_no_id(
            Type::create_structure(vec![Member {
                name: Some(Atom::new("field_b_1")),
                type_: TypeRef::Internal(a),
                source: FieldSource::Term,
                documentation: None,
                quant: Quant::default(),
            }]),
            Some(Atom::new("struct_b")),
            Visibility::Public,
            None,
        );

        let settings = InlineSettings {};
        do_inlining_step(&mut ist, &settings);

        // Check that the field was inlined
        let b = ist.types.get(&_b).unwrap();
        let b = b.type_.as_structure().expect("Expected structure type");
        assert_eq!(b.members.len(), 1);
        assert_eq!(b.members[0].name.as_ref().unwrap(), "field_b_1_field_a_1");
        assert_eq!(
            b.members[0].type_.as_external().unwrap().0.local_name(),
            "dummy_type"
        );
    }

    // TODO: This is a hack to produce a dummy reference to a builtin type. We should construct a
    // proper builtin, but this will do for now, since we don't actually resolve the type.
    fn dummy_ref<T: Component>() -> Ref<T>
    where
        ComponentTraits: HasArenaContainer<T>,
    {
        // SAFETY: `Ref` is marked as `#[repr(transparent)]`, so we can safely transmute a
        // `NonZeroU32` into it.
        //
        // ```
        // #[repr(transparent)]
        // struct Ref<R>(NonZeroU32, PhantomData<R>)
        // ```
        let dummy_ref: Ref<T> = unsafe { std::mem::transmute(NonZeroU32::new(42424242).unwrap()) };
        dummy_ref
    }

    #[test]
    fn not_inlining_builtin_simple_into_struct() {
        let mut ist = IstBuilder::new();
        let t = ist.create_type_no_id(
            Type::create_structure(vec![Member {
                name: Some(Atom::new("field_a_1")),
                type_: TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(dummy_ref())),
                source: FieldSource::Term,
                documentation: None,
                quant: Quant::default(),
            }]),
            Some(Atom::new("struct_a")),
            Visibility::Public,
            None,
        );

        let settings = InlineSettings {};
        println!("Inlining with settings: {:?}", settings);
        do_inlining_step(&mut ist, &settings);

        // Check that the field was not inlined
        let t = ist.types.get(&t).unwrap();
        let t = t.type_.as_structure().expect("Expected structure type");
        assert_eq!(t.members.len(), 1);
        assert_eq!(t.members[0].name.as_ref().unwrap(), "field_a_1");
        assert!(matches!(
            t.members[0].type_,
            TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(_))
        ));
    }

    #[test]
    fn inline_internal_quant_and_builtin_simple_into_struct() {
        let mut ist = IstBuilder::new();
        let a = ist.create_type_no_id(
            Type::create_quantified(
                TypeRef::External(
                    QName::without_namespace("dummy_type"),
                    ExternalKind::AttributeDeclaration,
                ),
                Quant::exactly_one(),
            ),
            Some(Atom::new("struct_a")),
            Visibility::Public,
            None,
        );
        ist.types.get_mut(&a).unwrap().inline = true;

        let _b = ist.create_type_no_id(
            Type::create_structure(vec![
                Member {
                    name: None, // Some(Name::new("field_b_1".into())),
                    type_: TypeRef::Internal(a),
                    source: FieldSource::AttributeUse,
                    documentation: None,
                    quant: Quant::default(),
                },
                Member {
                    name: Some(Atom::new("field_b_2")),
                    type_: TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(dummy_ref())),
                    source: FieldSource::SimpleContent,
                    documentation: None,
                    quant: Quant::default(),
                },
            ]),
            Some(Atom::new("struct_b")),
            Visibility::Public,
            None,
        );

        let settings = InlineSettings {};
        do_inlining_step(&mut ist, &settings);

        // Check that the field was inlined
        let b = ist.types.get(&_b).unwrap();
        let b = b.type_.as_structure().expect("Expected structure type");
        assert_eq!(b.members.len(), 2);
        assert_eq!(b.members[0].name.as_ref().unwrap(), "struct_a"); // TODO
        assert_eq!(
            b.members[0].type_.as_external().unwrap().0.local_name(),
            "dummy_type"
        );
        assert_eq!(b.members[1].name.as_ref().unwrap(), "field_b_2");
        assert!(matches!(
            b.members[1].type_,
            TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(_))
        ));
    }
}
