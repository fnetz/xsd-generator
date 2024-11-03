mod cli;
mod generators;
mod ist;

use clap::Parser;

use dt_xsd::import::{Import, ImportError, ImportResolver};
use dt_xsd::{RootContext, Schema};

struct HttpImportResolver;

impl ImportResolver for HttpImportResolver {
    fn resolve_import(
        &self,
        context: &mut RootContext,
        import: &Import,
    ) -> Result<Schema, ImportError> {
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
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let xsd = roxmltree::Document::parse_with_options(&text, options).unwrap();
        let schema = xsd.root_element();
        import.validate_imported_schema(schema)?;
        let schema = Schema::map_from_xml(context, schema).map_err(ImportError::Xsd)?;
        Ok(schema)
    }
}

fn main() {
    let cli = cli::Cli::parse();

    let mut import_resolvers = Vec::<Box<dyn ImportResolver>>::new();
    if cli.allow_http_imports {
        import_resolvers.push(Box::new(HttpImportResolver));
    }
    let xsd = std::fs::read_to_string(cli.input).unwrap();
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
        &import_resolvers,
    )
    .unwrap();
    let rst = cli.generator.generate(&schema, &components);
    print!("{rst}");
}
