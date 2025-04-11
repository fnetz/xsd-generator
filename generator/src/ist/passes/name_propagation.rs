use crate::ist::{Name, Type, builder::IstBuilder};

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
            type_.name = Some(Name::new(format!("Unnamed {suffix} k{key_kind} s{key_id}")));
        }

        match &mut type_.type_ {
            Type::Composite(s) => {
                let mut next_field_id = 0;
                for field in &mut s.members {
                    if field.name.is_none() {
                        field.name = Some(Name::new(format!("UnnamedMember{next_field_id}")));
                        next_field_id += 1;
                    }
                }
            }
            _ => {}
        }
    }
}
