// TODO pre-processing
// TODO case sensitivity?
// TODO (low prio) restrict visibility

pub mod annotation;
pub mod assertion;
pub mod attribute_decl;
pub mod attribute_group_def;
pub mod attribute_use;
pub mod complex_type_def;
pub mod constraining_facet;
pub mod element_decl;
pub mod error;
pub mod fundamental_facet;
pub mod identity_constraint_def;
pub mod import;
pub mod model_group;
pub mod model_group_def;
pub mod notation_decl;
pub mod particle;
pub mod schema;
pub mod shared;
pub mod simple_type_def;
pub mod type_alternative;
pub mod wildcard;
pub mod xstypes;

mod builtins;
mod components;
mod mapping_context;
mod values;

pub use annotation::Annotation;
pub use assertion::Assertion;
pub use attribute_decl::AttributeDeclaration;
pub use attribute_group_def::AttributeGroupDefinition;
pub use attribute_use::AttributeUse;
pub use complex_type_def::ComplexTypeDefinition;
pub use constraining_facet::ConstrainingFacet;
pub use element_decl::ElementDeclaration;
pub use fundamental_facet::FundamentalFacet;
pub use identity_constraint_def::IdentityConstraintDefinition;
pub use model_group::ModelGroup;
pub use model_group_def::ModelGroupDefinition;
pub use notation_decl::NotationDeclaration;
pub use particle::Particle;
pub use schema::Schema;
pub use shared::{Term, TypeDefinition};
pub use simple_type_def::SimpleTypeDefinition;
pub use type_alternative::TypeAlternative;
pub use wildcard::Wildcard;

pub use components::{Ref, RefNamed};
use import::ImportResolver;
use mapping_context::MappingContext;
pub use mapping_context::RootContext;
use xstypes::{Sequence, Set};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuiltinOverwriteAction {
    Deny,
    Warn,
    Allow,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegisterBuiltins {
    Yes,
    No,
}

pub use components::SchemaComponentTable;

pub fn read_schema(
    schema: roxmltree::Document,
    builtin_overwrite: BuiltinOverwriteAction,
    register_builtins: RegisterBuiltins,
    import_resolvers: &[Box<dyn ImportResolver>],
) -> (Schema, SchemaComponentTable) {
    let schema = schema.root_element();
    let mut root_context = RootContext::new(builtin_overwrite, import_resolvers);
    if register_builtins == RegisterBuiltins::Yes {
        builtins::register_builtins(&mut root_context);
    }
    let schema = Schema::map_from_xml(&mut root_context, schema);
    let components = root_context
        .into_components()
        .convert_to_schema_table()
        .unwrap();
    (schema, components)
}
