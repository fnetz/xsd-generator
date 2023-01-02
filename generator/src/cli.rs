use clap::{Parser, ValueEnum};

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

    /// Allow a XML Document Type Definition (DTD) to occur
    #[clap(long)]
    pub allow_dtd: bool,

    /// The action to take when trying to overwrite a built-in type
    #[clap(long, default_value = "deny", value_enum)]
    pub builtin_overwrite: BuiltinOverwriteAction,

    /// Whether to register the builtin types and attributes
    #[clap(long, default_value = "yes", value_enum)]
    pub register_builtins: RegisterBuiltins,
}
