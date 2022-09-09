mod cli;
mod generator;
mod xsd;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    let xsd = std::fs::read_to_string(cli.input).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = xsd::read_schema(xsd, cli.builtin_overwrite, cli.register_builtins);
    let rst = generator::generate_rust(&schema, &components);
    print!("{rst}");
}
