use crate::ist::{Field, Name, Quant, Type, TypeBinding, TypeRef, builder::IstBuilder};

#[derive(Debug)]
pub struct InlineSettings {
    pub inline_quantified_into_field: bool,
}

fn merge_inlined_name(replaced_field: &Field, inlined_field: &Field) -> Option<Name> {
    // TODO: Expand logic
    match (replaced_field.name.as_ref(), inlined_field.name.as_ref()) {
        (Some(name), Some(inlined_name)) => {
            if name.name == inlined_name.name {
                Some(name.clone())
            } else {
                Some(Name::new(format!("{}_{}", name.name, inlined_name.name)))
            }
        }
        (Some(name), None) | (None, Some(name)) => Some(name.clone()),
        (None, None) => None,
    }
}

/// Tries to inline a field into the parent structure, returning true if the field was inlined, and
/// false otherwise.
fn inline_field(
    field: &Field,
    new_fields: &mut Vec<Field>,
    ist: &IstBuilder,
    settings: &InlineSettings,
) -> bool {
    let TypeRef::Internal(ref field_type) = field.type_ else {
        return false;
    };

    let field_type: &TypeBinding = &ist.types[&field_type];
    if !field_type.inline {
        return false;
    }

    match field_type.type_ {
        Type::Structure(ref sub) => {
            new_fields.extend(sub.fields.iter().map(|sub_field| Field {
                name: merge_inlined_name(field, sub_field),
                type_: sub_field.type_.clone(),
                source: field.source.clone().inlined(),
                documentation: field.documentation.clone(),
                quant: sub_field.quant,
            }));
            true
        }
        Type::Quantified(ref q) => {
            // If the type is a quantified type and has range 1..1, or if the
            // target supports inlining of non-default quantified types, we can
            // inline the field
            if q.quant == Quant::exactly_one() || settings.inline_quantified_into_field {
                new_fields.push(Field {
                    name: field_type.name.clone(), // TODO
                    type_: q.type_.clone(),
                    source: field.source.clone().inlined(),
                    documentation: field.documentation.clone(),
                    quant: q.quant,
                });
                true
            } else {
                // Otherwise, we just add the field as is
                false
            }
        }
        _ => {
            // For other types, we just add the field as is
            false
        }
    }
}

pub fn do_inlining_step(ist: &mut IstBuilder, settings: &InlineSettings) {
    for k in ist.types.keys().cloned().collect::<Vec<_>>() {
        let outer = &ist.types[&k];

        if !outer.type_.children().any(|t| t.wants_inlining(&ist.types)) {
            continue;
        }

        match outer.type_ {
            Type::Structure(ref s) => {
                let mut new_fields = Vec::new();

                for field in s.fields.iter() {
                    // Don't inline fields containing the current type
                    if field.type_.as_internal() == Some(k) {
                        continue;
                    }

                    if !inline_field(field, &mut new_fields, ist, settings) {
                        new_fields.push(field.clone());
                    }
                }

                ist.types
                    .get_mut(&k)
                    .unwrap()
                    .type_
                    .as_structure_mut()
                    .unwrap()
                    .fields = new_fields;
            }
            Type::Quantified(ref outer) => {
                let TypeRef::Internal(ref inner_type) = outer.type_ else {
                    continue;
                };

                // Don't inline if the type is the same as the outer type
                if *inner_type == k {
                    continue;
                }

                let inner_type: &TypeBinding = &ist.types[&inner_type];

                if !inner_type.inline {
                    continue;
                }

                // Skip for now if the inner type has a name to prevent loss of information
                if inner_type.name.is_some() {
                    continue;
                }

                match inner_type.type_ {
                    Type::Quantified(ref inner_type) => {
                        let new_quant = if inner_type.quant == Quant::exactly_one() {
                            outer.quant
                        } else if outer.quant == Quant::exactly_one() {
                            inner_type.quant
                        } else {
                            continue;
                        };
                        let new_type = inner_type.type_.clone();

                        let outer = ist
                            .types
                            .get_mut(&k)
                            .unwrap()
                            .type_
                            .as_quantified_mut()
                            .unwrap();
                        outer.type_ = new_type;
                        outer.quant = new_quant;
                    }
                    _ => {}
                }
            }
            Type::Union(ref outer) => {
                // let mut new_variants = Vec::new();

                for member in outer.variants.iter() {
                    let TypeRef::Internal(ref inner_type) = member.type_ else {
                        continue;
                    };

                    // Don't inline members containing the current type
                    if *inner_type == k {
                        continue;
                    }

                    let inner_type: &TypeBinding = &ist.types[&inner_type];

                    if !inner_type.inline {
                        continue;
                    }

                    // Skip for now if the inner type has a name to prevent loss of information
                    if inner_type.name.is_some() {
                        continue;
                    }

                    // TODO
                }
            }
            _ => {}
        }
    }
}

pub fn perform_inlining(ist: &mut IstBuilder, settings: &InlineSettings) {
    // TODO: This is horrible.
    for _ in 0..ist.types.len() {
        do_inlining_step(ist, settings);
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
    use crate::ist::{ExternalKind, FieldSource, Name, QuantifiedType, StructureType, Visibility};

    #[test]
    fn inline_struct_into_struct() {
        let mut ist = IstBuilder::new();
        let a = ist.create_type_no_id(
            Type::Structure(StructureType {
                fields: vec![Field {
                    name: Some(Name::new("field_a_1".into())),
                    type_: TypeRef::External(
                        QName::without_namespace("dummy_type"),
                        ExternalKind::TypeDefinition,
                    ),
                    source: FieldSource::Term,
                    documentation: None,
                    quant: Quant::default(),
                }],
            }),
            Some(Name::new("struct_a".into())),
            Visibility::Public,
            None,
        );
        ist.types.get_mut(&a).unwrap().inline = true;

        let _b = ist.create_type_no_id(
            Type::Structure(StructureType {
                fields: vec![Field {
                    name: Some(Name::new("field_b_1".into())),
                    type_: TypeRef::Internal(a),
                    source: FieldSource::Term,
                    documentation: None,
                    quant: Quant::default(),
                }],
            }),
            Some(Name::new("struct_b".into())),
            Visibility::Public,
            None,
        );

        let settings = InlineSettings {
            inline_quantified_into_field: false,
        };
        do_inlining_step(&mut ist, &settings);

        // Check that the field was inlined
        let b = ist.types.get(&_b).unwrap();
        let b = b.type_.as_structure().expect("Expected structure type");
        assert_eq!(b.fields.len(), 1);
        assert_eq!(
            b.fields[0].name.as_ref().unwrap().name,
            "field_b_1_field_a_1"
        );
        assert_eq!(
            b.fields[0].type_.as_external().unwrap().0.local_name(),
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
            Type::Structure(StructureType {
                fields: vec![Field {
                    name: Some(Name::new("field_a_1".into())),
                    type_: TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(dummy_ref())),
                    source: FieldSource::Term,
                    documentation: None,
                    quant: Quant::default(),
                }],
            }),
            Some(Name::new("struct_a".into())),
            Visibility::Public,
            None,
        );

        let settings = InlineSettings {
            inline_quantified_into_field: false,
        };
        println!("Inlining with settings: {:?}", settings);
        do_inlining_step(&mut ist, &settings);

        // Check that the field was not inlined
        let t = ist.types.get(&t).unwrap();
        let t = t.type_.as_structure().expect("Expected structure type");
        assert_eq!(t.fields.len(), 1);
        assert_eq!(t.fields[0].name.as_ref().unwrap().name, "field_a_1");
        assert!(matches!(
            t.fields[0].type_,
            TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(_))
        ));
    }

    #[test]
    fn inline_internal_quant_and_builtin_simple_into_struct() {
        let mut ist = IstBuilder::new();
        let a = ist.create_type_no_id(
            Type::Quantified(QuantifiedType {
                type_: TypeRef::External(
                    QName::without_namespace("dummy_type"),
                    ExternalKind::AttributeDeclaration,
                ),
                quant: Quant::exactly_one(),
            }),
            Some(Name::new("struct_a".into())),
            Visibility::Public,
            None,
        );
        ist.types.get_mut(&a).unwrap().inline = true;

        let _b = ist.create_type_no_id(
            Type::Structure(StructureType {
                fields: vec![
                    Field {
                        name: Some(Name::new("field_b_1".into())),
                        type_: TypeRef::Internal(a),
                        source: FieldSource::AttributeUse,
                        documentation: None,
                        quant: Quant::default(),
                    },
                    Field {
                        name: Some(Name::new("field_b_2".into())),
                        type_: TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(dummy_ref())),
                        source: FieldSource::SimpleContent,
                        documentation: None,
                        quant: Quant::default(),
                    },
                ],
            }),
            Some(Name::new("struct_b".into())),
            Visibility::Public,
            None,
        );

        let settings = InlineSettings {
            inline_quantified_into_field: false,
        };
        do_inlining_step(&mut ist, &settings);

        // Check that the field was inlined
        let b = ist.types.get(&_b).unwrap();
        let b = b.type_.as_structure().expect("Expected structure type");
        assert_eq!(b.fields.len(), 2);
        assert_eq!(b.fields[0].name.as_ref().unwrap().name, "field_b_1");
        assert_eq!(
            b.fields[0].type_.as_external().unwrap().0.local_name(),
            "dummy_type"
        );
        assert_eq!(b.fields[1].name.as_ref().unwrap().name, "field_b_2");
        assert!(matches!(
            b.fields[1].type_,
            TypeRef::Builtin(dt_xsd::TypeDefinition::Simple(_))
        ));
    }
}
