mod naming;

use naming::{CamelCase, PascalCase, SnakeCase};

use syn::{Field, Ident, Item, ItemEnum, Type, __private::Span, parse_quote};

use crate::xsd::{
    attribute_decl::ScopeVariety, model_group::Compositor, particle::MaxOccurs,
    ComplexTypeDefinition, ElementDeclaration, Particle, RefNamed, Schema, SchemaComponentTable,
    Term, TypeDefinition,
};

fn visit_particle(
    items: &mut Vec<Item>,
    particle: &Particle,
    components: &SchemaComponentTable,
) -> (Type, String) {
    let (name, type_) = match particle.term {
        crate::xsd::Term::ElementDeclaration(element) => {
            let element = element.get(components);
            if element.scope.variety() == ScopeVariety::Local {
                visit_element_decl(items, element, components);
            }

            let name = naming::convert::<CamelCase, SnakeCase>(&element.name);
            let type_ = if let Some(type_) = element.type_definition.name(components) {
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
            let model_group = model_group.get(components);
            let type_ = match model_group.compositor {
                Compositor::All | Compositor::Sequence => {
                    let mut members = Vec::new();
                    for particle in model_group.particles.iter().copied() {
                        let particle = particle.get(components);
                        let (type_, _) = visit_particle(items, particle, components);
                        members.push(type_);
                    }
                    parse_quote! { (#(#members),*) }
                }
                Compositor::Choice => todo!(),
            };
            ("anon".into(), type_)
        }
        crate::xsd::Term::Wildcard(_) => todo!(),
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

fn visit_complex_type(
    items: &mut Vec<Item>,
    complex_type: &ComplexTypeDefinition,
    components: &SchemaComponentTable,
    parent_name: Option<&str>,
) {
    let particle = complex_type.content_type.particle.unwrap().get(components);

    let name = match (parent_name, complex_type.name.as_ref()) {
        (Some(name), None) => name,
        (_, Some(_)) => todo!("named complex type"),
        (None, None) => todo!("both unnamed"), // TODO use context
    };
    let name = Ident::new(name, Span::call_site());

    let item_: Item = match &particle.term {
        Term::ElementDeclaration(_) => {
            let (content, _) = visit_particle(items, particle, components);
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
            let group = group.get(components);
            match group.compositor {
                Compositor::All | Compositor::Sequence => {
                    let mut fields = Vec::new();
                    for particle in group.particles.iter().copied() {
                        let particle = particle.get(components);
                        let (type_, name) = visit_particle(items, particle, components);
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
                        items.push(inner_enum.into());
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
    items.push(item_);
}

fn visit_element_decl(
    items: &mut Vec<Item>,
    element: &ElementDeclaration,
    components: &SchemaComponentTable,
) {
    let name = naming::convert::<CamelCase, PascalCase>(&element.name);

    let type_def = element.type_definition;
    match type_def {
        TypeDefinition::Simple(simple_type_def) => {
            let _simple_type_def = simple_type_def.get(components);
        }
        TypeDefinition::Complex(complex_type_def) => {
            let complex_type = complex_type_def.get(components);
            visit_complex_type(items, complex_type, components, Some(&name));
        }
    }
}

pub fn generate_rust(schema: &Schema, components: &SchemaComponentTable) -> String {
    let mut items = Vec::new();

    for element in schema.element_declarations.iter().copied() {
        let element = element.get(components);
        visit_element_decl(&mut items, element, components);
    }

    let root = syn::File {
        shebang: None,
        attrs: Vec::new(),
        items,
    };
    prettyplease::unparse(&root)
}
