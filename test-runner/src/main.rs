use std::{
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
};

use dt_xsd::{
    import::{ImportError, ImportResolver},
    BuiltinOverwriteAction, RegisterBuiltins, Schema,
};
use encoding_rs::{Encoding, UTF_8};
use generated::{TestSet, TestSuite};
use roxmltree::Document;

mod generated;
mod parser;

fn main() {
    let base_path = Path::new("xsdtests");

    let suite = std::fs::read_to_string(base_path.join("suite.xml")).unwrap();
    let suite = Document::parse(&suite).unwrap();
    let suite = TestSuite::from_xml(suite.root_element());
    eprintln!(
        "SUITE {:?} (release date {})",
        suite.name.0, suite.release_date.0
    );

    let import_resolvers: [Box<dyn ImportResolver>; 1] = [Box::new(LocalImportResolver {
        base_path: std::env::current_dir().unwrap(),
    })];

    for test_set_ref in suite.test_set_ref {
        if test_set_ref.r#type.unwrap().0 != "locator" {
            eprintln!("Unsupported test set ref type");
            continue;
        }
        let href = test_set_ref.href.expect("href required for locator xlink");
        let href = (href.0).0.as_str();

        let path = base_path.join(href);
        let buf = std::fs::read(&path).unwrap();
        let (decoded, _, _) = Encoding::decode(UTF_8, &buf);
        let test_set = Document::parse(&decoded).unwrap();
        let test_set = TestSet::from_xml(test_set.root_element());
        eprintln!("  TEST SET {:?}", test_set.name.0);

        for group in test_set.test_group {
            eprintln!("    TEST GROUP {:?}", group.name.0);
            if let Some(schema_test) = group.schema_test {
                for schema in schema_test.schema_document {
                    let base_path = path.parent().unwrap();
                    let href = (schema.href.unwrap().0).0;
                    eprintln!("      SCHEMA {href}");
                    let schema = base_path.join(href);
                    let buf = std::fs::read(&schema).unwrap();
                    let (decoded, _, _) = Encoding::decode(UTF_8, &buf);
                    let schema = Document::parse(&decoded).unwrap();
                    let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        dt_xsd::read_schema(
                            schema,
                            BuiltinOverwriteAction::Deny,
                            RegisterBuiltins::Yes,
                            &import_resolvers,
                        )
                    }));
                    match res {
                        Ok((_schema, _components)) => {
                            eprintln!("        OK");
                        }
                        Err(e) => {
                            eprintln!("        PANIC: {:?}", e.downcast_ref::<String>());
                        }
                    }
                }
            }
        }
    }
}

struct LocalImportResolver {
    base_path: PathBuf,
}

impl ImportResolver for LocalImportResolver {
    fn resolve_import(
        &self,
        context: &mut dt_xsd::RootContext,
        import: &dt_xsd::import::Import,
    ) -> Result<Schema, ImportError> {
        let location = match import.namespace.as_deref() {
            Some("http://www.w3.org/1999/xlink") => "xlink.xsd",
            Some("http://www.w3.org/XML/1998/namespace") => "xml.xsd",
            _ => return Err(ImportError::UnsupportedImport),
        };

        let text = std::fs::read_to_string(self.base_path.join(location)).unwrap();
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let xsd = roxmltree::Document::parse_with_options(&text, options).unwrap();
        let schema = xsd.root_element();
        import.validate_imported_schema(schema)?;
        let schema = Schema::map_from_xml(context, schema);
        Ok(schema)
    }
}
