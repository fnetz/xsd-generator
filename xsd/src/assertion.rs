use super::{
    annotation::Annotation,
    components::Component,
    values::actual_value,
    xstypes::{AnyURI, NCName, Sequence, Set},
    MappingContext, Ref,
};
use roxmltree::Node;

/// Schema Component: Assertion, a kind of Annotated Component (§3.13)
#[derive(Clone, Debug)]
pub struct Assertion {
    pub annotations: Sequence<Ref<Annotation>>,
    pub test: XPathExpression,
}

/// Property Record: XPath Expression (§3.13)
#[derive(Clone, Debug)]
pub struct XPathExpression {
    pub namespace_bindings: Set<NamespaceBinding>,
    pub default_namespace: Option<AnyURI>,
    pub base_uri: Option<AnyURI>,
    pub expression: String,
}

/// Property Record: Namespace Binding (§3.13)
#[derive(Clone, Debug)]
pub struct NamespaceBinding {
    pub prefix: NCName,
    pub namespace: AnyURI,
}

impl Assertion {
    pub(super) fn map_from_xml(
        context: &mut MappingContext,
        assert: Node,
        schema: Node,
    ) -> Ref<Self> {
        assert_eq!(assert.tag_name().name(), "assert");

        // {test}
        //   An XPath Expression property record, as described below, with <assert> as the "host
        //   element" and test as the designated expression [attribute].
        let test = assert.attribute("test").unwrap();
        let test = XPathExpression::map_from_xml(test, assert, schema);

        // {annotations}
        //   The ·annotation mapping· of the <assert> element, as defined in XML Representation of
        //   Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, assert);

        context.create(Self { annotations, test })
    }
}

impl XPathExpression {
    pub(super) fn map_from_xml(
        designated_attribute: &str,
        host_element: Node,
        schema: Node,
    ) -> Self {
        // {namespace bindings}
        //    A set of Namespace Binding property records. Each member corresponds to an entry in
        //    the [in-scope namespaces] of the host element, with {prefix} being the [prefix] and
        //    {namespace} the [namespace name].
        let namespace_bindings = host_element
            .namespaces() // `namespaces()` is equivalent to the [in-scope namespaces]
            .map(|namespace| {
                NamespaceBinding {
                    // TODO does None map to the empty namespace?
                    prefix: namespace.name().unwrap_or_default().to_string(),
                    namespace: namespace.uri().into(),
                }
            })
            .collect::<Set<_>>();

        // {default namespace}
        //   Let D be the ·actual value· of the xpathDefaultNamespace [attribute], if present on
        //   the host element, otherwise that of the xpathDefaultNamespace [attribute] of the
        //   <schema> ancestor.
        let d = host_element
            .attribute("xpathDefaultNamespace")
            .or_else(|| schema.attribute("xpathDefaultNamespace"))
            .map(|v| actual_value::<&str>(v, host_element))
            .unwrap_or("##local");

        //   Then the value is the appropriate case among the following:
        let default_namespace = match d {
            // 1 If D is ##defaultNamespace, then the appropriate case among the following:
            "##defaultNamespace" => {
                // 1.1 If there is an entry in the [in-scope namespaces] of the host element whose
                //     [prefix] is ·absent·, then the corresponding [namespace name];
                // 1.2 otherwise ·absent·;
                namespace_bindings
                    .iter()
                    .find(|binding| binding.prefix.is_empty())
                    .map(|binding| binding.namespace.clone())
            }
            // 2 If D is ##targetNamespace, then the appropriate case among the following:
            "##targetNamespace" => {
                // 2.1 If the targetNamespace [attribute] is present on the <schema> ancestor, then
                //     its ·actual value·;
                // 2.2 otherwise ·absent·;
                schema
                    .attribute("targetNamespace")
                    .map(|v| actual_value::<String>(v, host_element))
            }
            // 3 If D is ##local, then ·absent·;
            "##local" => None,
            // 4 otherwise (D is an xs:anyURI value) D.
            d => Some(d.to_string()),
        };

        // {base URI}
        //   The [base URI] of the host element.
        // TODO compute according to https://www.w3.org/TR/xmlbase/
        let base_uri = None;

        let expression = actual_value::<String>(designated_attribute, host_element);

        Self {
            namespace_bindings,
            default_namespace,
            base_uri,
            expression,
        }
    }
}

impl Component for Assertion {
    const DISPLAY_NAME: &'static str = "Assertion";
}
