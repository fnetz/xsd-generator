// TODO pre-processing
// TODO case sensitivity?
// TODO (low prio) re-gq comments
// TODO (low prio) restrict visibility

pub mod annotation;
pub mod assertion;
pub mod attribute_decl;
pub mod attribute_group_def;
pub mod attribute_use;
pub mod complex_type_def;
pub mod constraining_facet;
pub mod element_decl;
pub mod fundamental_facet;
pub mod identity_constraint_def;
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

mod components;
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

use std::collections::HashMap;
use std::marker::PhantomData;

pub use components::Ref;
use components::{MappingContext, RefVisitor, RefsVisitable, Resolution};
use xstypes::{QName, Sequence, Set};

use self::components::SchemaComponentContainer;

pub fn read_schema(schema: roxmltree::Document) -> (Schema, SchemaComponentContainer) {
    let mut ctx = MappingContext::new();
    let schema = Schema::map_from_xml(&mut ctx, schema.root_element());
    let components = ctx.components.perform_ref_resolution_pass();
    (schema, components)
}
