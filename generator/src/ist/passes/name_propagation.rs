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
                Type::Structure(_) => "Structure",
                Type::Enum(_) => "Enum",
                Type::Union(_) => "Union",
                // Type::Quantified(_) => "Quantified",
            };
            let (key_kind, key_id) = key.sort_key();
            type_.name = Some(Name::new(format!("Unnamed {suffix} k{key_kind} s{key_id}")));
        }

        match &mut type_.type_ {
            Type::Structure(s) => {
                let mut next_field_id = 0;
                for field in &mut s.fields {
                    if field.name.is_none() {
                        field.name = Some(Name::new(format!("UnnamedField{next_field_id}")));
                        next_field_id += 1;
                    }
                }
            }
            Type::Union(u) => {
                let mut next_member_id = 0;
                for member in &mut u.variants {
                    if member.name.is_none() {
                        member.name = Some(Name::new(format!("UnnamedMember{next_member_id}")));
                        next_member_id += 1;
                    }
                }
            }
            _ => {}
        }
    }
}
