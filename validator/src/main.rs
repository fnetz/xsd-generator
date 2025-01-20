mod cli;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    let xsd = std::fs::read_to_string(cli.schema).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
        ..Default::default()
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = dt_xsd::read_schema(
        xsd,
        match cli.builtin_overwrite {
            cli::BuiltinOverwriteAction::Deny => dt_xsd::BuiltinOverwriteAction::Deny,
            cli::BuiltinOverwriteAction::Warn => dt_xsd::BuiltinOverwriteAction::Warn,
            cli::BuiltinOverwriteAction::Allow => dt_xsd::BuiltinOverwriteAction::Allow,
        },
        match cli.register_builtins {
            cli::RegisterBuiltins::Yes => dt_xsd::RegisterBuiltins::Yes,
            cli::RegisterBuiltins::No => dt_xsd::RegisterBuiltins::No,
        },
        &[],
    )
    .unwrap();

    let xml_input = std::fs::read_to_string(&cli.input).unwrap();
    let xml = roxmltree::Document::parse(&xml_input).unwrap();

    let e = xml.root_element();
    let ged =
        schema.find_element_by_name(e.tag_name().namespace(), e.tag_name().name(), &components);
    let res = dt_xsd::validation::element_locally_valid_element(
        &e,
        ged.map(|g| g.get(&components)),
        &components,
    );
    println!("Result: {:?}", res);
}
