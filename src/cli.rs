use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum BuiltinOverwriteAction {
    Deny,
    Warn,
    Allow,
}

#[derive(Parser)]
#[clap(version, about)]
pub struct Cli {
    #[clap(value_parser, help = "The source file or URL")]
    pub input: String,

    #[clap(long, help = "Allow a XML Document Type Definition (DTD) to occur")]
    pub allow_dtd: bool,

    #[clap(long, default_value = "deny", value_parser, arg_enum)]
    pub builtin_overwrite: BuiltinOverwriteAction,
}
