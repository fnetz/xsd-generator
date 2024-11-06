use crate::{
    annotation::Annotation,
    assertion::XPathExpression,
    components::{AnnotatedComponent, Component, Named, NamedXml},
    error::XsdError,
    mapping_context::{MappingContext, TopLevelMappable},
    values::actual_value,
    xstypes::{AnyURI, NCName, QName, Sequence},
    Ref,
};
use roxmltree::Node;

/// Schema Component: Identity-Constraint Definition, a kind of Annotated Component (§3.11)
#[derive(Clone, Debug)]
pub struct IdentityConstraintDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: NCName,
    pub target_namespace: Option<AnyURI>,
    pub identity_constraint_category: IdentityConstraintCategory,
    pub selector: XPathExpression,
    pub fields: Sequence<XPathExpression>,
    pub referenced_key: Option<Ref<IdentityConstraintDefinition>>,
}

#[derive(Clone, Debug)]
pub enum IdentityConstraintCategory {
    Key,
    KeyRef,
    Unique,
}

impl Named for IdentityConstraintDefinition {
    fn name(&self) -> Option<QName> {
        Some(QName::with_optional_namespace(
            self.target_namespace.as_ref(),
            &self.name,
        ))
    }
}

impl NamedXml for IdentityConstraintDefinition {
    fn get_name_from_xml(icd: Node, schema: Node) -> QName {
        // {name} The ·actual value· of the name [attribute]
        let name = icd
            .attribute("name")
            .map(|v| actual_value::<String>(v, icd))
            .unwrap();

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the <schema> ancestor element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<String>(v, schema));

        QName::with_optional_namespace(target_namespace, name)
    }
}

impl IdentityConstraintDefinition {
    pub const KEY_TAG_NAME: &'static str = "key";
    pub const KEYREF_TAG_NAME: &'static str = "keyref";
    pub const UNIQUE_TAG_NAME: &'static str = "unique";
    pub const TAG_NAMES: [&'static str; 3] = [
        Self::KEY_TAG_NAME,
        Self::KEYREF_TAG_NAME,
        Self::UNIQUE_TAG_NAME,
    ];

    pub(super) fn map_from_xml_local(
        context: &mut MappingContext,
        icd: Node,
        schema: Node,
    ) -> Result<Ref<Self>, XsdError> {
        let self_ref = context.reserve();
        Self::map_from_xml(context, icd, schema, self_ref)
    }

    fn map_from_xml(
        context: &mut MappingContext,
        icd: Node,
        schema: Node,
        self_ref: Ref<Self>,
    ) -> Result<Ref<Self>, XsdError> {
        let QName {
            local_name: name,
            namespace_name: target_namespace,
        } = Self::get_name_from_xml(icd, schema);

        // {identity-constraint category}
        //   One of key, keyref or unique, depending on the item.
        let identity_constraint_category = match icd.tag_name().name() {
            "key" => IdentityConstraintCategory::Key,
            "keyref" => IdentityConstraintCategory::KeyRef,
            "unique" => IdentityConstraintCategory::Unique,
            _ => panic!("Invalid ICD element name {:?}", icd.tag_name().name()),
        };

        // {selector}
        //   An XPath Expression property record, as described in section XML Representation of
        //   Assertion Schema Components (§3.13.2), with <selector> as the "host element" and xpath
        //   as the designated expression [attribute].
        // TODO absence?
        let selector = icd
            .children()
            .find(|c| c.tag_name().name() == "selector")
            .unwrap();
        let xpath = selector.attribute("xpath").unwrap();
        let selector = XPathExpression::map_from_xml(xpath, selector, schema);

        // {fields}
        //   A sequence of XPath Expression property records, corresponding to the <field> element
        //   information item [children], in order, following the rules given in XML Representation
        //   of Assertion Schema Components (§3.13.2), with <field> as the "host element" and xpath
        //   as the designated expression [attribute].
        let fields = icd
            .children()
            .filter(|c| c.tag_name().name() == "field")
            .map(|field| {
                let xpath = field.attribute("xpath").unwrap();
                XPathExpression::map_from_xml(xpath, field, schema)
            })
            .collect();

        // {referenced key}
        //   If the item is a <keyref>, the identity-constraint definition ·resolved· to by the
        //   ·actual value· of the refer [attribute], otherwise ·absent·.
        let referenced_key = if icd.tag_name().name() == "keyref" {
            let refer: QName = actual_value(icd.attribute("refer").unwrap(), icd);
            Some(
                context
                    .resolve(&refer)
                    .ok_or(XsdError::UnresolvedReference(refer))?,
            )
        } else {
            None
        };

        // {annotations}
        //   The ·annotation mapping· of the set of elements containing the <key>, <keyref>, or
        //   <unique> element, whichever is present, and the <selector> and <field> [children], if
        //   present, as defined in XML Representation of Annotation Schema Components (§3.15.2).
        let ae = vec![icd];
        // TODO selector, field
        let annotations = Annotation::xml_element_set_annotation_mapping(context, &ae);

        Ok(context.insert(
            self_ref,
            Self {
                annotations,
                name,
                target_namespace,
                identity_constraint_category,
                selector,
                fields,
                referenced_key,
            },
        ))
    }
}

impl Component for IdentityConstraintDefinition {
    const DISPLAY_NAME: &'static str = "IdentityConstraintDefinition";
}

impl AnnotatedComponent for IdentityConstraintDefinition {
    fn annotations(&self) -> &[Ref<Annotation>] {
        &self.annotations
    }
}

impl TopLevelMappable for IdentityConstraintDefinition {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        icd: Node,
        schema: Node,
    ) -> Result<(), XsdError> {
        Self::map_from_xml(context, icd, schema, self_ref)?;
        Ok(())
    }
}
