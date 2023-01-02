mod cli;
mod generator;
mod xsd;

use clap::Parser;

use xsd::import::{Import, ImportError, ImportResolver};

struct TempImportResolver;

impl ImportResolver for TempImportResolver {
    fn resolve_import(
        &self,
        context: &mut xsd::RootContext,
        import: &Import,
    ) -> Result<xsd::Schema, ImportError> {
        if import.namespace.as_deref() == Some("http://www.w3.org/XML/1998/namespace") {
            let xsd = std::fs::read_to_string("schemas/xml.xsd").unwrap();
            let xsd = roxmltree::Document::parse(&xsd).unwrap();
            let child_schema = xsd.root_element();
            let child_schema = xsd::Schema::map_from_xml(context, child_schema);
            Ok(child_schema)
        } else {
            Err(ImportError::UnsupportedImport)
        }
    }
}

fn main() {
    let cli = cli::Cli::parse();

    let import_resolvers = [Box::new(TempImportResolver) as Box<dyn ImportResolver>];
    let xsd = std::fs::read_to_string(cli.input).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = xsd::read_schema(
        xsd,
        cli.builtin_overwrite,
        cli.register_builtins,
        &import_resolvers,
    );
    let rst = generator::generate_rust(&schema, &components);
    print!("{rst}");
}
