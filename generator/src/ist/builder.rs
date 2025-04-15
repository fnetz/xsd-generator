use std::collections::{HashMap, HashSet};

use dt_xsd::{
    AttributeDeclaration, AttributeUse, ComplexTypeDefinition, ElementDeclaration, ModelGroup,
    ModelGroupDefinition, Particle, Ref, Schema, SchemaComponentTable, SimpleTypeDefinition, Term,
    TypeDefinition, Wildcard,
    attribute_decl::ScopeParent as AttributeScopeParent,
    complex_type_def::{ContentType, Context as ComplexContext, OpenContent},
    components::{IsBuiltinRef, Named},
    element_decl::ScopeParent as ElementScopeParent,
    model_group::Compositor,
    particle::MaxOccurs,
    shared::{Scope, ScopeVariety},
    simple_type_def::{Context as SimpleContext, Variety},
};
use hstr::Atom;

use crate::ist::{FieldSource, Member, Type, TypeBinding, TypeIndex, TypeRef};

use super::{ExternalKind, Quant, Visibility};

pub struct IstBuilder {
    pub types: HashMap<TypeIndex, TypeBinding>,
    generated_index_counter: u32,
}

impl IstBuilder {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            generated_index_counter: 0,
        }
    }

    pub fn create_type_no_id(
        &mut self,
        type_: Type,
        name: Option<Atom>,
        visibility: Visibility,
        owner: Option<TypeIndex>,
    ) -> TypeIndex {
        let type_index = TypeIndex::Generated(self.generated_index_counter);
        self.generated_index_counter += 1;

        let type_binding = TypeBinding {
            xml_name: None,
            name,
            type_,
            // global,
            owner,
            documentation: None,
            inline: false,
            visibility,
        };

        self.types.insert(type_index, type_binding);
        type_index
    }

    pub fn insert_binding(&mut self, index: TypeIndex, binding: TypeBinding) {
        self.types.insert(index, binding);
    }

    fn create_quant(&mut self, type_: TypeRef, min: u64, max: MaxOccurs) -> TypeIndex {
        self.create_type_no_id(
            Type::create_quantified(type_, Quant::new(min, max)),
            None,
            Visibility::Intermediate,
            None,
        )
    }

    fn create_unbounded_quant(&mut self, item_type: TypeRef) -> TypeIndex {
        self.create_quant(item_type, 0, MaxOccurs::Unbounded)
    }
}

pub struct IstBuildVisitor<'a> {
    builder: IstBuilder,
    table: &'a SchemaComponentTable,
    target_namespace: Option<String>,
    visited: HashSet<TypeIndex>,
}

impl<'a> IstBuildVisitor<'a> {
    pub fn new(table: &'a SchemaComponentTable, target_namespace: Option<String>) -> Self {
        Self {
            builder: IstBuilder::new(),
            visited: HashSet::new(),
            table,
            target_namespace,
        }
    }

    pub fn into_ist(self) -> IstBuilder {
        self.builder
    }

    fn generate_fields_for_attribute_uses(
        &mut self,
        attribute_uses: &[Ref<AttributeUse>],
        fields: &mut Vec<Member>,
    ) {
        for attribute_use in attribute_uses.iter().copied() {
            fields.push(
                Member::builder(self.visit_attribute_use(attribute_use))
                    .source(FieldSource::AttributeUse)
                    .build(),
            );
        }
    }

    fn generate_field_for_open_content(&mut self, open_content: OpenContent) -> Member {
        let content_type = self.visit_wildcard(open_content.wildcard);
        let quantifier = self.builder.create_unbounded_quant(content_type);

        Member::builder(TypeRef::Internal(quantifier))
            .source(FieldSource::OpenContent)
            .build()
    }

    pub fn visit_schema(&mut self, schema: &Schema) {
        for model_group_def in schema.model_group_definitions.iter().copied() {
            self.visit_model_group_definition(model_group_def);
        }

        for element in schema.element_declarations.iter().copied() {
            self.visit_element_declaration(element);
        }

        for type_definition in schema.type_definitions.iter().copied() {
            match type_definition {
                TypeDefinition::Complex(complex_type) => {
                    self.visit_complex_type(complex_type);
                }
                TypeDefinition::Simple(simple_type) => {
                    self.visit_simple_type(simple_type);
                }
            }
        }
    }

    fn visit_complex_type(&mut self, complex_type: Ref<ComplexTypeDefinition>) -> TypeRef {
        if complex_type.is_builtin(self.table) {
            return TypeRef::Builtin(TypeDefinition::Complex(complex_type));
        }

        let type_index = TypeIndex::ComplexType(complex_type);

        let complex_type = complex_type.get(self.table);

        if complex_type.target_namespace != self.target_namespace {
            let name = complex_type
                .name()
                .expect("Unexpected missing name for type with different target namespace");
            return TypeRef::External(name, ExternalKind::TypeDefinition);
        }

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let mut fields = Vec::new();

        self.generate_fields_for_attribute_uses(&complex_type.attribute_uses, &mut fields);

        // TODO: complex_type.attribute_wildcard

        match complex_type.content_type {
            ContentType::Empty => {
                // Nothing to do
            }
            ContentType::Simple {
                simple_type_definition,
            } => {
                fields.push(
                    Member::builder(self.visit_simple_type(simple_type_definition))
                        .source(FieldSource::SimpleContent)
                        .build(),
                );
            }
            ContentType::ElementOnly {
                particle,
                open_content,
            } => {
                fields.push(
                    Member::builder(self.visit_particle(particle, type_index))
                        .source(FieldSource::ComplexContent)
                        .build(),
                );

                if let Some(open_content) = open_content {
                    fields.push(self.generate_field_for_open_content(open_content));
                }
            }
            ContentType::Mixed {
                particle,
                open_content,
            } => {
                fields.push(
                    Member::builder(self.visit_particle(particle, type_index))
                        .source(FieldSource::ComplexContent)
                        .build(),
                );

                if let Some(open_content) = open_content {
                    fields.push(self.generate_field_for_open_content(open_content));
                }

                // TODO: string content?
            }
        }

        let type_ = Type::create_structure(fields);

        let type_binding = TypeBinding {
            xml_name: complex_type.name(),
            name: complex_type.name().map(|n| Atom::new(n.local_name())),
            type_,
            // global: complex_type.name.is_some(),
            owner: complex_type.context.map(|c| match c {
                ComplexContext::Element(e) => TypeIndex::ElementDecl(e),
                ComplexContext::ComplexType(c) => TypeIndex::ComplexType(c),
            }),
            documentation: None,
            inline: complex_type.name.is_none(),
            visibility: if complex_type.name.is_some() {
                Visibility::Public
            } else {
                Visibility::Intermediate
            },
        };

        self.builder.insert_binding(type_index, type_binding);
        TypeRef::Internal(type_index)
    }

    fn visit_simple_type(&mut self, simple_type: Ref<SimpleTypeDefinition>) -> TypeRef {
        if simple_type.is_builtin(self.table) {
            return TypeRef::Builtin(TypeDefinition::Simple(simple_type));
        }

        let type_index = TypeIndex::SimpleType(simple_type);

        let simple_type = simple_type.get(self.table);

        if simple_type.target_namespace != self.target_namespace {
            let name = simple_type
                .name()
                .expect("Unexpected missing name for type with different target namespace");
            return TypeRef::External(name, ExternalKind::TypeDefinition);
        }

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        // Only xs::anyAtomicType is allowed to have a variety of None
        let variety = simple_type
            .variety
            .expect("Non-builtin simple types must have a variety");

        let type_ = match variety {
            Variety::Atomic => {
                let primitive_type = simple_type
                    .primitive_type_definition
                    .expect("Atomic simple types must have a primitive type");

                Type::create_newtype(self.visit_simple_type(primitive_type))
            }
            Variety::List => {
                let item_type = simple_type
                    .item_type_definition
                    .expect("List simple types must have an item type");

                let item_type = self.visit_simple_type(item_type);

                Type::create_quantified(item_type, Quant::zero_or_more())
            }
            Variety::Union => {
                let member_types = simple_type
                    .member_type_definitions
                    .as_deref()
                    .expect("Union simple types must have member types");

                Type::create_union(
                    member_types
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(i, member_type)| Member {
                            name: None,
                            type_: self.visit_simple_type(member_type),
                            source: FieldSource::SimpleTypeMember(i),
                            documentation: None,
                            quant: Quant::default(),
                        })
                        .collect(),
                )
            }
        };

        let type_binding = TypeBinding {
            xml_name: simple_type.name(),
            name: simple_type.name().map(|n| Atom::new(n.local_name())),
            type_,
            // global: simple_type.name.is_some(),
            owner: simple_type.context.map(|c| match c {
                SimpleContext::Element(e) => TypeIndex::ElementDecl(e),
                SimpleContext::ComplexType(c) => TypeIndex::ComplexType(c),
                SimpleContext::Attribute(a) => TypeIndex::AttributeDecl(a),
                SimpleContext::SimpleType(s) => TypeIndex::SimpleType(s),
            }),
            documentation: None,
            inline: simple_type.name.is_none(),
            visibility: if simple_type.name.is_some() {
                Visibility::Public
            } else {
                Visibility::Intermediate
            },
        };
        self.builder.insert_binding(type_index, type_binding);

        TypeRef::Internal(type_index)
    }

    fn visit_particle(&mut self, particle: Ref<Particle>, owner: TypeIndex) -> TypeRef {
        let type_index = TypeIndex::Particle(particle);
        let type_index_term = TypeIndex::ParticleTerm(particle);
        // let quant_source = QuantifiedSource::Particle(particle);

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let particle = particle.get(self.table);

        let type_ = match particle.term {
            Term::ElementDeclaration(element) => {
                let element = self.visit_element_declaration(element);
                Type::create_newtype(element)
            }
            Term::ModelGroup(model_group) => {
                let model_group = self.visit_model_group(model_group);
                Type::create_newtype(model_group)
            }
            Term::Wildcard(wildcard) => {
                let wildcard = self.visit_wildcard(wildcard);
                Type::create_newtype(wildcard)
            }
        };

        let type_binding = TypeBinding {
            xml_name: None,
            name: None,
            type_,
            // global: false,
            owner: Some(type_index),
            documentation: None,
            inline: true,
            visibility: Visibility::Intermediate,
        };

        self.builder.insert_binding(type_index_term, type_binding);

        let quant = Quant::new(particle.min_occurs, particle.max_occurs);
        let quantifier = Type::create_quantified(TypeRef::Internal(type_index_term), quant);

        let type_binding = TypeBinding {
            xml_name: None,
            name: None,
            type_: quantifier,
            // global: false,
            owner: Some(owner),
            documentation: None,
            inline: true,
            visibility: Visibility::Intermediate,
        };
        self.builder.insert_binding(type_index, type_binding);

        TypeRef::Internal(type_index)
    }

    fn create_type_for_model_group(
        &mut self,
        model_group: &ModelGroup,
        type_index: TypeIndex,
    ) -> Type {
        let content_types_iter = model_group
            .particles
            .iter()
            .copied()
            .map(|particle| self.visit_particle(particle, type_index))
            .enumerate();

        match model_group.compositor {
            Compositor::All | Compositor::Sequence => {
                let fields = content_types_iter
                    .map(|(i, type_)| {
                        Member::builder(type_)
                            .source(FieldSource::ModelGroupParticle(i))
                            .build()
                    })
                    .collect();

                Type::create_structure(fields)
            }
            Compositor::Choice => {
                let variants = content_types_iter
                    .map(|(i, type_)| Member {
                        name: None,
                        type_,
                        source: FieldSource::ModelGroupParticle(i),
                        documentation: None,
                        quant: Quant::default(),
                    })
                    .collect();

                Type::create_union(variants)
            }
        }
    }

    fn visit_model_group_definition(
        &mut self,
        model_group_def: Ref<ModelGroupDefinition>,
    ) -> TypeRef {
        let type_index = TypeIndex::ModelGroupDef(model_group_def);

        let model_group_def = model_group_def.get(self.table);

        if model_group_def.target_namespace != self.target_namespace {
            let name = model_group_def
                .name()
                .expect("Unexpected missing name for type with different target namespace");
            return TypeRef::External(name, ExternalKind::TypeDefinition);
        }

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        // let model_group = self.visit_model_group(model_group_def.model_group);

        let type_ = self
            .create_type_for_model_group(model_group_def.model_group.get(self.table), type_index);

        let type_binding = TypeBinding {
            xml_name: model_group_def.name(),
            name: model_group_def.name().map(|n| Atom::new(n.local_name())),
            type_,
            owner: None, // TODO
            documentation: None,
            // We try to inline model group definitions even though this might drop the name, since
            // this reduces unneccessary depth in the generated types.
            inline: true,
            // Model group definitions are always global. They are split into unnamed model groups
            // and top-level-only named model group definitions, unlike elements or attributes for
            // some reason.
            visibility: Visibility::Public,
        };

        self.builder.insert_binding(type_index, type_binding);
        TypeRef::Internal(type_index)
    }

    fn visit_model_group(&mut self, model_group_ref: Ref<ModelGroup>) -> TypeRef {
        let type_index = TypeIndex::ModelGroup(model_group_ref);

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let model_group = model_group_ref.get(self.table);

        // We generate model groups "the other way round": If the model group is a child of a
        // (global) model group definition, we let the child type be a newtype for the parent type.
        // This way, we preserve the name of the model group definition, and get imports instead of
        // copied internal types.
        let type_ = if let Some(model_group_def) = model_group.parent {
            let type_ = self.visit_model_group_definition(model_group_def);
            Type::create_newtype(type_)
        } else {
            self.create_type_for_model_group(model_group, type_index)
        };

        let type_binding = TypeBinding {
            xml_name: None,
            name: None,
            type_,
            owner: None, // TODO
            documentation: None,
            inline: true,
            visibility: Visibility::Intermediate,
        };

        self.builder.insert_binding(type_index, type_binding);
        TypeRef::Internal(type_index)
    }

    fn visit_element_declaration(&mut self, element: Ref<ElementDeclaration>) -> TypeRef {
        let type_index = TypeIndex::ElementDecl(element);
        // let quant_source = QuantifiedSource::ElementDecl(element);

        let element = element.get(self.table);

        // Local element declarations can be unqualified, depending on the form.
        if element.target_namespace != self.target_namespace
            && element.scope.variety() == ScopeVariety::Global
        {
            let name = element
                .name()
                .expect("Unexpected missing name for element with different target namespace");
            return TypeRef::External(name, ExternalKind::ElementDeclaration);
        }

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let content_type = match element.type_definition {
            TypeDefinition::Complex(complex_type) => self.visit_complex_type(complex_type),
            TypeDefinition::Simple(simple_type) => self.visit_simple_type(simple_type),
        };

        let type_ = if element.nillable {
            Type::create_quantified(content_type, Quant::zero_or_one())
        } else {
            Type::create_newtype(content_type)
        };

        let type_binding = TypeBinding {
            xml_name: element.name(),
            name: element.name().map(|n| Atom::new(n.local_name())),
            type_,
            // global: element.scope.variety() == ScopeVariety::Global,
            owner: match element.scope {
                Scope::Global => None,
                Scope::Local(p) => match p {
                    ElementScopeParent::ComplexType(c) => Some(TypeIndex::ComplexType(c)),
                    ElementScopeParent::Group(_g) => None, // TODO
                },
            },
            documentation: None,
            inline: element.scope.variety() == ScopeVariety::Local,
            visibility: match element.scope.variety() {
                ScopeVariety::Global => Visibility::Public,
                ScopeVariety::Local => Visibility::Intermediate,
            },
        };

        self.builder.insert_binding(type_index, type_binding);
        TypeRef::Internal(type_index)
    }

    fn visit_wildcard(&mut self, wildcard: Ref<Wildcard>) -> TypeRef {
        let type_index = TypeIndex::Wildcard(wildcard);

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let _wildcard = wildcard.get(self.table);

        // TODO:
        let type_ = Type::create_structure(Vec::new());

        let type_binding = TypeBinding {
            xml_name: None,
            name: None,
            type_,
            // global: false,
            owner: None, // TODO
            documentation: None,
            inline: false,
            visibility: Visibility::Intermediate,
        };

        self.builder.insert_binding(type_index, type_binding);
        TypeRef::Internal(type_index)
    }

    fn visit_attribute_use(&mut self, attribute_use: Ref<AttributeUse>) -> TypeRef {
        let type_index = TypeIndex::AttributeUse(attribute_use);

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let attribute_use = attribute_use.get(self.table);

        let type_ = self.visit_attribute_declaration(attribute_use.attribute_declaration);

        let type_ = if attribute_use.required {
            Type::create_newtype(type_)
        } else {
            Type::create_quantified(type_, Quant::zero_or_one())
        };

        let type_binding = TypeBinding {
            xml_name: None,
            name: None,
            type_,
            // global: false,
            owner: None, // TODO
            documentation: None,
            inline: true,
            visibility: Visibility::Intermediate,
        };

        self.builder.insert_binding(type_index, type_binding);
        TypeRef::Internal(type_index)
    }

    fn visit_attribute_declaration(
        &mut self,
        attribute_decl: Ref<AttributeDeclaration>,
    ) -> TypeRef {
        let type_index = TypeIndex::AttributeDecl(attribute_decl);

        let attribute_decl = attribute_decl.get(self.table);

        // Local attribute declarations can be unqualified, depending on the form.
        if attribute_decl.target_namespace != self.target_namespace
            && attribute_decl.scope.variety() == ScopeVariety::Global
        {
            let name = attribute_decl
                .name()
                .expect("Unexpected missing name for attribute with different target namespace");
            return TypeRef::External(name, ExternalKind::AttributeDeclaration);
        }

        if !self.visited.insert(type_index) {
            return TypeRef::Internal(type_index);
        }

        let type_ = Type::create_newtype(self.visit_simple_type(attribute_decl.type_definition));

        let type_binding = TypeBinding {
            xml_name: attribute_decl.name(),
            name: attribute_decl.name().map(|n| Atom::new(n.local_name())),
            type_,
            // global: attribute_decl.scope.variety() == ScopeVariety::Global,
            owner: match attribute_decl.scope {
                Scope::Global => None,
                Scope::Local(p) => match p {
                    AttributeScopeParent::ComplexType(c) => Some(TypeIndex::ComplexType(c)),
                    AttributeScopeParent::AttributeGroup(_g) => None, // TODO
                },
            },
            documentation: None,
            inline: attribute_decl.scope.variety() == ScopeVariety::Local,
            visibility: match attribute_decl.scope.variety() {
                ScopeVariety::Global => Visibility::Public,
                ScopeVariety::Local => Visibility::Intermediate,
            },
        };

        self.builder.insert_binding(type_index, type_binding);

        TypeRef::Internal(type_index)
    }
}
