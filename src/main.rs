mod xsd;

use clap::Parser;

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    #[clap(value_parser, help = "The source file or URL")]
    input: String,

    #[clap(long, help = "Allow a XML Document Type Definition (DTD) to occur")]
    allow_dtd: bool,
}

fn main() {
    let cli = Cli::parse();

    let xsd = std::fs::read_to_string(cli.input).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = xsd::read_schema(xsd);
}
