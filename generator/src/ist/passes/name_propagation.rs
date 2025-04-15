use std::collections::HashMap;

use hstr::Atom;

use crate::ist::{CompositeType, Type, builder::IstBuilder};

/// Fills in unnamed types and fields with dummy names
/// so the generated code is more readable.
/// This is a temporary solution until we have a better way to handle this.
pub fn fill_unnamed_types(ist: &mut IstBuilder, skip_discarded: bool) {
    for (key, type_) in ist.types.iter_mut() {
        if skip_discarded && type_.visibility.is_discard() {
            continue;
        }

        if type_.name.is_none() {
            let suffix = match type_.type_ {
                Type::Composite(_) => "Structure",
                Type::Enum(_) => "Enum",
            };
            let (key_kind, key_id) = key.sort_key();
            type_.name = Some(Atom::new(format!("Unnamed {suffix} k{key_kind} s{key_id}")));
        }

        match &mut type_.type_ {
            Type::Composite(s) => {
                fill_composite_fields(s);
            }
            _ => {}
        }
    }
}

fn fill_composite_fields(composite: &mut CompositeType) {
    let mut names = HashMap::<Atom, Vec<usize>>::new();

    for (i, field) in composite.members.iter().enumerate() {
        if let Some(name) = &field.name {
            names.entry(name.clone()).or_default().push(i);
        }
    }

    for (name, indices) in names.iter() {
        if indices.len() > 1 {
            // Conflict resolution: Lowest index gets to keep its name, others get new names
            let min = *indices.iter().min().unwrap();
            let mut next_field_id = 1;
            for i in indices.iter().filter(|&&i| i != min) {
                // TODO: Check for conflicts with existing names
                composite.members[*i].name = Some(Atom::new(format!("{name}_{next_field_id}")));
                next_field_id += 1;
            }
        }
    }

    let mut next_field_id = 1;
    for field in &mut composite.members {
        if field.name.is_none() {
            field.name = Some(Atom::new(format!("unnamed_member_{next_field_id}")));
            next_field_id += 1;
        }
    }
}
