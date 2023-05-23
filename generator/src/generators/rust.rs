use crate::naming::{self, CamelCase, PascalCase, SnakeCase};

use syn::{
    Field, Ident, Item, ItemEnum, Type, __private::Span, parse_quote, FieldMutability, Fields,
    Variant,
};

use dt_xsd::complex_type_def::Context as ComplexContext;
use dt_xsd::simple_type_def::Context as SimpleContext;
use dt_xsd::simple_type_def::Variety as SimpleVariety;
use dt_xsd::{
    attribute_decl::ScopeVariety,
    complex_type_def::{ContentType, ContentTypeVariety},
    model_group::Compositor,
    particle::MaxOccurs,
    AttributeUse, ComplexTypeDefinition, ElementDeclaration, Particle, Ref, RefNamed, Schema,
    SchemaComponentTable, SimpleTypeDefinition, Term, TypeDefinition,
};

use super::common::ComponentVisitor;
use super::common::GeneratorContext;

struct RustVisitor {
    output_items: Vec<Item>,
}

impl RustVisitor {
    fn new() -> Self {
        Self {
            output_items: Vec::new(),
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

                let name = naming::convert::<CamelCase, SnakeCase>(&element.name);
                let type_ = if let Some(type_) = element.type_definition.name(ctx.table) {
                    // TODO qualify
                    naming::convert::<CamelCase, PascalCase>(&type_.local_name)
                } else {
                    naming::convert::<CamelCase, PascalCase>(&element.name)
                };
                let type_ = Ident::new(&type_, Span::call_site());
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

            let name = naming::convert::<CamelCase, SnakeCase>(&attribute_decl.name);

            // TODO: visit simple type
            let type_def = attribute_decl.type_definition.get(ctx.table);
            let type_name = Self::get_simple_type_name(ctx, type_def);
            let type_name = Ident::new(&type_name, Span::call_site());

            let field: Field = Field {
                attrs: vec![],
                vis: parse_quote!(pub),
                ident: Some(Ident::new(&name, Span::call_site())),
                colon_token: None,
                ty: parse_quote!(#type_name),
                mutability: FieldMutability::None,
            };
            // TODO: required, value_constraint
            fields.push(field);
        }
        fields
    }

    fn get_simple_type_name(
        ctx: &mut GeneratorContext,
        simple_type: &SimpleTypeDefinition,
    ) -> String {
        simple_type
            .name
            .as_deref()
            .or_else(|| {
                simple_type
                    .context
                    .as_ref()
                    .and_then(|context| match context {
                        SimpleContext::ComplexType(ct) => ct.get(ctx.table).name.as_deref(),
                        SimpleContext::SimpleType(st) => st.get(ctx.table).name.as_deref(),
                        SimpleContext::Element(e) => Some(e.get(ctx.table).name.as_str()),
                        SimpleContext::Attribute(a) => Some(a.get(ctx.table).name.as_str()),
                    })
            })
            // TODO convert in caller
            .map(naming::convert::<CamelCase, PascalCase>)
            .unwrap_or_else(|| "UnnamedType".into()) // TODO can name be none here?
    }

    fn visit_simple_type_inline(
        ctx: &mut GeneratorContext,
        simple_type_ref: Ref<SimpleTypeDefinition>,
    ) -> Type {
        let simple_type = simple_type_ref.get(ctx.table);

        if let Some(ref name) = simple_type.name {
            // Named type => has a named struct
            let name = Ident::new(name, Span::call_site());
            parse_quote!(#name)
        } else {
            // Unnamed type => create inline type
            let _variety = simple_type.variety.unwrap(); // TODO
            let name = Ident::new("Placeholder", Span::call_site());
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

        let complex_type = complex_type.get(ctx.table);

        let name = complex_type
            .name
            .as_deref()
            .or_else(|| {
                complex_type
                    .context
                    .as_ref()
                    .and_then(|context| match context {
                        ComplexContext::ComplexType(ct) => ct.get(ctx.table).name.as_deref(),
                        ComplexContext::Element(e) => Some(e.get(ctx.table).name.as_str()),
                    })
            })
            .map(naming::convert::<CamelCase, PascalCase>)
            .unwrap_or_else(|| "UnnamedType".into()); // TODO can name be none here?

        let name = Ident::new(&name, Span::call_site());

        if complex_type.content_type.variety() == ContentTypeVariety::Empty {
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
            return;
        }

        let (ContentType::Mixed { particle, .. } | ContentType::ElementOnly { particle, .. }) =
            complex_type.content_type else {
                todo!("Simple content type")
            };

        let particle = particle.get(ctx.table);

        let item_: Item = match &particle.term {
            Term::ElementDeclaration(_) => {
                let (content, _) = self.visit_particle(ctx, particle);
                if complex_type.attribute_uses.is_empty() {
                    parse_quote! {
                        pub struct #name(#content)
                    }
                } else {
                    let fields =
                        Self::generate_fields_for_attribute_uses(ctx, &complex_type.attribute_uses);
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
                            let field: Field = Field {
                                attrs: vec![],
                                vis: parse_quote!(pub),
                                ident: Some(Ident::new(&name, Span::call_site())),
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
                            let inner_name = Ident::new(&inner_name, Span::call_site());
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

        let name = Self::get_simple_type_name(ctx, simple_type);
        let name = Ident::new(&name, Span::call_site());

        let item: Item = match simple_type.variety {
            Some(SimpleVariety::Atomic) => {
                // TODO handle built-in primitives
                let primitive_type_ref = simple_type.primitive_type_definition.unwrap();
                if primitive_type_ref == simple_type_ref {
                    // We have a primitive type here
                    return;
                }

                let primitive_type = primitive_type_ref.get(ctx.table);
                let prim_name = Self::get_simple_type_name(ctx, primitive_type);
                let prim_name = Ident::new(&prim_name, Span::call_site());
                // TODO special case for simple alias (without more facets)?
                parse_quote! {
                    pub struct #name(#prim_name);
                }
            }
            Some(SimpleVariety::List) => {
                let item_type = simple_type.item_type_definition.unwrap().get(ctx.table);
                let item_name = Self::get_simple_type_name(ctx, item_type);
                let item_name = Ident::new(&item_name, Span::call_site());

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
                    let variant_name = Ident::new(variant_name, Span::call_site());
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
