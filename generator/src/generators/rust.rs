use std::collections::BTreeSet;

use syn::{
    __private::Span, Arm, Block, Expr, ExprIf, ExprMatch, Field, FieldMutability, FieldValue,
    Fields, FieldsUnnamed, Ident, Item, ItemEnum, Member, Pat, Path, Stmt, Token, Type, TypePath,
    Variant, Visibility, parse_quote, punctuated::Punctuated, token::Paren,
};

use dt_xsd::{
    AttributeUse, ComplexTypeDefinition, ElementDeclaration, Particle, Ref, RefNamed, Schema,
    SchemaComponentTable, SimpleTypeDefinition, Term, TypeDefinition,
    attribute_decl::ScopeVariety,
    complex_type_def::ContentType,
    components::{IsBuiltinRef, Named},
    constraining_facet::WhiteSpaceValue,
    model_group::Compositor,
    particle::MaxOccurs,
    simple_type_def::Variety as SimpleVariety,
    state_machine::{self, Action2, Transition},
};

use super::common::{ComponentVisitor, GeneratorContext};

use check_keyword::CheckKeyword;
use heck::{ToPascalCase, ToSnakeCase};

#[derive(Default)]
struct RustVisitor {
    output_items: Vec<Item>,
    unnamed_enums: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BuiltinSource {
    RustPrimitive,
    HelperType,
}

impl BuiltinSource {
    fn name_to_path(&self, name: &str) -> syn::Path {
        let name = Ident::new(name, Span::call_site());
        match self {
            BuiltinSource::RustPrimitive => parse_quote!(#name),
            BuiltinSource::HelperType => parse_quote!(dt_builtins::#name),
        }
    }
}

impl RustVisitor {
    fn new() -> Self {
        Self::default()
    }

    fn name_to_ident(name: &str) -> Ident {
        if ["crate", "self", "super", "Self"].contains(&name) {
            // These are keywords that are not allowed as raw identifiers
            Ident::new(&format!("{}_", name), Span::call_site())
        } else if name.is_keyword() {
            Ident::new_raw(name, Span::call_site())
        } else {
            Ident::new(name, Span::call_site())
        }
    }

    fn get_builtin_source_name(name: dt_xsd::xstypes::QName) -> (BuiltinSource, &'static str) {
        use BuiltinSource::*;
        let (source, name) = match name.local_name.as_ref() {
            "boolean" => (RustPrimitive, "bool"),
            "double" => (RustPrimitive, "f64"),
            "float" => (RustPrimitive, "f32"),
            "long" => (RustPrimitive, "i64"),
            "int" => (RustPrimitive, "i32"),
            "short" => (RustPrimitive, "i16"),
            "byte" => (RustPrimitive, "i8"),
            "unsignedLong" => (RustPrimitive, "u64"),
            "unsignedInt" => (RustPrimitive, "u32"),
            "unsignedShort" => (RustPrimitive, "u16"),
            "unsignedByte" => (RustPrimitive, "u8"),
            "string" => (RustPrimitive, "String"),
            "anyType" => (HelperType, "AnyType"),
            "anySimpleType" => (HelperType, "AnySimpleType"),
            "anyAtomicType" => (HelperType, "AnyAtomicType"),
            "error" => (HelperType, "Error"),
            "decimal" => (HelperType, "Decimal"),
            "dateTime" => (HelperType, "DateTime"),
            "duration" => (HelperType, "Duration"),
            "time" => (HelperType, "Time"),
            "date" => (HelperType, "Date"),
            "gMonth" => (HelperType, "GMonth"),
            "gMonthDay" => (HelperType, "GMonthDay"),
            "gDay" => (HelperType, "GDay"),
            "gYear" => (HelperType, "GYear"),
            "gYearMonth" => (HelperType, "GYearMonth"),
            "hexBinary" => (HelperType, "HexBinary"),
            "base64Binary" => (HelperType, "Base64Binary"),
            "anyURI" => (HelperType, "AnyURI"),
            "QName" => (HelperType, "QName"),
            "NOTATION" => (HelperType, "Notation"),
            "normalizedString" => (HelperType, "NormalizedString"),
            "token" => (HelperType, "Token"),
            "language" => (HelperType, "Language"),
            "NMTOKEN" => (HelperType, "NmToken"),
            "NMTOKENS" => (HelperType, "NmTokens"),
            "Name" => (HelperType, "Name"),
            "NCName" => (HelperType, "NcName"),
            "ID" => (HelperType, "Id"),
            "IDREF" => (HelperType, "IdRef"),
            "IDREFS" => (HelperType, "IdRefs"),
            "ENTITY" => (HelperType, "Entity"),
            "ENTITIES" => (HelperType, "Entities"),
            "integer" => (HelperType, "Integer"),
            "nonPositiveInteger" => (HelperType, "NonPositiveInteger"),
            "negativeInteger" => (HelperType, "NegativeInteger"),
            "nonNegativeInteger" => (HelperType, "NonNegativeInteger"),
            "positiveInteger" => (HelperType, "PositiveInteger"),
            "yearMonthDuration" => (HelperType, "YearMonthDuration"),
            "dayTimeDuration" => (HelperType, "DayTimeDuration"),
            "dateTimeStamp" => (HelperType, "DateTimeStamp"),
            _ => panic!("Unknown builtin type: {:?}", name.local_name),
        };
        (source, name)
    }

    fn get_builtin_type_name(
        type_def: TypeDefinition,
        components: &SchemaComponentTable,
    ) -> syn::Path {
        assert!(type_def.is_builtin(components));
        let name = type_def
            .name(components)
            .expect("Builtin type without name");
        let (source, name) = Self::get_builtin_source_name(name);
        source.name_to_path(name)
    }

    fn compute_type_name_ident_non_builtin(
        type_def: TypeDefinition,
        components: &SchemaComponentTable,
    ) -> Ident {
        assert!(!type_def.is_builtin(components));
        if let Some(name) = type_def.name(components) {
            Self::name_to_ident(&name.local_name.to_pascal_case())
        } else {
            let context_name = match type_def {
                TypeDefinition::Simple(simple_type) => {
                    let simple_type = simple_type.get(components);
                    let context = simple_type
                        .context
                        .as_ref()
                        .expect("context must be set if name is None");
                    use dt_xsd::simple_type_def::Context;
                    match context {
                        Context::Attribute(attribute) => attribute.get(components).name.to_string(),
                        Context::Element(element) => element.get(components).name.to_string(),
                        // TODO: Simple/Complex types need more logic to determine the kind of
                        // subtype we are dealing with
                        Context::ComplexType(c) => {
                            Self::compute_type_name_ident_non_builtin(
                                TypeDefinition::Complex(*c),
                                components,
                            )
                            .to_string()
                                + "Inner"
                        }
                        Context::SimpleType(s) => {
                            Self::compute_type_name_ident_non_builtin(
                                TypeDefinition::Simple(*s),
                                components,
                            )
                            .to_string()
                                + "Inner"
                        }
                    }
                }
                TypeDefinition::Complex(complex_type) => {
                    let complex_type = complex_type.get(components);
                    let context = complex_type
                        .context
                        .as_ref()
                        .expect("context must be set if name is None");
                    use dt_xsd::complex_type_def::Context;
                    match context {
                        Context::Element(element) => element.get(components).name.to_string(),
                        // TODO: Same as above
                        Context::ComplexType(c) => {
                            Self::compute_type_name_ident_non_builtin(
                                TypeDefinition::Complex(*c),
                                components,
                            )
                            .to_string()
                                + "Inner"
                        }
                    }
                }
            };
            Self::name_to_ident(&context_name.to_pascal_case())
        }
    }

    fn compute_type_name_path(
        type_def: TypeDefinition,
        components: &SchemaComponentTable,
    ) -> syn::Path {
        if type_def.is_builtin(components) {
            Self::get_builtin_type_name(type_def, components)
        } else {
            let name = Self::compute_type_name_ident_non_builtin(type_def, components);
            parse_quote!(#name)
        }
    }

    fn visit_particle(
        &mut self,
        ctx: &mut GeneratorContext,
        particle: &Particle,
        path: &mut Vec<Ref<Particle>>,
    ) -> (Type, String) {
        let (type_, name) = match particle.term {
            Term::ElementDeclaration(element_ref) => {
                let element = element_ref.get(ctx.table);
                if element.scope.variety() == ScopeVariety::Local {
                    self.visit_element_declaration(ctx, element_ref);
                }

                let name = element.name.to_snake_case();
                let type_ = Self::compute_type_name_path(element.type_definition, ctx.table);
                let type_ = parse_quote!(#type_);
                (type_, name)
            }
            Term::ModelGroup(model_group) => {
                let model_group = model_group.get(ctx.table);
                match model_group.compositor {
                    Compositor::All | Compositor::Sequence => {
                        let mut members = Vec::new();
                        for particle in model_group.particles.iter().copied() {
                            path.push(particle);
                            let particle = particle.get(ctx.table);
                            let particle = self.visit_particle(ctx, particle, path);
                            path.pop();
                            members.push(particle);
                        }
                        if members.len() == 1 {
                            members.pop().unwrap()
                        } else {
                            let members = members.into_iter().map(|(ty, _)| ty);
                            (parse_quote! { (#(#members),*) }, "anon_sequence".into())
                        }
                    }
                    Compositor::Choice => {
                        let mut variants = Vec::new();
                        for particle in model_group.particles.iter().copied() {
                            path.push(particle);
                            let particle = particle.get(ctx.table);
                            let (type_, name) = self.visit_particle(ctx, particle, path);
                            let name = Self::name_to_ident(&name.to_pascal_case());
                            path.pop();
                            variants.push(Variant {
                                attrs: vec![],
                                ident: name,
                                fields: Fields::Unnamed(parse_quote! { (#type_) }),
                                discriminant: None,
                            });
                        }
                        let choice_name = format!("Choice{}", self.unnamed_enums);
                        let choice_name = Ident::new(&choice_name, Span::call_site());
                        self.unnamed_enums += 1;
                        let choice_enum: ItemEnum = parse_quote! {
                            #[derive(Debug)]
                            pub enum #choice_name {
                                #(#variants),*
                            }
                        };
                        self.output_items.push(choice_enum.into());
                        (parse_quote! { #choice_name }, "anon_choice".into())
                    }
                }
            }
            Term::Wildcard(_) => (parse_quote! { () }, "wildcard".into()), //todo!(),
        };

        let (type_, name) = if matches!(particle.max_occurs, MaxOccurs::Count(1)) {
            if particle.min_occurs == 0 {
                (parse_quote!(Option<#type_>), name)
            } else if particle.min_occurs == 1 {
                (type_, name)
            } else {
                unreachable!("minOccurs > maxOccurs");
            }
        } else {
            (parse_quote!(Vec<#type_>), format!("{}s", name))
        };

        (type_, name)
    }

    fn generate_fields_for_attribute_uses(
        &mut self,
        ctx: &mut GeneratorContext,
        attribute_uses: &[Ref<AttributeUse>],
    ) -> Vec<Field> {
        let mut fields = Vec::with_capacity(attribute_uses.len());
        for attribute_use in attribute_uses.iter().copied() {
            let attribute_use = attribute_use.get(ctx.table);
            let attribute_decl = attribute_use.attribute_declaration.get(ctx.table);

            let name = attribute_decl.name.to_snake_case();
            let name = Self::name_to_ident(&name);

            self.visit_simple_type(ctx, attribute_decl.type_definition);
            let type_name = Self::compute_type_name_path(
                TypeDefinition::Simple(attribute_decl.type_definition),
                ctx.table,
            );
            let type_ = Type::Path(parse_quote!(#type_name));

            let type_ = if !attribute_use.required {
                parse_quote!(Option<#type_>)
            } else {
                type_
            };

            let field: Field = Field {
                attrs: vec![],
                vis: parse_quote!(pub),
                ident: Some(name),
                colon_token: None,
                ty: type_,
                mutability: FieldMutability::None,
            };
            // TODO: value_constraint
            fields.push(field);
        }
        fields
    }

    fn visit_simple_type_inline(
        &mut self,
        ctx: &mut GeneratorContext,
        simple_type_ref: Ref<SimpleTypeDefinition>,
    ) -> Type {
        self.visit_simple_type(ctx, simple_type_ref);

        let path = Self::compute_type_name_path(TypeDefinition::Simple(simple_type_ref), ctx.table);
        Type::Path(TypePath { qself: None, path })
    }

    fn string_variant_to_ident(variant: &str) -> Ident {
        // TODO: unicode-ident
        if variant.is_empty() {
            Ident::new("Empty", Span::call_site())
        } else {
            let first_char = variant.chars().next().unwrap();
            let sanitized_name = variant.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
            // TODO: Improve this (e.g. for "decimal"/version values like 1.0.0)
            if !first_char.is_ascii_alphabetic() {
                Ident::new(
                    &format!("_{}", sanitized_name.to_pascal_case()),
                    Span::call_site(),
                )
            } else {
                Self::name_to_ident(&sanitized_name.to_pascal_case())
            }
        }
    }

    fn particle_to_type_name(&self, particle: Ref<Particle>, ctx: &GeneratorContext) -> Path {
        parse_quote!(Todo)
    }

    fn particle_term_to_type_name(&self, particle: Ref<Particle>, ctx: &GeneratorContext) -> Path {
        todo!()
    }

    fn particle_partial_term_to_type_name(
        &self,
        particle: Ref<Particle>,
        ctx: &GeneratorContext,
    ) -> Type {
        let particle = particle.get(ctx.table);

        match particle.term {
            Term::ElementDeclaration(e) => {
                let e = e.get(ctx.table);
                let t = e.type_definition;
                match t {
                    TypeDefinition::Simple(t) => {
                        let t = t.get(ctx.table);
                        if t.target_namespace.as_deref() == Some("http://www.w3.org/2001/XMLSchema")
                            && t.name.as_deref() == Some("string")
                        {
                            return parse_quote!(Option<String>);
                        }
                    }
                    TypeDefinition::Complex(_) => {}
                }
            }
            _ => {}
        }

        parse_quote!(TodoPartial)
    }
}

impl ComponentVisitor for RustVisitor {
    type ComplexTypeValue = ();
    fn visit_complex_type(
        &mut self,
        ctx: &mut GeneratorContext,
        complex_type: Ref<ComplexTypeDefinition>,
    ) {
        if complex_type.is_builtin(ctx.table) {
            return;
        }
        if !ctx.visited_complex_types.insert(complex_type) {
            return;
        }

        let name = Self::compute_type_name_ident_non_builtin(
            TypeDefinition::Complex(complex_type),
            ctx.table,
        );

        let complex_type = complex_type.get(ctx.table);

        let attribute_fields =
            self.generate_fields_for_attribute_uses(ctx, &complex_type.attribute_uses);

        let attribute_values = complex_type.attribute_uses.iter().map(|attr| {
            let attr = attr.get(ctx.table);
            let name = attr
                .attribute_declaration
                .get(ctx.table)
                .name
                .to_snake_case();
            let name = Self::name_to_ident(&name);
            let decl = attr.attribute_declaration.get(ctx.table);
            let attr_type = Self::compute_type_name_path(
                TypeDefinition::Simple(decl.type_definition),
                ctx.table,
            );
            let raw_name = &decl.name;
            let expr = parse_quote! {
                node.attribute(#raw_name).map(#attr_type::from_string).transpose()?
            };
            let expr = if attr.required {
                parse_quote!(#expr.ok_or(meta::Error::MissingAttribute(#raw_name))?)
            } else {
                expr
            };
            FieldValue {
                attrs: vec![],
                member: Member::Named(name),
                colon_token: Some(Token![:](Span::call_site())),
                expr,
            }
        });

        match complex_type.content_type {
            ContentType::Empty => {
                let final_value: Expr = if !attribute_fields.is_empty() {
                    self.output_items.push(parse_quote! {
                        #[derive(Debug)]
                        pub struct #name {
                            #(#attribute_fields),*
                        }
                    });
                    parse_quote! {
                        Self {
                            #(#attribute_values),*
                        }
                    }
                } else {
                    self.output_items.push(parse_quote! {
                        pub type #name = ();
                    });
                    parse_quote! {
                        Self
                    }
                };

                // 3.4.4.2 Element Locally Valid (Complex Type)
                //  1.1 [...] E has no character or element information item [children].
                self.output_items.push(parse_quote! {
                    impl meta::ComplexType for #name {
                        type Node<'a> = roxmltree::Node<'a, 'a>;
                        fn from_node(node: &Self::Node<'_>) -> Result<Self, meta::Error> {
                            if node.children().any(|n| n.is_element() || n.is_text()) {
                                return Err(meta::Error::ElementOrCharacterInEmptyContentType);
                            }
                            Ok(#final_value)
                        }
                    }
                });
            }
            ContentType::Mixed { particle, .. } | ContentType::ElementOnly { particle, .. } => {
                let state_machine = state_machine::create_state_machine(particle, ctx.table);
                let initial_state = state_machine.start_state.unwrap();
                let initial_state_name =
                    Ident::new(&format!("State{initial_state}"), Span::call_site());

                let particle_field = |particle: Ref<Particle>| -> Field {
                    Field {
                        attrs: vec![],
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: {
                            let type_name = self.particle_to_type_name(particle, ctx);
                            parse_quote!(Vec<#type_name>)
                        },
                    }
                };
                let term_field = |particle: Ref<Particle>| -> Field {
                    Field {
                        attrs: vec![],
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: {
                            let type_name = self.particle_partial_term_to_type_name(particle, ctx);
                            parse_quote!(#type_name)
                        },
                    }
                };

                let state_variants =
                    state_machine
                        .states
                        .iter()
                        .enumerate()
                        .map(|(state_index, state)| {
                            let state_name =
                                Ident::new(&format!("State{state_index}"), Span::call_site());
                            Variant {
                                attrs: vec![],
                                ident: state_name,
                                fields: Fields::Unnamed(FieldsUnnamed {
                                    paren_token: Paren::default(),
                                    unnamed: Punctuated::from_iter(
                                        state
                                            .context
                                            .iter()
                                            .copied()
                                            .flat_map(|p| [particle_field(p), term_field(p)]),
                                    ),
                                }),
                                discriminant: None,
                            }
                        });

                let state_enum: ItemEnum = parse_quote! {
                    #[derive(Debug)]
                    enum State {
                        #( #state_variants ),*
                    }
                };

                let mut arms = Vec::new();

                for (state_index, state) in state_machine.states.iter().enumerate() {
                    let state_name = Ident::new(&format!("State{state_index}"), Span::call_site());
                    for (event, (target, actions)) in state.transitions.iter() {
                        match event {
                            Transition::ElementDeclaration(event) => {
                                let event = event.get(ctx.table);
                                let name = &event.name;
                                let namespace: Pat = match &event.target_namespace {
                                    Some(ns) => parse_quote!(Some(#ns)),
                                    None => parse_quote!(None),
                                };

                                let mut lstk = state.context.len();

                                let mut stmts = Vec::<Stmt>::new();

                                for action in actions {
                                    match action {
                                        Action2::CommitTerm => {
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            let particle_field_name = Ident::new(
                                                &format!("p{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            stmts.push(parse_quote! {
                                        #particle_field_name.push(#term_field_name.finalize());
                                    });
                                            stmts.push(parse_quote! {
                                                let #term_field_name = TodoPartial::new();
                                            });
                                        }
                                        Action2::BeginParticleAndTerm(p) => {
                                            let field_name =
                                                Ident::new(&format!("p{lstk}"), Span::call_site());
                                            let type_name = self.particle_to_type_name(*p, ctx);
                                            stmts.push(parse_quote! {
                                                let mut #field_name = Vec::<#type_name>::new();
                                            });

                                            let field_name =
                                                Ident::new(&format!("t{lstk}"), Span::call_site());
                                            // let type_name = self.particle_partial_term_to_type_name(p);
                                            stmts.push(parse_quote! {
                                                let mut #field_name = TodoPartial::new();
                                            });
                                            lstk += 1;
                                        }
                                        Action2::EndTermParticleAndStore(p, s) => {
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            let particle_field_name = Ident::new(
                                                &format!("p{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            // let type_name = self.particle_partial_term_to_type_name(p);
                                            stmts.push(parse_quote! {
                                        #particle_field_name.push(#term_field_name.finalize());
                                    });

                                            let type_name = self.particle_to_type_name(*p, ctx);

                                            lstk -= 1;
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            stmts.push(parse_quote! {
                                        #term_field_name.insert(#type_name::finalize(#particle_field_name), #s);
                                    });
                                        }
                                        Action2::EndTermParticleAndTerminate(_) => unimplemented!(),
                                        Action2::StoreEmpty(p, s) => {
                                            let type_name = self.particle_to_type_name(*p, ctx);

                                            let term_field_name = Ident::new(
                                                &format!("f{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            stmts.push(parse_quote! {
                                        #term_field_name.insert(#type_name::finalize(vec::<#type_name>::new()), #s);
                                    });
                                        }
                                        Action2::TerminateEmpty(_) => unimplemented!(),
                                    }
                                }

                                let target_state = &state_machine.states[*target as usize];

                                let mut args = Vec::<Expr>::new();
                                for (i, particle) in target_state.context.iter().enumerate() {
                                    let field_name =
                                        Ident::new(&format!("p{}", i), Span::call_site());
                                    args.push(parse_quote! { #field_name });

                                    let field_name =
                                        Ident::new(&format!("t{}", i), Span::call_site());
                                    args.push(parse_quote! { #field_name });
                                }

                                let target_name =
                                    Ident::new(&format!("State{target}"), Span::call_site());
                                let arm = Arm {
                                    attrs: vec![],
                                    pat: parse_quote! { (State::#state_name(), #namespace, #name) },
                                    guard: None,
                                    fat_arrow_token: Default::default(),
                                    body: parse_quote! { {
                                        #(#stmts)*
                                        State::#target_name(#(#args),*)
                                    } },
                                    comma: Some(Default::default()),
                                };
                                arms.push(arm);
                            }
                            Transition::Wildcard(_) => {
                                // TODO: Wildcard
                                eprintln!("Ignoring wildcard transition");
                                continue;
                            }
                            Transition::Eof => {
                                assert!(state_machine.is_end_state(*target));

                                let mut lstk = state.context.len();

                                let mut stmts = Vec::<Stmt>::new();

                                for action in actions {
                                    match action {
                                        Action2::CommitTerm => {
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            let particle_field_name = Ident::new(
                                                &format!("p{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            stmts.push(parse_quote! {
                                        #particle_field_name.push(#term_field_name.finalize());
                                    });
                                            stmts.push(parse_quote! {
                                                let #term_field_name = TodoPartial::new();
                                            });
                                        }
                                        Action2::BeginParticleAndTerm(p) => {
                                            let field_name =
                                                Ident::new(&format!("p{lstk}"), Span::call_site());
                                            let type_name = self.particle_to_type_name(*p, ctx);
                                            stmts.push(parse_quote! {
                                                let mut #field_name = Vec::<#type_name>::new();
                                            });

                                            let field_name =
                                                Ident::new(&format!("t{lstk}"), Span::call_site());
                                            // let type_name = self.particle_partial_term_to_type_name(p);
                                            stmts.push(parse_quote! {
                                                let mut #field_name = TodoPartial::new();
                                            });
                                            lstk += 1;
                                        }
                                        Action2::EndTermParticleAndStore(p, s) => {
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            let particle_field_name = Ident::new(
                                                &format!("p{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            // let type_name = self.particle_partial_term_to_type_name(p);
                                            stmts.push(parse_quote! {
                                        #particle_field_name.push(#term_field_name.finalize());
                                    });

                                            let type_name = self.particle_to_type_name(*p, ctx);

                                            lstk -= 1;
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            stmts.push(parse_quote! {
                                        #term_field_name.insert(#type_name::finalize(#particle_field_name), #s);
                                    });
                                        }
                                        Action2::EndTermParticleAndTerminate(p) => {
                                            let term_field_name = Ident::new(
                                                &format!("t{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            let particle_field_name = Ident::new(
                                                &format!("p{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            // let type_name = self.particle_partial_term_to_type_name(p);
                                            stmts.push(parse_quote! {
                                        #particle_field_name.push(#term_field_name.finalize());
                                    });

                                            let type_name = self.particle_to_type_name(*p, ctx);
                                            stmts.push(parse_quote! {
                                        return Ok(#type_name::finalize(#particle_field_name));
                                    });
                                        }
                                        Action2::StoreEmpty(p, s) => {
                                            let type_name = self.particle_to_type_name(*p, ctx);

                                            let term_field_name = Ident::new(
                                                &format!("f{}", lstk - 1),
                                                Span::call_site(),
                                            );
                                            stmts.push(parse_quote! {
                                        #term_field_name.insert(#type_name::finalize(vec::<#type_name>::new()), #s);
                                    });
                                        }
                                        Action2::TerminateEmpty(p) => {
                                            let type_name = self.particle_to_type_name(*p, ctx);

                                            stmts.push(parse_quote! {
                                        return Ok(#type_name::finalize(vec::<#type_name>::new()));
                                    });
                                        }
                                    }
                                }

                                let arm = Arm {
                                    attrs: vec![],
                                    pat: parse_quote! { (State::#state_name(), None, None) },
                                    guard: None,
                                    fat_arrow_token: Default::default(),
                                    body: parse_quote! { {
                                        #(#stmts)*
                                    } },
                                    comma: Some(Default::default()),
                                };
                                arms.push(arm);
                            }
                        }
                    }
                }

                arms.push(parse_quote! {
                    (_, _, _) => return Err(meta::Error::NoValidBranch),
                });

                let tt: ExprMatch = parse_quote! {
                    match (state, namespace_name, name) {
                        #(#arms)*
                    }
                };

                let mut path = Vec::new();

                let particle = particle.get(ctx.table);
                let (type_item, impl_block): (Item, Block) = match &particle.term {
                    Term::ElementDeclaration(_) => {
                        let (content, _) = self.visit_particle(ctx, particle, &mut path);
                        if attribute_fields.is_empty() {
                            (
                                parse_quote! {
                                    #[derive(Debug)]
                                    pub struct #name(#content)
                                },
                                parse_quote! { {
                                    let value = #content::from_node(node)?;
                                    Ok(Self(value))
                                } },
                            )
                        } else {
                            (
                                parse_quote! {
                                    #[derive(Debug)]
                                    pub struct #name {
                                        inner: #content,
                                        #(#attribute_fields),*
                                    }
                                },
                                parse_quote! { {
                                    let value = #content::from_node(node)?;
                                    Ok(Self {
                                        inner: value,
                                        #(#attribute_values),*
                                    })
                                } },
                            )
                        }
                    }
                    Term::ModelGroup(group) => {
                        let group = group.get(ctx.table);
                        match group.compositor {
                            Compositor::All | Compositor::Sequence => {
                                let mut fields = Vec::new();
                                for particle in group.particles.iter().copied() {
                                    let particle = particle.get(ctx.table);
                                    let (type_, name) =
                                        self.visit_particle(ctx, particle, &mut path);
                                    let name = Self::name_to_ident(&name);
                                    let field: Field = Field {
                                        attrs: vec![],
                                        vis: parse_quote!(pub),
                                        ident: Some(name),
                                        colon_token: None,
                                        ty: type_,
                                        mutability: FieldMutability::None,
                                    };
                                    fields.push(field);
                                }
                                let names = fields
                                    .iter()
                                    .map(|f| f.ident.as_ref().unwrap().clone())
                                    .collect::<BTreeSet<_>>();
                                fields.extend(attribute_fields.into_iter().map(|f| {
                                    if names.contains(f.ident.as_ref().unwrap()) {
                                        Field {
                                            ident: Some(Ident::new(
                                                &(f.ident.unwrap().to_string() + "_attr"),
                                                Span::call_site(),
                                            )),
                                            ..f
                                        }
                                    } else {
                                        f
                                    }
                                }));
                                (
                                    parse_quote! {
                                        #[derive(Debug)]
                                        pub struct #name {
                                            #(#fields),*
                                        }
                                    },
                                    parse_quote! { {
                                        #state_enum
                                        let mut state = State::#initial_state_name;
                                        for element in node.children().filter(|n| n.is_element()) {
                                            let name = element.tag_name().name();
                                            let namespace_name = element.tag_name().namespace();
                                            state = #tt;
                                        }
                                        todo!()
                                    } },
                                )
                            }
                            Compositor::Choice => {
                                let mut variants = Vec::new();
                                for particle in group.particles.iter().copied() {
                                    let particle = particle.get(ctx.table);
                                    let (type_, name) =
                                        self.visit_particle(ctx, particle, &mut path);
                                    let name = Self::name_to_ident(&name.to_pascal_case());
                                    variants.push(Variant {
                                        attrs: vec![],
                                        ident: name,
                                        fields: Fields::Unnamed(parse_quote! { (#type_) }),
                                        discriminant: None,
                                    });
                                }

                                if complex_type.attribute_uses.is_empty() {
                                    (
                                        parse_quote! {
                                            #[derive(Debug)]
                                            pub enum #name {
                                                #(#variants),*
                                            }
                                        },
                                        parse_quote! {{ todo!() }},
                                    )
                                } else {
                                    let inner_name = format!("{}Inner", name);
                                    let inner_name = Self::name_to_ident(&inner_name);
                                    let inner_enum: ItemEnum = parse_quote! {
                                        #[derive(Debug)]
                                        pub enum #inner_name {
                                            #(#variants),*
                                        }
                                    };
                                    self.output_items.push(inner_enum.into());

                                    (
                                        parse_quote! {
                                            #[derive(Debug)]
                                            pub struct #name {
                                                inner: #inner_name,
                                                #(#attribute_fields),*
                                            }
                                        },
                                        parse_quote! {{ todo!() }},
                                    )
                                }
                            }
                        }
                    }
                    Term::Wildcard(_) => todo!(),
                };
                self.output_items.push(type_item);
                self.output_items.push(parse_quote! {
                    impl meta::ComplexType for #name {
                        type Node<'a> = roxmltree::Node<'a, 'a>;
                        fn from_node(node: &Self::Node<'_>) -> Result<Self, meta::Error>
                            #impl_block
                    }
                });
            }
            ContentType::Simple {
                simple_type_definition,
            } => {
                // Just a wrapper around a simple type for now
                self.visit_simple_type(ctx, simple_type_definition);

                let simple_type_name = Self::compute_type_name_path(
                    TypeDefinition::Simple(simple_type_definition),
                    ctx.table,
                );

                let from_string: Expr = if simple_type_definition.is_builtin(ctx.table) {
                    // TODO: dedup
                    let (source, _) = Self::get_builtin_source_name(
                        simple_type_definition.get(ctx.table).name().unwrap(),
                    );

                    match source {
                        BuiltinSource::RustPrimitive => parse_quote! {
                            dt_builtins::PrimitiveType::<#simple_type_name>::from_string(&initial_value)?.into_inner()
                        },
                        BuiltinSource::HelperType => {
                            parse_quote! { #simple_type_name::from_string(&initial_value)? }
                        }
                    }
                } else {
                    parse_quote! { #simple_type_name::from_string(&initial_value)? }
                };

                let final_value: Expr = if attribute_fields.is_empty() {
                    self.output_items.push(parse_quote! {
                        #[derive(Debug)]
                        pub struct #name(#simple_type_name);
                    });

                    parse_quote! {
                        Self(value)
                    }
                } else {
                    self.output_items.push(parse_quote! {
                        #[derive(Debug)]
                        pub struct #name {
                            pub inner: #simple_type_name,
                            #(#attribute_fields),*
                        }
                    });
                    parse_quote! {
                        Self {
                            inner: value,
                            #(#attribute_values),*
                        }
                    }
                };

                // [T]he initial value of an element information item is the string composed of, in
                // order, the [character code] of each character information item in the [children]
                // of that element information item.
                // - Pt. 1, 3.1.4 White Space Normalization during Validation
                //
                // node.text() returns "a first text child" for elements, which is not what we want.
                // TODO: is CDATA handled correctly?
                self.output_items.push(parse_quote! {
                    impl meta::ComplexType for #name {
                        type Node<'a> = roxmltree::Node<'a, 'a>;
                        fn from_node(node: &Self::Node<'_>) -> Result<Self, meta::Error> {
                            debug_assert!(node.is_element());
                            if node.children().any(|n| n.is_element()) {
                                return Err(meta::Error::ElementInSimpleContentType);
                            }
                            let initial_value = node.children()
                                .filter(|n| n.is_text())
                                .map(|n| n.text().unwrap())
                                .collect::<String>();
                            let value = #from_string;
                            Ok(#final_value)
                        }
                    }
                });
            }
        }
    }

    type SimpleTypeValue = ();
    fn visit_simple_type(
        &mut self,
        ctx: &mut GeneratorContext,
        simple_type_ref: Ref<SimpleTypeDefinition>,
    ) {
        if simple_type_ref.is_builtin(ctx.table) {
            return;
        }
        if !ctx.visited_simple_types.insert(simple_type_ref) {
            return;
        }

        let simple_type = simple_type_ref.get(ctx.table);

        let name = Self::compute_type_name_ident_non_builtin(
            TypeDefinition::Simple(simple_type_ref),
            ctx.table,
        );
        let whitespace = simple_type
            .facets
            .white_space(ctx.table)
            .map(|x| x.value)
            .unwrap_or(
                // Default according to Pt. 2, Section 4.3.6 whiteSpace
                WhiteSpaceValue::Collapse,
            );
        let (type_def, impl_block) = match simple_type.variety {
            Some(SimpleVariety::Atomic) => {
                // TODO handle built-in primitives
                let primitive_type_ref = simple_type.primitive_type_definition.unwrap();
                if primitive_type_ref == simple_type_ref {
                    // We have a primitive type here
                    return;
                }

                // NOTE: According to Pt. 2, 4.1.1, {primitive type definition} must be a
                // ·primitive· built-in definition if not absent.
                let primitive_type = &primitive_type_ref.get(ctx.table);
                let (prim_source, prim_name_raw) = Self::get_builtin_source_name(
                    primitive_type.name().expect("primitive type without name"),
                );
                let prim_name = prim_source.name_to_path(prim_name_raw);

                let whitespace_ident: Ident = match whitespace {
                    WhiteSpaceValue::Preserve => parse_quote!(Preserve),
                    WhiteSpaceValue::Replace => parse_quote!(Replace),
                    WhiteSpaceValue::Collapse => parse_quote!(Collapse),
                };
                // let prim_name_raw = Ident::new(prim_name_raw, Span::call_site());

                let from_literal: Expr = match prim_source {
                    BuiltinSource::RustPrimitive => parse_quote! {
                        dt_builtins::PrimitiveType::<#prim_name>::from_literal(normalized)?.into_inner()
                    },
                    BuiltinSource::HelperType => {
                        parse_quote! { #prim_name::from_literal(normalized)? }
                    }
                };

                if let Some(enumeration) = simple_type
                    .facets
                    .enumerations(ctx.table)
                    .filter(|_| primitive_type.name.as_deref().unwrap() == "string")
                {
                    // TODO: Non-string enumerations
                    let enum_members = enumeration
                        .value
                        .iter()
                        .map(|value| (value, Self::string_variant_to_ident(value)))
                        .collect::<Vec<_>>();

                    let variants = enum_members.iter().map(|(value, name)| Variant {
                        ident: name.clone(),
                        fields: Fields::Unit,
                        attrs: vec![{
                            let c = format!("Enumeration value for `` {value} ``");
                            parse_quote!(#[doc = #c])
                        }],
                        discriminant: None,
                    });

                    // TODO: equality according to facets
                    let type_def: Item = parse_quote! {
                        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
                        pub enum #name {
                            #(#variants),*
                        }
                    };
                    let match_arms = enum_members.iter().map(|(value, name)| -> Arm {
                        parse_quote! {
                            #value => Ok(Self::#name),
                        }
                    });
                    let impl_block: Item = parse_quote! {
                        impl meta::SimpleType for #name {
                            const FACET_WHITE_SPACE: Option<meta::Whitespace>
                                = Some(meta::Whitespace::#whitespace_ident);
                            fn from_literal(normalized: &str) -> Result<Self, meta::Error> {
                                let value = #from_literal;
                                match value.as_str() {
                                    #(#match_arms)*
                                    _ => Err(meta::Error::ValueNotInEnumeration(value.to_string())),
                                }
                            }
                        }
                    };
                    (type_def, impl_block)
                } else {
                    // TODO special case for simple alias (without more facets)?
                    let type_def: Item = parse_quote! {
                        #[derive(Debug)]
                        pub struct #name(pub #prim_name);
                    };

                    // Relevant sections:
                    // - Pt. 1, 3.16.4 Simple Type Definition Validation Rules; Validation Rule: String Valid
                    // - Pt. 2, 4.1.4 Simple Type Definition Validation Rules; Validation Rule: Datatype Valid
                    //   - 1 pattern valid (TODO)
                    //   - 2.1 (atomic variety)
                    //   - 3 facet valid (TODO)
                    // let value = dt_builtins::#prim_name_raw::from_literal(normalized)?;
                    let impl_block: Item = parse_quote! {
                        impl meta::SimpleType for #name {
                            const FACET_WHITE_SPACE: Option<meta::Whitespace>
                                = Some(meta::Whitespace::#whitespace_ident);
                            fn from_literal(normalized: &str) -> Result<Self, meta::Error> {
                                let value = #from_literal;
                                Ok(Self(value))
                            }
                        }
                    };
                    (type_def, impl_block)
                }
            }
            Some(SimpleVariety::List) => {
                let item_type = simple_type.item_type_definition.unwrap();
                let item_name =
                    Self::compute_type_name_path(TypeDefinition::Simple(item_type), ctx.table);

                let type_def: Item = parse_quote! {
                    #[derive(Debug)]
                    pub struct #name(pub Vec<#item_name>);
                };
                // whiteSpace:
                //   "For all datatypes ·constructed· by ·list· the value of whiteSpace is collapse
                //   and cannot be changed by a schema author"
                //   (Pt. 2, 4.3.6 whiteSpace)
                debug_assert_eq!(whitespace, WhiteSpaceValue::Collapse);
                let impl_block: Item = parse_quote! {
                    impl meta::SimpleType for #name {
                        const FACET_WHITE_SPACE: Option<meta::Whitespace> = Some(dt_builtins::meta::Whitespace::Collapse);
                        fn from_literal(normalized: &str) -> Result<Self, meta::Error> {
                            let list = normalized.split(' ').map(|s| {
                                #item_name::from_literal(s)
                            }).collect::<Result<Vec<_>, _>>()?;
                            Ok(Self(list))
                        }
                    }
                };
                (type_def, impl_block)
            }
            Some(SimpleVariety::Union) => {
                let member_types = simple_type.member_type_definitions.as_ref().unwrap();

                let mut variants = Vec::new();
                let mut branches = Vec::new();
                for member in member_types {
                    let content = self.visit_simple_type_inline(ctx, *member);
                    let variant_name = if let Some(ref name) = member.get(ctx.table).name {
                        name.as_str()
                    } else {
                        "Unnamed"
                    };
                    let variant_name = variant_name.to_pascal_case();
                    let variant_name = Self::name_to_ident(&variant_name);
                    branches.push(ExprIf {
                        attrs: vec![],
                        if_token: Token![if](Span::call_site()),
                        cond: Box::new(Expr::Let(
                            parse_quote! { let Ok(value) = #content::from_string(s) },
                        )),
                        then_branch: parse_quote! { { Ok(Self::#variant_name(value)) } },
                        else_branch: None,
                    });
                    variants.push(Variant {
                        ident: variant_name,
                        fields: Fields::Unnamed(parse_quote! { (#content) }),
                        attrs: Vec::new(),
                        discriminant: None,
                    });
                }

                let mut if_chain = Expr::Block(parse_quote! {
                    { Err(meta::Error::NoValidBranch) }
                });
                for branch in branches.into_iter().rev() {
                    if_chain = Expr::If(ExprIf {
                        else_branch: Some((Token![else](Span::call_site()), Box::new(if_chain))),
                        ..branch
                    });
                }

                let type_def: Item = parse_quote! {
                    #[derive(Debug)]
                    pub enum #name {
                        #(#variants),*
                    }
                };
                let impl_block: Item = parse_quote! {
                    impl meta::SimpleType for #name {
                        const FACET_WHITE_SPACE: Option<meta::Whitespace> = None;
                        fn from_literal(s: &str) -> Result<Self, meta::Error> {
                            #if_chain
                        }
                    }
                };
                (type_def, impl_block)
            }
            None => todo!("anySimpleType"),
        };
        self.output_items.push(type_def);
        self.output_items.push(impl_block);
    }

    type ElementDeclarationValue = ();
    fn visit_element_declaration(
        &mut self,
        ctx: &mut GeneratorContext,
        element: Ref<ElementDeclaration>,
    ) {
        let element = element.get(ctx.table);
        let type_def = element.type_definition;
        match type_def {
            TypeDefinition::Simple(simple_type) => {
                self.visit_simple_type(ctx, simple_type);
            }
            TypeDefinition::Complex(complex_type) => {
                self.visit_complex_type(ctx, complex_type);
            }
        }
    }
}

pub fn generate(schema: &Schema, components: &SchemaComponentTable) -> String {
    let mut ctx = GeneratorContext::new(components);
    let mut visitor = RustVisitor::new();

    visitor.output_items.push(Item::Use(parse_quote!(
        use dt_builtins::meta;
    )));
    visitor.output_items.push(Item::Use(parse_quote!(
        use meta::SimpleType as _;
    )));

    for type_def in schema.type_definitions.iter().copied() {
        match type_def {
            TypeDefinition::Complex(complex_type) => {
                visitor.visit_complex_type(&mut ctx, complex_type);
            }
            TypeDefinition::Simple(simple_type) => {
                visitor.visit_simple_type(&mut ctx, simple_type);
            }
        }
    }

    for element in schema.element_declarations.iter().copied() {
        visitor.visit_element_declaration(&mut ctx, element);
    }

    let doc_comment = concat!(
        "Generated by ",
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    );
    let root = syn::File {
        shebang: None,
        attrs: vec![
            parse_quote!(#![doc = #doc_comment]),
            parse_quote!(#![allow(dead_code, unused_imports)]),
        ],
        items: visitor.output_items,
    };
    prettyplease::unparse(&root)
}
