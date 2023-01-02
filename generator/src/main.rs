mod cli;
mod generator;
mod naming;

use clap::Parser;

use dt_xsd as xsd;
use xsd::import::{Import, ImportError, ImportResolver};

struct HttpImportResolver;

impl ImportResolver for HttpImportResolver {
    fn resolve_import(
        &self,
        context: &mut xsd::RootContext,
        import: &Import,
    ) -> Result<xsd::Schema, ImportError> {
        let Some(schema_location) = import.schema_location.as_ref() else {
            return Err(ImportError::UnsupportedImport);
        };

        if !schema_location.starts_with("http://") && !schema_location.starts_with("https://") {
            return Err(ImportError::UnsupportedImport);
        }

        eprintln!(
            "Loading import from {} (target namespace: {:?})",
            schema_location, import.namespace
        );
        let text = reqwest::blocking::get(schema_location)
            .unwrap()
            .text()
            .unwrap();
        let xsd = roxmltree::Document::parse(&text).unwrap();
        let schema = xsd.root_element();
        let schema = xsd::Schema::map_from_xml(context, schema);
        Ok(schema)
    }
}

fn main() {
    let cli = cli::Cli::parse();

    let import_resolvers = [Box::new(HttpImportResolver) as Box<dyn ImportResolver>];
    let xsd = std::fs::read_to_string(cli.input).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
        ..Default::default()
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = xsd::read_schema(
        xsd,
        match cli.builtin_overwrite {
            cli::BuiltinOverwriteAction::Deny => xsd::BuiltinOverwriteAction::Deny,
            cli::BuiltinOverwriteAction::Warn => xsd::BuiltinOverwriteAction::Warn,
            cli::BuiltinOverwriteAction::Allow => xsd::BuiltinOverwriteAction::Allow,
        },
        match cli.register_builtins {
            cli::RegisterBuiltins::Yes => xsd::RegisterBuiltins::Yes,
            cli::RegisterBuiltins::No => xsd::RegisterBuiltins::No,
        },
        &import_resolvers,
    );
    let rst = generator::generate_rust(&schema, &components);
    print!("{rst}");
}
