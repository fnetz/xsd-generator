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

    fn compute_type_name_ident(
        type_def: TypeDefinition,
        components: &SchemaComponentTable,
    ) -> Ident {
        if type_def.is_builtin(components) {
            let name = type_def
                .name(components)
                .expect("Builtin type without name");
            let name = match name.local_name.as_str() {
                "boolean" => "bool",
                "double" => "f64",
                "float" => "f32",
                "long" => "i64",
                "int" => "i32",
                "short" => "i16",
                "byte" => "i8",
                "unsignedLong" => "u64",
                "unsignedInt" => "u32",
                "unsignedShort" => "u16",
                "unsignedByte" => "u8",
                // TODO
                _ => return Self::name_to_ident(&name.local_name.to_pascal_case()),
            };
            Ident::new(name, Span::call_site())
        } else if let Some(name) = type_def.name(components) {
            Self::name_to_ident(&name.local_name.to_pascal_case())
        } else {
            // TODO determine from context
            Ident::new("UnnamedTypeTodo", Span::call_site())
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
                let type_ = if let Some(type_) = element.type_definition.name(ctx.table) {
                    // TODO qualify
                    type_.local_name.to_pascal_case()
                } else {
                    element.name.to_pascal_case()
                };
                let type_ = Self::name_to_ident(&type_);
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
            let type_name = Self::compute_type_name_ident(
                TypeDefinition::Simple(attribute_decl.type_definition),
                ctx.table,
            );

            let field: Field = Field {
                attrs: vec![],
                vis: parse_quote!(pub),
                ident: Some(name),
                colon_token: None,
                ty: parse_quote!(#type_name),
                mutability: FieldMutability::None,
            };
            // TODO: required, value_constraint
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
        if !ctx.visited_complex_types.insert(complex_type) {
            return;
        }

        let name = Self::compute_type_name_ident(TypeDefinition::Complex(complex_type), ctx.table);

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
                                if complex_type.attribute_uses.is_empty() {
                                    parse_quote! {
                                        pub enum #name {

                                        }
                                    }
                                } else {
                                    let inner_name = format!("{}Inner", name);
                                    let inner_name = Self::name_to_ident(&inner_name);
                                    let inner_enum: ItemEnum = parse_quote! {
                                        pub enum #inner_name {
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

                let simple_type_name = Self::compute_type_name_ident(
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
        if !ctx.visited_simple_types.insert(simple_type_ref) {
            return;
        }

        let simple_type = simple_type_ref.get(ctx.table);

        let name =
            Self::compute_type_name_ident(TypeDefinition::Simple(simple_type_ref), ctx.table);

        let item: Item = match simple_type.variety {
            Some(SimpleVariety::Atomic) => {
                // TODO handle built-in primitives
                let primitive_type_ref = simple_type.primitive_type_definition.unwrap();
                if primitive_type_ref == simple_type_ref {
                    // We have a primitive type here
                    return;
                }

                let prim_name = Self::compute_type_name_ident(
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
                    Self::compute_type_name_ident(TypeDefinition::Simple(item_type), ctx.table);

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
