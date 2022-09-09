mod naming;

use std::collections::HashSet;

use naming::{CamelCase, PascalCase, SnakeCase};

use syn::{Field, Ident, Item, ItemEnum, Type, __private::Span, parse_quote};

use crate::xsd::complex_type_def::Context as ComplexContext;
use crate::xsd::simple_type_def::Context as SimpleContext;
use crate::xsd::simple_type_def::Variety as SimpleVariety;
use crate::xsd::{
    attribute_decl::ScopeVariety, complex_type_def::ContentTypeVariety, model_group::Compositor,
    particle::MaxOccurs, ComplexTypeDefinition, ElementDeclaration, Particle, Ref, RefNamed,
    Schema, SchemaComponentTable, SimpleTypeDefinition, Term, TypeDefinition,
};

struct GeneratorContext<'a> {
    table: &'a SchemaComponentTable,
    output_items: Vec<Item>,
    visited_complex_types: HashSet<Ref<ComplexTypeDefinition>>,
    visited_simple_types: HashSet<Ref<SimpleTypeDefinition>>,
}

impl<'a> GeneratorContext<'a> {
    fn new(table: &'a SchemaComponentTable) -> Self {
        Self {
            table,
            output_items: Vec::new(),
            visited_complex_types: HashSet::new(),
            visited_simple_types: HashSet::new(),
        }
    }
}

fn visit_particle(ctx: &mut GeneratorContext, particle: &Particle) -> (Type, String) {
    let (name, type_) = match particle.term {
        crate::xsd::Term::ElementDeclaration(element) => {
            let element = element.get(ctx.table);
            if element.scope.variety() == ScopeVariety::Local {
                visit_element_decl(ctx, element);
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
        crate::xsd::Term::ModelGroup(model_group) => {
            let model_group = model_group.get(ctx.table);
            let type_ = match model_group.compositor {
                Compositor::All | Compositor::Sequence => {
                    let mut members = Vec::new();
                    for particle in model_group.particles.iter().copied() {
                        let particle = particle.get(ctx.table);
                        let (type_, _) = visit_particle(ctx, particle);
                        members.push(type_);
                    }
                    parse_quote! { (#(#members),*) }
                }
                Compositor::Choice => todo!(),
            };
            ("anon".into(), type_)
        }
        crate::xsd::Term::Wildcard(_) => ("wildcard".into(), parse_quote! { () }), //todo!(),
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

fn visit_complex_type(ctx: &mut GeneratorContext, complex_type: Ref<ComplexTypeDefinition>) {
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

    if complex_type.content_type.variety == ContentTypeVariety::Empty {
        ctx.output_items.push(parse_quote! {
            pub type #name = ();
        });
        return;
    }

    let particle = complex_type.content_type.particle.unwrap().get(ctx.table);

    let item_: Item = match &particle.term {
        Term::ElementDeclaration(_) => {
            let (content, _) = visit_particle(ctx, particle);
            if complex_type.attribute_uses.is_empty() {
                parse_quote! {
                    pub struct #name(#content)
                }
            } else {
                // TODO attributes
                parse_quote! {
                    pub struct #name {
                        inner: #content
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
                        let (type_, name) = visit_particle(ctx, particle);
                        let field: Field = Field {
                            attrs: vec![],
                            vis: parse_quote!(pub),
                            ident: Some(Ident::new(&name, Span::call_site())),
                            colon_token: None,
                            ty: type_,
                        };
                        fields.push(field);
                    }
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
                        ctx.output_items.push(inner_enum.into());
                        parse_quote! {
                            pub struct #name {
                                inner: #inner_name,
                            }
                        }
                    }
                }
            }
        }
        Term::Wildcard(_) => todo!(),
    };
    ctx.output_items.push(item_);
}

fn get_simple_type_name(ctx: &mut GeneratorContext, simple_type: &SimpleTypeDefinition) -> String {
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

fn visit_simple_type(ctx: &mut GeneratorContext, simple_type_ref: Ref<SimpleTypeDefinition>) {
    if !ctx.visited_simple_types.insert(simple_type_ref) {
        return;
    }

    let simple_type = simple_type_ref.get(ctx.table);

    let name = get_simple_type_name(ctx, simple_type);
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
            let prim_name = get_simple_type_name(ctx, primitive_type);
            let prim_name = Ident::new(&prim_name, Span::call_site());
            // TODO special case for simple alias (without more facets)?
            parse_quote! {
                pub struct #name(#prim_name);
            }
        }
        Some(SimpleVariety::List) => {
            let item_type = simple_type.item_type_definition.unwrap().get(ctx.table);
            let item_name = get_simple_type_name(ctx, item_type);
            let item_name = Ident::new(&item_name, Span::call_site());

            parse_quote! {
                pub type #name = Vec<#item_name>;
            }
        }
        Some(SimpleVariety::Union) => {
            // TODO
            parse_quote! {
                pub enum #name {}
            }
        }
        None => todo!("anySimpleType"),
    };
    ctx.output_items.push(item);
}

fn visit_element_decl(ctx: &mut GeneratorContext, element: &ElementDeclaration) {
    let type_def = element.type_definition;
    match type_def {
        TypeDefinition::Simple(simple_type) => {
            visit_simple_type(ctx, simple_type);
        }
        TypeDefinition::Complex(complex_type) => {
            visit_complex_type(ctx, complex_type);
        }
    }
}

pub fn generate_rust(schema: &Schema, components: &SchemaComponentTable) -> String {
    let mut ctx = GeneratorContext::new(components);

    for type_def in schema.type_definitions.iter().copied() {
        match type_def {
            TypeDefinition::Complex(complex_type) => {
                visit_complex_type(&mut ctx, complex_type);
            }
            TypeDefinition::Simple(simple_type) => {
                visit_simple_type(&mut ctx, simple_type);
            }
        }
    }

    for element in schema.element_declarations.iter().copied() {
        let element = element.get(ctx.table);
        visit_element_decl(&mut ctx, element);
    }

    let root = syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: ctx.output_items,
    };
    prettyplease::unparse(&root)
}
