mod common;

#[cfg(feature = "generator-rust")]
mod rust;
#[cfg(feature = "generator-typescript")]
mod typescript;

#[cfg(not(any(feature = "generator-rust", feature = "generator-typescript")))]
compile_error!("At least one generator must be enabled");

use clap::ValueEnum;
use dt_xsd::{Schema, SchemaComponentTable};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Generator {
    #[cfg(feature = "generator-rust")]
    Rust,
    #[cfg(feature = "generator-typescript")]
    Typescript,
}

impl Generator {
    pub fn generate(&self, schema: &Schema, table: &SchemaComponentTable) -> String {
        match *self {
            #[cfg(feature = "generator-rust")]
            Self::Rust => rust::generate(schema, table),
            #[cfg(feature = "generator-typescript")]
            Self::Typescript => typescript::generate(schema, table),
        }
    }
}
