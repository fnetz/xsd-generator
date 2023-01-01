use roxmltree::Node;

use super::{error::XsdError, values::actual_value};

/// This structure represents the `import` element; it is not a schema component.
///
/// Note that an import is allowed to have neither a `schemaLocation` nor a `namespace` attribute.
#[derive(Clone, Debug)]
pub struct Import {
    pub namespace: Option<String>,
    pub schema_location: Option<String>,
}

impl Import {
    pub const TAG_NAME: &str = "import";

    pub fn map_from_xml(import: Node, schema: Node) -> Result<Self, XsdError> {
        let namespace = import
            .attribute("namespace")
            .map(|ns| actual_value(ns, import));
        let schema_location = import
            .attribute("schemaLocation")
            .map(|sl| actual_value(sl, import));

        // § 4.2.6 Schema Representation Constraint: Import Constraints and Semantics
        // 1 The appropriate case among the following must be true:
        if let Some(namespace) = namespace.as_ref() {
            // 1.1 If the namespace [attribute] is present, then its ·actual value· does not match
            //   the ·actual value· of the enclosing <schema>'s targetNamespace [attribute].
            if let Some(target_namespace) = schema.attribute("targetNamespace") {
                let target_namespace: &str = actual_value(target_namespace, schema);
                if namespace == target_namespace {
                    panic!("TODO: error: import namespace matches target namespace");
                }
            }
        } else {
            // 1.2 If the namespace [attribute] is not present, then the enclosing <schema> has a
            //   targetNamespace [attribute]
            if schema.attribute("targetNamespace").is_none() {
                panic!("TODO: error: no import namespace and schema target namespace is missing");
            }
        }

        Ok(Import {
            namespace,
            schema_location,
        })
    }
}
