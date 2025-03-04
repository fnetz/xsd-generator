use crate::error::XsdError;
use std::{borrow::Cow, fmt};

pub type NCName = String;
pub type AnyURI = String;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QName {
    pub namespace_name: Option<Cow<'static, str>>,
    pub local_name: Cow<'static, str>,
}

impl fmt::Display for QName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace_name) = self.namespace_name.as_ref() {
            write!(f, "{{{}}}:{}", namespace_name, self.local_name)
        } else {
            write!(f, "{}", self.local_name)
        }
    }
}

impl QName {
    pub fn with_namespace(
        namespace_name: impl Into<String>,
        local_name: impl Into<String>,
    ) -> Self {
        Self::with_optional_namespace(Some(namespace_name), local_name)
    }

    pub(crate) const fn with_ns_const(
        namespace_name: &'static str,
        local_name: &'static str,
    ) -> Self {
        Self {
            namespace_name: Some(Cow::Borrowed(namespace_name)),
            local_name: Cow::Borrowed(local_name),
        }
    }

    pub fn without_namespace(local_name: impl Into<String>) -> Self {
        Self::with_optional_namespace(None::<String>, local_name)
    }

    pub fn with_optional_namespace(
        namespace_name: Option<impl Into<String>>,
        local_name: impl Into<String>,
    ) -> Self {
        Self {
            namespace_name: namespace_name.map(Into::into).map(Cow::Owned),
            local_name: Cow::Owned(local_name.into()),
        }
    }

    /// Consumes the QName and returns its parts as a tuple `(namespace_name, local_name)`.
    pub fn into_parts(self) -> (Option<String>, String) {
        (
            self.namespace_name.map(Cow::into_owned),
            self.local_name.into_owned(),
        )
    }

    pub fn qualified(
        prefix: impl AsRef<str>,
        local_name: impl Into<String>,
        context: roxmltree::Node,
    ) -> Result<Self, XsdError> {
        let prefix = prefix.as_ref();
        let resolved_prefix = if prefix == "xml" {
            // The prefix xml is by definition bound to the namespace name
            // http://www.w3.org/XML/1998/namespace.
            // (Namespaces in XML 1.0, §3, Reserved Prefixes and Namespace Names)
            "http://www.w3.org/XML/1998/namespace"
        } else {
            context
                .lookup_namespace_uri(Some(prefix))
                .ok_or_else(|| XsdError::NamePrefixNotResolved(prefix.into()))?
        };
        Ok(Self::with_namespace(resolved_prefix, local_name))
    }

    pub fn unqualified(local_name: impl Into<String>, context: roxmltree::Node) -> Self {
        // If there is a default namespace declaration in scope, the expanded name corresponding to
        // an unprefixed element name has the URI of the default namespace as its namespace name.
        // If there is no default namespace declaration in scope, the namespace name has no value.
        // (Namespaces in XML 1.0, §6.2)
        let namespace_name = context.lookup_namespace_uri(None);
        QName::with_optional_namespace(namespace_name, local_name)
    }

    pub fn parse(source: &str, context: roxmltree::Node) -> Result<Self, XsdError> {
        if let Some((prefix, local)) = source.rsplit_once(':') {
            Self::qualified(prefix, local, context)
        } else {
            Ok(Self::unqualified(source, context))
        }
    }

    pub fn namespace_name(&self) -> Option<&str> {
        self.namespace_name.as_deref()
    }

    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}

pub type Sequence<T> = Vec<T>;
pub type Set<T> = Vec<T>; //std::collections::HashSet<T>;
