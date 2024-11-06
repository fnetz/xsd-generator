use crate::{error::XsdError, mapping_context::RootContext, values::actual_value, Schema};
use roxmltree::Node;
use thiserror::Error;

/// This structure represents the `import` element; it is not a schema component.
///
/// Note that an import is allowed to have neither a `schemaLocation` nor a `namespace` attribute.
#[derive(Clone, Debug)]
pub struct Import {
    pub namespace: Option<String>,
    pub schema_location: Option<String>,
}

impl Import {
    pub const TAG_NAME: &'static str = "import";

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

    pub fn validate_imported_schema(&self, schema: Node) -> Result<(), ImportError> {
        // § 4.2.6 Schema Representation Constraint: Import Constraints and Semantics
        let valid = if let Some(namespace) = self.namespace.as_ref() {
            // 3.1 If there is a namespace [attribute], then its ·actual value· is identical to the
            //   ·actual value· of the targetNamespace [attribute] of D2.
            let target_namespace = schema
                .attribute("targetNamespace")
                .map(|tn| actual_value::<&str>(tn, schema));
            target_namespace == Some(namespace)
        } else {
            // 3.2 If there is no namespace [attribute], then D2 has no targetNamespace [attribute]
            schema.attribute("targetNamespace").is_none()
        };
        valid.then_some(()).ok_or(ImportError::ValidationFailed)
    }
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("the resolver does not support the import")]
    UnsupportedImport,
    #[error("the schema failed to parse")]
    Xsd(XsdError),
    #[error("the imported schema is not valid with regard to the import")]
    ValidationFailed,
    #[error("an unspecified error occurred while loading the schema")]
    UnspecifiedLoad(Box<dyn std::error::Error>),
}

pub trait ImportResolver {
    fn resolve_import(
        &self,
        context: &mut RootContext,
        import: &Import,
    ) -> Result<Schema, ImportError>;
}
