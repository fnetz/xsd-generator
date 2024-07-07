use syn::{
    Field, Ident, Item, ItemEnum, Type, __private::Span, parse_quote, FieldMutability, Fields,
    Variant,
};

use dt_xsd::components::IsBuiltinRef;
use dt_xsd::simple_type_def::Variety as SimpleVariety;
use dt_xsd::{
    attribute_decl::ScopeVariety, complex_type_def::ContentType, model_group::Compositor,
    particle::MaxOccurs, AttributeUse, ComplexTypeDefinition, ElementDeclaration, Particle, Ref,
    RefNamed, Schema, SchemaComponentTable, SimpleTypeDefinition, Term, TypeDefinition,
};

use super::common::{ComponentVisitor, GeneratorContext};

use check_keyword::CheckKeyword;
use heck::{ToPascalCase, ToSnakeCase};

struct RustVisitor {
    output_items: Vec<Item>,
}

impl RustVisitor {
    fn new() -> Self {
        Self {
            output_items: Vec::new(),
        }
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

    fn get_type_name_builtin(
        type_def: TypeDefinition,
        components: &SchemaComponentTable,
    ) -> syn::Path {
        assert!(type_def.is_builtin(components));
        let name = type_def
            .name(components)
            .expect("Builtin type without name");
        enum BuiltinSource {
            RustPrimitive,
            HelperType,
        }
        use BuiltinSource::*;
        let (source, name) = match name.local_name.as_str() {
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
        let name = Ident::new(name, Span::call_site());
        match source {
            RustPrimitive => parse_quote!(#name),
            HelperType => parse_quote!(dt_builtin::#name),
        }
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
            Self::get_type_name_builtin(type_def, components)
        } else {
            let name = Self::compute_type_name_ident_non_builtin(type_def, components);
            parse_quote!(#name)
        }
    }

    fn visit_particle(
        &mut self,
        ctx: &mut GeneratorContext,
        particle: &Particle,
    ) -> (Type, String) {
        let (name, type_) = match particle.term {
            Term::ElementDeclaration(element_ref) => {
                let element = element_ref.get(ctx.table);
                if element.scope.variety() == ScopeVariety::Local {
                    self.visit_element_declaration(ctx, element_ref);
                }

                let name = element.name.to_snake_case();
                let type_ = Self::compute_type_name_path(element.type_definition, ctx.table);
                let type_ = parse_quote!(#type_);
                (name, type_)
            }
            Term::ModelGroup(model_group) => {
                let model_group = model_group.get(ctx.table);
                let type_ = match model_group.compositor {
                    Compositor::All | Compositor::Sequence => {
                        let mut members = Vec::new();
                        for particle in model_group.particles.iter().copied() {
                            let particle = particle.get(ctx.table);
                            let (type_, _) = self.visit_particle(ctx, particle);
                            members.push(type_);
                        }
                        parse_quote! { (#(#members),*) }
                    }
                    Compositor::Choice => todo!(),
                };
                ("anon".into(), type_)
            }
            Term::Wildcard(_) => ("wildcard".into(), parse_quote! { () }), //todo!(),
        };

        let type_ = if matches!(particle.max_occurs, MaxOccurs::Count(1)) {
            if particle.min_occurs == 0 {
                parse_quote!(Option<#type_>)
            } else if particle.min_occurs == 1 {
                type_
            } else {
                unreachable!("minOccurs > maxOccurs");
            }
        } else {
            parse_quote!(Vec<#type_>)
        };

        (type_, name)
    }

    fn generate_fields_for_attribute_uses(
        ctx: &mut GeneratorContext,
        attribute_uses: &[Ref<AttributeUse>],
    ) -> Vec<Field> {
        let mut fields = Vec::with_capacity(attribute_uses.len());
        for attribute_use in attribute_uses.iter().copied() {
            let attribute_use = attribute_use.get(ctx.table);
            let attribute_decl = attribute_use.attribute_declaration.get(ctx.table);

            let name = attribute_decl.name.to_snake_case();
            let name = Self::name_to_ident(&name);

            // TODO: visit simple type
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
        ctx: &mut GeneratorContext,
        simple_type_ref: Ref<SimpleTypeDefinition>,
    ) -> Type {
        let simple_type = simple_type_ref.get(ctx.table);

        if let Some(ref name) = simple_type.name {
            // Named type => has a named struct
            let name = Self::name_to_ident(name);
            parse_quote!(#name)
        } else {
            // Unnamed type => create inline type
            let _variety = simple_type.variety.unwrap(); // TODO
            let name = Self::name_to_ident("Placeholder");
            parse_quote!(#name)
        }
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

        match complex_type.content_type {
            ContentType::Empty => {
                if !complex_type.attribute_uses.is_empty() {
                    let fields =
                        Self::generate_fields_for_attribute_uses(ctx, &complex_type.attribute_uses);
                    self.output_items.push(parse_quote! {
                        pub struct #name {
                            #(#fields),*
                        }
                    });
                } else {
                    self.output_items.push(parse_quote! {
                        pub type #name = ();
                    });
                }
            }
            ContentType::Mixed { particle, .. } | ContentType::ElementOnly { particle, .. } => {
                let particle = particle.get(ctx.table);

                let item_: Item = match &particle.term {
                    Term::ElementDeclaration(_) => {
                        let (content, _) = self.visit_particle(ctx, particle);
                        if complex_type.attribute_uses.is_empty() {
                            parse_quote! {
                                pub struct #name(#content)
                            }
                        } else {
                            let fields = Self::generate_fields_for_attribute_uses(
                                ctx,
                                &complex_type.attribute_uses,
                            );
                            parse_quote! {
                                pub struct #name {
                                    inner: #content,
                                    #(#fields),*
                                }
                            }
                        }
                    }
                    Term::ModelGroup(group) => {
                        let group = group.get(ctx.table);
                        match group.compositor {
                            Compositor::All | Compositor::Sequence => {
                                let mut fields = Vec::new();
                                for particle in group.particles.iter().copied() {
                                    let particle = particle.get(ctx.table);
                                    let (type_, name) = self.visit_particle(ctx, particle);
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
                                fields.extend(Self::generate_fields_for_attribute_uses(
                                    ctx,
                                    &complex_type.attribute_uses,
                                ));
                                parse_quote! {
                                    pub struct #name {
                                        #(#fields),*
                                    }
                                }
                            }
                            Compositor::Choice => {
                                let mut variants = Vec::new();
                                for particle in group.particles.iter().copied() {
                                    let particle = particle.get(ctx.table);
                                    let (type_, name) = self.visit_particle(ctx, particle);
                                    let name = Self::name_to_ident(&name.to_pascal_case());
                                    variants.push(Variant {
                                        attrs: vec![],
                                        ident: name,
                                        fields: Fields::Unnamed(parse_quote! { (#type_) }),
                                        discriminant: None,
                                    });
                                }

                                if complex_type.attribute_uses.is_empty() {
                                    parse_quote! {
                                        pub enum #name {
                                            #(#variants),*
                                        }
                                    }
                                } else {
                                    let inner_name = format!("{}Inner", name);
                                    let inner_name = Self::name_to_ident(&inner_name);
                                    let inner_enum: ItemEnum = parse_quote! {
                                        pub enum #inner_name {
                                            #(#variants),*
                                        }
                                    };
                                    self.output_items.push(inner_enum.into());

                                    let fields = Self::generate_fields_for_attribute_uses(
                                        ctx,
                                        &complex_type.attribute_uses,
                                    );
                                    parse_quote! {
                                        pub struct #name {
                                            inner: #inner_name,
                                            #(#fields),*
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Term::Wildcard(_) => todo!(),
                };
                self.output_items.push(item_);
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

                self.output_items.push(parse_quote! {
                    pub struct #name(#simple_type_name);
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

        let item: Item = match simple_type.variety {
            Some(SimpleVariety::Atomic) => {
                // TODO handle built-in primitives
                let primitive_type_ref = simple_type.primitive_type_definition.unwrap();
                if primitive_type_ref == simple_type_ref {
                    // We have a primitive type here
                    return;
                }

                let prim_name = Self::compute_type_name_path(
                    TypeDefinition::Simple(primitive_type_ref),
                    ctx.table,
                );

                // TODO special case for simple alias (without more facets)?
                parse_quote! {
                    pub struct #name(#prim_name);
                }
            }
            Some(SimpleVariety::List) => {
                let item_type = simple_type.item_type_definition.unwrap();
                let item_name =
                    Self::compute_type_name_path(TypeDefinition::Simple(item_type), ctx.table);

                parse_quote! {
                    pub struct #name(Vec<#item_name>);
                }
            }
            Some(SimpleVariety::Union) => {
                let member_types = simple_type.member_type_definitions.as_ref().unwrap();

                let mut variants = Vec::new();
                for member in member_types {
                    let content = Self::visit_simple_type_inline(ctx, *member);
                    let variant_name = if let Some(ref name) = member.get(ctx.table).name {
                        name.as_str()
                    } else {
                        "Unnamed"
                    };
                    let variant_name = variant_name.to_pascal_case();
                    let variant_name = Self::name_to_ident(&variant_name);
                    variants.push(Variant {
                        ident: variant_name,
                        fields: Fields::Unnamed(parse_quote! { (#content) }),
                        attrs: Vec::new(),
                        discriminant: None,
                    });
                }

                parse_quote! {
                    pub enum #name {
                        #(#variants),*
                    }
                }
            }
            None => todo!("anySimpleType"),
        };
        self.output_items.push(item);
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

    let root = syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: visitor.output_items,
    };
    prettyplease::unparse(&root)
}
