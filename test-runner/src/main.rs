use std::path::{Path, PathBuf};

use dt_xsd::{
    import::{ImportError, ImportResolver},
    BuiltinOverwriteAction, RegisterBuiltins, Schema,
};
use encoding_rs::{Encoding, UTF_8};
use generated::{
    Expected, ExpectedOutcome, KnownToken, KnownXsdVersion, TestOutcome, TestSet, TestSuite,
    TypeType, VersionInfo, VersionToken, XmlSubstrate,
};
use roxmltree::Document;

mod generated;
mod parser;

fn version_token_applies(version_token: &VersionToken) -> bool {
    match version_token {
        VersionToken::KnownToken(token) => match token {
            KnownToken::KnownXsdVersion(version) => match version {
                KnownXsdVersion::_10 => false,
                KnownXsdVersion::_11 => true,
            },
            KnownToken::Xsd10Editions(_) => false,
            KnownToken::XmlSubstrate(substrate) => match substrate {
                // Only test XML 1.0 for now
                XmlSubstrate::Xml11 => false,
                _ => true,
            },
            KnownToken::UnicodeVersions(_) => true,
            // TODO: Skipped for now, our processor doesn't really handle this case at all
            KnownToken::RuntimeSchemaError(_) => false,
            // TODO: Skipped for now, same
            KnownToken::XpathInCta(_) => false,
            // TODO: Skipped for now, same
            KnownToken::XdmFiltering(_) => false,
        },
        // We have no good way to know what we should do with unknown version tokens
        VersionToken::Decimal(_) | VersionToken::Nmtoken(_) => false,
    }
}

#[derive(Copy, Clone)]
enum Connector {
    And,
    Or,
}

fn version_applies(version: &VersionInfo, connector: Connector) -> bool {
    let mut tokens = version.0.iter();
    match connector {
        Connector::And => tokens.all(version_token_applies),
        Connector::Or => tokens.any(version_token_applies),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ExpectedResult {
    Valid,
    Invalid,
    /// e.g. implementation-specific or not yet supported
    Undefined,
}

fn compute_expected_outcome(expected: &[Expected]) -> ExpectedResult {
    let mut validities = expected.iter().filter(|e| {
        e.version
            .as_ref()
            .map(|version| version_applies(version, Connector::And))
            .unwrap_or(true)
    });
    let Some(validity) = &validities.next() else {
        eprintln!("No expected outcome for this version");
        return ExpectedResult::Undefined;
    };
    if validities.next().is_some() {
        eprintln!("Multiple conflicting expected outcomes");
        return ExpectedResult::Undefined;
    }

    match validity.validity {
        ExpectedOutcome::TestOutcome(outcome) => match outcome {
            TestOutcome::Valid => ExpectedResult::Valid,
            TestOutcome::Invalid => ExpectedResult::Invalid,
            TestOutcome::NotKnown => ExpectedResult::Undefined,
            TestOutcome::RuntimeSchemaError => ExpectedResult::Undefined,
        },
        ExpectedOutcome::Unnamed(_) => ExpectedResult::Undefined,
    }
}

fn read_test_set(path: &Path) -> TestSet {
    let buf = std::fs::read(path).unwrap();
    let (decoded, _, _) = Encoding::decode(UTF_8, &buf);
    let test_set = Document::parse(&decoded).unwrap();
    TestSet::from_xml(test_set.root_element())
}

fn main() {
    let base_path = Path::new("xsdtests");
    let predefined_schemas_path = std::env::current_dir().unwrap();

    let suite = std::fs::read_to_string(base_path.join("suite.xml")).unwrap();
    let suite = Document::parse(&suite).unwrap();
    let suite = TestSuite::from_xml(suite.root_element());
    eprintln!(
        "SUITE {:?} (release date {})",
        suite.name.0, suite.release_date.0
    );

    let mut count_ok = 0;
    let mut count_fail = 0;
    let mut count_skip = 0;
    let mut count_impl = 0;

    for test_set_ref in suite.test_set_ref {
        if test_set_ref.r#type.unwrap() != TypeType::Locator {
            eprintln!("Unsupported test set ref type");
            continue;
        }
        let href = test_set_ref.href.expect("href required for locator xlink");
        let href = (href.0).0.as_str();

        let path = base_path.join(href);
        let test_set = read_test_set(&path);
        eprintln!("  TEST SET {:?}", test_set.name.0);

        let apply_test_set = test_set
            .version
            .map(|version| version_applies(&version, Connector::Or))
            .unwrap_or(true);
        if !apply_test_set {
            eprintln!("  SKIPPED");
            count_skip += 1;
            continue;
        }

        for group in test_set.test_group {
            eprintln!("    TEST GROUP {:?}", group.name.0);

            let apply_test_group = group
                .version
                .map(|version| version_applies(&version, Connector::Or))
                .unwrap_or(true);
            if !apply_test_group {
                eprintln!("  SKIPPED");
                count_skip += 1;
                continue;
            }

            let mut schemata = Vec::new();

            if let Some(schema_test) = group.schema_test {
                let expected_result = compute_expected_outcome(&schema_test.expected);
                let mut ok = true;
                for schema in schema_test.schema_document {
                    let base_path = path.parent().unwrap();
                    let href = (schema.href.unwrap().0).0;
                    eprintln!("      SCHEMA {href}");
                    let schema_path = base_path.join(href);
                    let schema_dir = schema_path.parent().unwrap();
                    let buf = std::fs::read(&schema_path).unwrap();
                    let (decoded, _, _) = Encoding::decode(UTF_8, &buf);
                    let schema = Document::parse(&decoded).unwrap();
                    let res = std::panic::catch_unwind(|| {
                        let import_resolvers: [Box<dyn ImportResolver>; 1] =
                            [Box::new(LocalImportResolver {
                                base_path: predefined_schemas_path.clone(),
                                schema_dir: schema_dir.to_path_buf(),
                            })];
                        dt_xsd::read_schema(
                            schema,
                            BuiltinOverwriteAction::Deny,
                            RegisterBuiltins::Yes,
                            &import_resolvers,
                        )
                    });
                    match res {
                        Err(_) | Ok(Err(_)) => {
                            ok = false;
                        }
                        Ok(Ok(schema)) => {
                            schemata.push(schema);
                        }
                    }
                }
                eprint!("        ");
                let actual_result = if ok {
                    ExpectedResult::Valid
                } else {
                    ExpectedResult::Invalid
                };

                if expected_result == ExpectedResult::Undefined {
                    count_impl += 1;
                    eprintln!("IMPLEMENTATION-SPECIFIC - got {actual_result:?}");
                } else if actual_result == expected_result {
                    count_ok += 1;
                    eprintln!("OK (expected {expected_result:?}, got {actual_result:?})");
                } else {
                    count_fail += 1;
                    eprintln!("FAIL (expected {expected_result:?}, got {actual_result:?})");
                }
            }

            for instance_test in group.instance_test {
                eprintln!("      INSTANCE TEST {:?}", instance_test.name.0);
                if schemata.len() != 1 {
                    eprintln!("        SKIPPED");
                    count_skip += 1;
                    continue;
                }

                let (schema, components) = &schemata[0];

                let base_path = path.parent().unwrap();
                let href = (instance_test.instance_document.href.unwrap().0).0;

                if href.ends_with("mgG014.xml") {
                    // TODO: Tries to allocate a lot using maxOccurs 999999999
                    eprintln!("        SKIPPED");
                    count_skip += 1;
                    continue;
                }

                let schema_path = base_path.join(href);
                let Ok(buf) = std::fs::read(&schema_path) else {
                    eprintln!("        FILE NOT FOUND");
                    count_fail += 1;
                    continue;
                };
                let (decoded, _, _) = Encoding::decode(UTF_8, &buf);

                let xml = roxmltree::Document::parse(&decoded);
                let Ok(xml) = xml else {
                    eprintln!("        INVALID XML");
                    count_fail += 1;
                    continue;
                };

                let e = xml.root_element();
                let ged = schema.find_element_by_name(
                    e.tag_name().namespace(),
                    e.tag_name().name(),
                    components,
                );

                let res = std::panic::catch_unwind(|| {
                    dt_xsd::validation::element_locally_valid_element(
                        &e,
                        ged.map(|g| g.get(components)),
                        &components,
                    )
                });

                // TODO: treat panic separately
                let actual_result = match res {
                    Ok(true) => ExpectedResult::Valid,
                    Ok(false) => ExpectedResult::Invalid,
                    Err(_) => ExpectedResult::Undefined,
                };

                let expected_result = compute_expected_outcome(&instance_test.expected);

                if expected_result == ExpectedResult::Undefined {
                    count_impl += 1;
                    eprintln!("        IMPLEMENTATION-SPECIFIC - got {actual_result:?}");
                } else if actual_result == expected_result {
                    count_ok += 1;
                    eprintln!("        OK (expected {expected_result:?}, got {actual_result:?})");
                } else {
                    count_fail += 1;
                    eprintln!("        FAIL (expected {expected_result:?}, got {actual_result:?})");
                }
            }
        }
    }

    eprintln!(
        "OK: {:3} FAIL: {:3} SKIP: {:3} IMPL: {:3}",
        count_ok, count_fail, count_skip, count_impl
    );
}

struct LocalImportResolver {
    base_path: PathBuf,
    schema_dir: PathBuf,
}

impl ImportResolver for LocalImportResolver {
    fn resolve_import(
        &self,
        context: &mut dt_xsd::RootContext,
        import: &dt_xsd::import::Import,
    ) -> Result<Schema, ImportError> {
        let path = if let Some(location) = import
            .schema_location
            .as_ref()
            .filter(|l| !l.contains(['/', '\\', ':']))
        {
            // TODO: better path validation?
            let location = self.schema_dir.join(location);
            if !location.exists() {
                return Err(ImportError::ValidationFailed);
            }
            location
        } else {
            let location = match import.namespace.as_deref() {
                Some("http://www.w3.org/1999/xlink") => "xlink.xsd",
                Some("http://www.w3.org/XML/1998/namespace") => "xml.xsd",
                _ => return Err(ImportError::UnsupportedImport),
            };
            self.base_path.join(location)
        };

        let text = std::fs::read_to_string(path).unwrap();
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
