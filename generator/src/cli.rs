use clap::{Parser, ValueEnum};

use crate::generators::Generator;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum BuiltinOverwriteAction {
    Deny,
    Warn,
    Allow,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum RegisterBuiltins {
    Yes,
    No,
}

#[derive(Parser)]
#[clap(version, about)]
pub struct Cli {
    /// The source file or URL
    #[clap(value_parser)]
    pub input: String,

    /// Selects for which language the generator will output code
    #[clap(short, long, value_enum)]
    pub generator: Generator,

    /// Allow a XML Document Type Definition (DTD) to occur
    #[clap(long)]
    pub allow_dtd: bool,

    /// Allow automatic downloading of imports over HTTP
    #[clap(long)]
    pub allow_http_imports: bool,

    /// The action to take when trying to overwrite a built-in type
    #[clap(long, default_value = "deny", value_enum)]
    pub builtin_overwrite: BuiltinOverwriteAction,

    /// Whether to register the builtin types and attributes
    #[clap(long, default_value = "yes", value_enum)]
    pub register_builtins: RegisterBuiltins,
}
