//! This pass basically performs a "dead code elimination" by computing the effective visibility of
//! types. It removes all intermediate types that are not referenced by any other type.

use std::collections::HashSet;

use crate::ist::{TypeIndex, TypeRef, Visibility, builder::IstBuilder};

pub fn reduce_effective_visibility(ist: &mut IstBuilder) {
    let mut keep = HashSet::with_capacity(ist.types.len());
    let mut queue = Vec::with_capacity(ist.types.len());

    for (key, type_) in ist.types.iter() {
        if type_.visibility.must_be_kept() {
            keep.insert(*key);
            queue.push(*key);
        }
    }

    let mut notice_type_ref = |type_ref: &TypeRef, queue: &mut Vec<TypeIndex>| {
        if let TypeRef::Internal(type_id) = type_ref {
            if keep.insert(*type_id) {
                queue.push(*type_id);
            }
        }
    };

    while let Some(type_id) = queue.pop() {
        for child in ist.types[&type_id].type_.children() {
            notice_type_ref(child, &mut queue);
        }
    }

    for (key, type_) in ist.types.iter_mut() {
        if !keep.contains(key) && !type_.visibility.must_be_kept() {
            type_.visibility = Visibility::Discard;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ist::builder::IstBuilder;
    use crate::ist::{Member, Name, Type};

    #[test]
    fn intermediate_referenced_by_public_is_kept() {
        let mut ist = IstBuilder::new();

        // t1 (intermediate) is referenced by t2 (public)
        // -> both should be kept with same visibility

        let type1 = ist.create_type_no_id(
            Type::create_structure(vec![]),
            Some(Name::new("t1".into())),
            Visibility::Intermediate,
            None,
        );
        let type2 = ist.create_type_no_id(
            Type::create_structure(vec![Member::builder(TypeRef::Internal(type1)).build()]),
            Some(Name::new("t2".into())),
            Visibility::Public,
            None,
        );

        reduce_effective_visibility(&mut ist);

        assert_eq!(ist.types[&type1].visibility, Visibility::Intermediate);
        assert_eq!(ist.types[&type2].visibility, Visibility::Public);
    }

    #[test]
    fn intermediate_not_referenced_is_discarded() {
        let mut ist = IstBuilder::new();

        // t1 (intermediate) is not referenced by any other type
        // -> it should be discarded

        let type1 = ist.create_type_no_id(
            Type::create_structure(vec![]),
            Some(Name::new("t1".into())),
            Visibility::Intermediate,
            None,
        );

        reduce_effective_visibility(&mut ist);

        assert_eq!(ist.types[&type1].visibility, Visibility::Discard);
    }

    #[test]
    fn intermediate_referenced_by_intermediate_is_discarded() {
        let mut ist = IstBuilder::new();

        // t1 (intermediate) is referenced by t2 (intermediate)
        // -> both should be discarded

        let type1 = ist.create_type_no_id(
            Type::create_structure(vec![]),
            Some(Name::new("t1".into())),
            Visibility::Intermediate,
            None,
        );
        let type2 = ist.create_type_no_id(
            Type::create_structure(vec![Member::builder(TypeRef::Internal(type1)).build()]),
            Some(Name::new("t2".into())),
            Visibility::Intermediate,
            None,
        );

        reduce_effective_visibility(&mut ist);

        assert_eq!(ist.types[&type1].visibility, Visibility::Discard);
        assert_eq!(ist.types[&type2].visibility, Visibility::Discard);
    }

    #[test]
    fn discard_is_unchanged() {
        let mut ist = IstBuilder::new();

        // t1 (discard) is not referenced by any other type
        // -> it should stay discarded

        let type1 = ist.create_type_no_id(
            Type::create_structure(vec![]),
            Some(Name::new("t1".into())),
            Visibility::Discard,
            None,
        );

        reduce_effective_visibility(&mut ist);

        assert_eq!(ist.types[&type1].visibility, Visibility::Discard);
    }
}
