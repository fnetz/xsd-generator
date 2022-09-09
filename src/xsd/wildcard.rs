use roxmltree::Node;

use super::{
    annotation::Annotation,
    components::Component,
    mapping_context::MappingContext,
    values::{actual_value, ActualValue},
    xstypes::{AnyURI, QName, Sequence, Set},
    Ref,
};

/// Schema Component: Wildcard, a kind of [Term](super::shared::Term) (§3.10)
#[derive(Clone, Debug)]
pub struct Wildcard {
    pub annotations: Sequence<Ref<Annotation>>,
    pub namespace_constraint: NamespaceConstraint,
    pub process_contents: ProcessContents,
}

#[derive(Clone, Debug)]
pub enum ProcessContents {
    Skip,
    Strict,
    Lax,
}

/// Property Record: Namespace Constraint (§3.10)
#[derive(Clone, Debug)]
pub struct NamespaceConstraint {
    pub variety: NamespaceConstraintVariety,
    pub namespaces: Set<Option<AnyURI>>,
    pub disallowed_names: Set<DisallowedName>,
}

#[derive(Clone, Debug)]
pub enum NamespaceConstraintVariety {
    Any,
    Enumeration,
    Not,
}

#[derive(Clone, Debug)]
pub enum DisallowedName {
    QName(QName),
    Defined,
    Sibling,
}

impl Wildcard {
    pub(super) fn map_from_xml_any(
        context: &mut MappingContext,
        any: Node,
        schema: Node,
    ) -> Ref<Self> {
        // A Namespace Constraint with the following properties:
        let namespace_constraint = {
            // {variety} the appropriate case among the following:
            let variety = if let Some(namespace) = any.attribute("namespace") {
                // 1 If the namespace [attribute] is present, then the appropriate case among the
                //   following:
                match namespace {
                    // 1.1 If namespace = "##any", then any;
                    "##any" => NamespaceConstraintVariety::Any,
                    // 1.2 If namespace = "##other", then not;
                    "##other" => NamespaceConstraintVariety::Not,
                    // 1.3 otherwise enumeration;
                    _ => NamespaceConstraintVariety::Enumeration,
                }
            } else if any.has_attribute("notNamespace") {
                // 2 If the notNamespace [attribute] is present, then not;
                NamespaceConstraintVariety::Not
            } else {
                // 3 otherwise (neither namespace nor notNamespace is present) any.
                NamespaceConstraintVariety::Any
            };

            // {namespaces} the appropriate case among the following:
            let namespaces = if !any.has_attribute("namespace")
                && !any.has_attribute("notNamespace")
                || any.attribute("namespace") == Some("##any")
            {
                // 1 If neither namespace nor notNamespace is present, then the empty set;
                // 2 If namespace = "##any", then the empty set;
                Set::new()
            } else if any.attribute("namespace") == Some("##other") {
                // 3 If namespace = "##other", then a set consisting of ·absent· and, if the
                //   targetNamespace [attribute] of the <schema> ancestor element information
                //   item is present, its ·actual value·;
                let mut namespaces = vec![None];
                if let Some(target_namespace) = schema.attribute("targetNamespace") {
                    namespaces.push(Some(actual_value::<AnyURI>(target_namespace, schema)));
                }
                namespaces
            } else {
                // 4 otherwise a set whose members are namespace names corresponding to the
                // space-delimited substrings of the ·actual value· of the namespace or
                // notNamespace [attribute] (whichever is present), except
                // 4.1 if one such substring is ##targetNamespace, the corresponding member is
                //     the ·actual value· of the targetNamespace [attribute] of the <schema>
                //     ancestor element information item if present, otherwise ·absent·;
                // 4.2 if one such substring is ##local, the corresponding member is ·absent·.
                let namespaces = any
                    .attribute("namespace")
                    .or_else(|| any.attribute("notNamespace"))
                    .unwrap(); // One of these must be present here (ensured by 1)
                let namespaces = actual_value::<Vec<String>>(namespaces, any);
                namespaces
                    .into_iter()
                    .map(|ns| match ns.as_str() {
                        "##targetNamespace" => schema
                            .attribute("namespace")
                            .map(|v| actual_value::<AnyURI>(v, schema)),
                        "##local" => None,
                        _ => Some(ns),
                    })
                    .collect()
            };

            let disallowed_names = if let Some(not_qname) = any.attribute("notQName") {
                // If the notQName [attribute] is present, then a set whose members correspond to
                // the items in the ·actual value· of the notQName [attribute], as follows.
                let not_qname = actual_value::<Vec<String>>(not_qname, any);
                not_qname
                    .into_iter()
                    .map(|n| match n.as_str() {
                        // If the item is the token "##defined", then the keyword defined is a
                        // member of the set.
                        "##defined" => DisallowedName::Defined,
                        // If the item is the token "##definedSibling", then the keyword sibling is
                        // a member of the set.
                        "##definedSibling" => DisallowedName::Sibling,
                        // If the item is a QName value (i.e. an expanded name), then that QName
                        // value is a member of the set.
                        _ => DisallowedName::QName(QName::parse(&n, any).unwrap()),
                    })
                    .collect()
            } else {
                // If the notQName [attribute] is not present, then the empty set.
                Set::new()
            };

            NamespaceConstraint {
                variety,
                namespaces,
                disallowed_names,
            }
        };

        // The ·actual value· of the processContents [attribute], if present, otherwise strict.
        let process_contents = any
            .attribute("processContents")
            .map(|v| actual_value::<ProcessContents>(v, any))
            .unwrap_or(ProcessContents::Strict);

        // The ·annotation mapping· of the <any> element, as defined in XML Representation of
        // Annotation Schema Components (§3.15.2).
        let annotations = Annotation::xml_element_annotation_mapping(context, any);

        context.create(Wildcard {
            namespace_constraint,
            process_contents,
            annotations,
        })
    }
}

impl ActualValue<'_> for ProcessContents {
    fn convert(src: &'_ str, _parent: Node) -> Self {
        match src {
            "lax" => ProcessContents::Lax,
            "skip" => ProcessContents::Skip,
            "strict" => ProcessContents::Strict,
            _ => panic!("Invalid value for processContents"),
        }
    }
}

impl Component for Wildcard {
    const DISPLAY_NAME: &'static str = "Wildcard";
}
