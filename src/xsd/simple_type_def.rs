use super::{
    annotation::Annotation,
    attribute_decl::AttributeDeclaration,
    builtins::{XS_ANY_SIMPLE_TYPE_NAME, XS_ANY_TYPE_NAME, XS_NAMESPACE},
    complex_type_def::ComplexTypeDefinition,
    components::{Component, Named, RefNamed},
    constraining_facet::{ConstrainingFacet, WhiteSpace, WhiteSpaceValue},
    element_decl::ElementDeclaration,
    fundamental_facet::FundamentalFacet,
    mapping_context::{MappingContext, TopLevelMappable},
    shared::TypeDefinition,
    values::actual_value,
    xstypes::{AnyURI, NCName, QName, Sequence, Set},
    Ref,
};
use roxmltree::Node;

/// Simple Type Definition, a kind of [Type Definition](super::shared::TypeDefinition), §3.16
#[derive(Clone, Debug)]
pub struct SimpleTypeDefinition {
    pub annotations: Sequence<Ref<Annotation>>,
    pub name: Option<NCName>,
    pub target_namespace: Option<AnyURI>,
    pub final_: Set<DerivationMethod>,
    /// Required if `name` is `None`, otherwise must be `None`.
    pub context: Option<Context>,
    /// With one exception, the `base_type_definition` of any Simple Type Definition is a Simple
    /// Type Definition. The exception is `anySimpleType`, which has `anyType`, a
    /// [Complex Type Definition](ComplexTypeDefinition), as its `base_type_definition`.
    pub base_type_definition: TypeDefinition,
    pub facets: Set<Ref<ConstrainingFacet>>,
    pub fundamental_facets: Set<FundamentalFacet>,
    /// Required for all Simple Type Definitions except `anySimpleType`, in which it is `None`.
    pub variety: Option<Variety>,
    /// With one exception, required if `variety` is atomic, otherwise must be `None`. The exception
    /// is `anyAtomicType`, whose `primitive_type_definition` is `None`. If not `None`, must be a
    /// _primitive_ built-in definition.
    pub primitive_type_definition: Option<Ref<SimpleTypeDefinition>>,
    /// Required if `variety` is [list](Variety::List), otherwise must be `None`. The value of this
    /// property must be a _primitive_ or _ordinary_ simple type definition with `variety` =
    /// [atomic](Variety::Atomic), or an _ordinary_ simple type definition with `variety` =
    /// [union](Variety::Union) whose basic members are all atomic; the value must not itself be a
    /// list type (have `variety` = [list](Variety::List)) or have any basic members which are
    /// list types.
    pub item_type_definition: Option<Ref<SimpleTypeDefinition>>,
    /// A sequence of _primitive_ or _ordinary_ Simple Type Definition components. Must be present
    /// (but may be empty) if `variety` is [union](Variety::Union), otherwise must be `None`.
    ///
    /// The sequence may contain any _primitive_ or _ordinary_ simple type definition, but must not
    /// contain any _special_ type definitions.
    pub member_type_definitions: Option<Sequence<Ref<SimpleTypeDefinition>>>,
}

#[derive(Clone, Debug)]
pub enum DerivationMethod {
    Extension,
    Restriction,
    List,
    Union,
}

#[derive(Clone, Debug)]
pub enum Context {
    Attribute(Ref<AttributeDeclaration>),
    Element(Ref<ElementDeclaration>),
    ComplexType(Ref<ComplexTypeDefinition>),
    SimpleType(Ref<SimpleTypeDefinition>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Variety {
    Atomic,
    List,
    Union,
}

#[derive(PartialEq, Eq)]
enum ChildType {
    Restriction,
    List,
    Union,
}

impl SimpleTypeDefinition {
    pub const TAG_NAME: &'static str = "simpleType";

    pub(super) fn name_from_xml(simple_type: Node, schema: Node) -> Option<QName> {
        // {name}
        //   The ·actual value· of the name [attribute] if present on the <simpleType> element,
        //   otherwise ·absent·.
        let name = simple_type
            .attribute("name")
            .map(|v| actual_value::<String>(v, simple_type));

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the ancestor <schema> element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<AnyURI>(v, simple_type));

        name.map(|name| QName::with_optional_namespace(target_namespace, name))
    }

    /// Maps a Simple Type Definition from its XML representation
    ///
    /// In case of non-top-level Simple Type Definitions, the parent must be `Some`; see
    /// Specification pt. 1, §3.16.2.1, {context}, clause 2
    pub(super) fn map_from_xml(
        ctx: &mut MappingContext,
        simple_type: Node,
        schema: Node,
        tlref: Option<Ref<Self>>,
        parent: Option<Context>,
    ) -> Ref<Self> {
        assert_eq!(simple_type.tag_name().name(), Self::TAG_NAME);

        let self_ref = tlref.unwrap_or_else(|| ctx.reserve());

        // {name}
        //   The ·actual value· of the name [attribute] if present on the <simpleType> element,
        //   otherwise ·absent·.
        let name = simple_type
            .attribute("name")
            .map(|v| actual_value::<String>(v, simple_type));

        // {target namespace}
        //   The ·actual value· of the targetNamespace [attribute] of the ancestor <schema> element
        //   information item if present, otherwise ·absent·.
        let target_namespace = schema
            .attribute("targetNamespace")
            .map(|v| actual_value::<AnyURI>(v, simple_type));

        let (child_type, child) = if let Some(restriction) = simple_type
            .children()
            .find(|e| e.tag_name().name() == "restriction")
        {
            (ChildType::Restriction, restriction)
        } else if let Some(list) = simple_type
            .children()
            .find(|e| e.tag_name().name() == "list")
        {
            (ChildType::List, list)
        } else if let Some(union) = simple_type
            .children()
            .find(|e| e.tag_name().name() == "union")
        {
            (ChildType::Union, union)
        } else {
            unreachable!()
        };

        // {base type definition} The appropriate case among the following:
        let base_type_definition = if child_type == ChildType::Restriction {
            // 1 If the <restriction> alternative is chosen, then the type definition ·resolved· to
            //   by the ·actual value· of the base [attribute] of <restriction>, if present,
            //   otherwise the type definition corresponding to the <simpleType> among the
            //   [children] of <restriction>.
            child
                .attribute("base")
                .map(|v| actual_value::<QName>(v, child))
                .map(|name| ctx.resolve(&name))
                .unwrap_or_else(|| {
                    let st = simple_type
                        .children()
                        .find(|c| c.tag_name().name() == Self::TAG_NAME)
                        .map(|st| {
                            Self::map_from_xml(
                                ctx,
                                st,
                                schema,
                                None,
                                Some(Context::SimpleType(self_ref)),
                            )
                        })
                        .unwrap();
                    TypeDefinition::Simple(st)
                })
        } else {
            // 2 If the <list> or <union> alternative is chosen, then ·xs:anySimpleType·.
            ctx.resolve(&XS_ANY_SIMPLE_TYPE_NAME)
        };

        // {final}
        //   A subset of {restriction, extension, list, union}, determined as follows.
        //   Let FS be the ·actual value· of the final [attribute], if present, otherwise the
        //   ·actual value· of the finalDefault [attribute] of the ancestor schema element, if
        //   present, otherwise the empty string.
        let fs = simple_type
            .attribute("final")
            .or_else(|| schema.attribute("finalDefault"))
            .map(|v| actual_value::<String>(v, simple_type))
            .unwrap_or_default();
        // Then the property value is the appropriate case
        //   among the following:
        //   1 If ·FS· is the empty string, then the empty set;
        //   2 If ·FS· is "#all", then {restriction, extension, list, union};
        //   3 otherwise Consider ·FS· as a space-separated list, and include restriction if
        //     "restriction" is in that list, and similarly for extension, list and union.
        let final_ = if fs.is_empty() {
            Set::new()
        } else if fs == "#all" {
            [
                DerivationMethod::Restriction,
                DerivationMethod::Extension,
                DerivationMethod::List,
                DerivationMethod::Union,
            ]
            .into_iter()
            .collect()
        } else {
            let fs = fs.split_whitespace();
            fs.map(|v| match v {
                "restriction" => DerivationMethod::Restriction,
                "extension" => DerivationMethod::Extension,
                "list" => DerivationMethod::List,
                "union" => DerivationMethod::Union,
                _ => panic!("Invalid value in final set"),
            })
            .collect()
        };

        // {facets} The appropriate case among the following:
        let facets = match child_type {
            // 1 If the <restriction> alternative is chosen and the children of the <restriction>
            //   element are all either <simpleType> elements, <annotation> elements, or elements
            //   which specify constraining facets supported by the processor, then the set of
            //   Constraining Facet components obtained by ·overlaying· the {facets} of the {base
            //   type definition} with the set of Constraining Facet components corresponding to
            //   those [children] of <restriction> which specify facets, as defined in Simple Type
            //   Restriction (Facets) (§3.16.6.4).
            // 2 If the <restriction> alternative is chosen and the children of the <restriction>
            //   element include at least one element of which the processor has no prior knowledge
            //   (i.e. not a <simpleType> element, an <annotation> element, or an element denoting a
            //   constraining facet known to and supported by the processor), then the <simpleType>
            //   element maps to no component at all (but is not in error solely on account of the
            //   presence of the unknown element).
            ChildType::Restriction => {
                // TODO ·overlaying· algorithm
                let mut facet_nodes = Vec::new();
                for facet in child.children() {
                    if !facet.is_element() {
                        continue;
                    }
                    if [Self::TAG_NAME, Annotation::TAG_NAME].contains(&facet.tag_name().name()) {
                        continue;
                    }
                    facet_nodes.push(facet);
                }
                ConstrainingFacet::map_from_xml(ctx, &facet_nodes, schema).unwrap()
            }
            // 3 If the <list> alternative is chosen, then a set with one member, a whiteSpace facet
            //   with {value} = collapse and {fixed} = true.
            ChildType::List => {
                let ws = ctx.create(ConstrainingFacet::WhiteSpace(WhiteSpace::new(
                    WhiteSpaceValue::Collapse,
                    true,
                )));
                [ws].into_iter().collect()
            }
            // 4 otherwise the empty set
            _ => Set::new(),
        };

        // {context} The appropriate case among the following:
        let context = if simple_type.has_attribute("name") {
            // 1 If the name [attribute] is present, then ·absent·
            None
        } else {
            // 2 otherwise the appropriate case among the following:
            //   (see spec; in our case, the caller already knows the appropriate case)
            let context = parent.expect("Unnamed simple type must have a parent");
            Some(context)
        };

        // {variety}
        //   If the <list> alternative is chosen, then list, otherwise if the <union> alternative is
        //   chosen, then union, otherwise (the <restriction> alternative is chosen), then the
        //   {variety} of the {base type definition}.
        let variety = match child_type {
            ChildType::List => Some(Variety::List),
            ChildType::Union => Some(Variety::Union),
            ChildType::Restriction => {
                // TODO what happens when the parent variety is ·absent·?
                // TODO always simple?
                ctx.request(base_type_definition.simple().unwrap()).variety
            }
        };

        // {annotations}
        //   The ·annotation mapping· of the set of elements containing the <simpleType>, and one of
        //   the <restriction>, <list> or <union> [children], whichever is present, as defined in
        //   XML Representation of Annotation Schema Components (§3.15.2).
        let annotations =
            Annotation::xml_element_set_annotation_mapping(ctx, &[simple_type, child]);

        let mut primitive_type_definition = None;
        let mut item_type_definition = None;
        let mut member_type_definitions = None;

        match variety.unwrap() {
            Variety::Atomic => {
                // {primitive type definition}
                // From among the ·ancestors· of this Simple Type Definition, that Simple Type
                // Definition which corresponds to a primitive datatype.
                let ancestors = std::iter::once(base_type_definition)
                    .chain(base_type_definition.ancestors(ctx.components()));
                primitive_type_definition = Some(
                    ancestors
                        .take_while(|r| {
                            r.name(ctx.components()).as_ref() != Some(&XS_ANY_TYPE_NAME)
                        })
                        .find(|t| t.is_primitive(ctx.components()))
                        .unwrap() // TODO can this fail?
                        .simple()
                        .unwrap(),
                );
            }
            Variety::List => {
                let list = child;

                // {item type definition} The appropriate case among the following:
                item_type_definition = Some(
                    if base_type_definition.name(ctx.components()).as_ref()
                        == Some(&XS_ANY_SIMPLE_TYPE_NAME)
                    {
                        // 1 If the {base type definition} is ·xs:anySimpleType·, then the
                        //   Simple Type Definition
                        //   (a) ·resolved· to by the ·actual value· of the itemType [attribute] of
                        //       <list>, or
                        //   (b), corresponding to the <simpleType> among the [children] of <list>,
                        //   whichever is present.
                        list.attribute("itemType")
                            .map(|item_type| actual_value::<QName>(item_type, list))
                            .map(|item_type| ctx.resolve(&item_type))
                            .or_else(|| {
                                list.children()
                                    .find(|c| c.tag_name().name() == Self::TAG_NAME)
                                    .map(|simple_type| {
                                        Self::map_from_xml(
                                            ctx,
                                            simple_type,
                                            schema,
                                            None,
                                            Some(Context::SimpleType(self_ref)),
                                        )
                                    })
                            })
                            .unwrap()
                    } else {
                        // 2 otherwise (that is, the {base type definition} is not
                        //   ·xs:anySimpleType·), the {item type definition} of the
                        //   {base type definition}.
                        ctx.request(base_type_definition.simple().unwrap())
                            .item_type_definition
                            .unwrap() // TODO unwrap allowed?
                    },
                )
            }
            Variety::Union => {
                let union_ = child;

                // {member type definitions} The appropriate case among the following:
                let base_type_definition = ctx.request(base_type_definition.simple().unwrap());
                member_type_definitions = Some(
                    if base_type_definition.name().as_ref() == Some(&XS_ANY_SIMPLE_TYPE_NAME) {
                        // 1 If the {base type definition} is ·xs:anySimpleType·, then the sequence of
                        //   Simple Type Definitions
                        //   (a) ·resolved· to by the items in the ·actual value· of the memberTypes
                        //       [attribute] of <union>, if any, and
                        //   (b) corresponding to the <simpleType>s among the [children] of <union>,
                        //       if any, in order.
                        let mut member_types = union_
                            .attribute("memberTypes")
                            .map(|member_types| actual_value::<Vec<QName>>(member_types, union_))
                            .map(|member_types| {
                                member_types
                                    .into_iter()
                                    .map(|member_type| {
                                        ctx.resolve::<Ref<SimpleTypeDefinition>>(&member_type)
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();

                        member_types.extend(
                            union_
                                .children()
                                .filter(|c| c.tag_name().name() == Self::TAG_NAME)
                                .map(|simple_type| {
                                    Self::map_from_xml(
                                        ctx,
                                        simple_type,
                                        schema,
                                        None,
                                        Some(Context::SimpleType(self_ref)),
                                    )
                                }),
                        );

                        member_types
                    } else {
                        // 2 otherwise (that is, the {base type definition} is not ·xs:anySimpleType·),
                        //   the {member type definitions} of the {base type definition}.
                        base_type_definition
                            .member_type_definitions
                            .clone()
                            .unwrap()
                    },
                );
            }
        }

        // TODO
        let fundamental_facets = Set::new();

        ctx.insert(
            self_ref,
            Self {
                name,
                target_namespace,
                base_type_definition,
                final_,
                context,
                variety,
                facets,
                annotations,
                fundamental_facets,
                primitive_type_definition,
                item_type_definition,
                member_type_definitions,
            },
        );

        self_ref
    }

    pub fn is_primitive(&self) -> bool {
        if self.target_namespace.as_deref() != Some(XS_NAMESPACE) {
            false
        } else if let Some(name) = self.name.as_ref() {
            // TODO "All ·primitive· datatypes have anyAtomicType as their ·base type·" -> optimize
            matches!(
                name.as_str(),
                "string"
                    | "boolean"
                    | "decimal"
                    | "float"
                    | "double"
                    | "duration"
                    | "dateTime"
                    | "time"
                    | "date"
                    | "gYearMonth"
                    | "gYear"
                    | "gMonthDay"
                    | "gDay"
                    | "gMonth"
                    | "hexBinary"
                    | "base64Binary"
                    | "anyURI"
                    | "QName"
                    | "NOTATION"
            )
        } else {
            false
        }
    }
}

impl Component for SimpleTypeDefinition {
    const DISPLAY_NAME: &'static str = "SimpleTypeDefinition";
}

impl Named for SimpleTypeDefinition {
    fn name(&self) -> Option<QName> {
        self.name.as_ref().map(|local_name| {
            QName::with_optional_namespace(self.target_namespace.as_ref(), local_name)
        })
    }
}

impl TopLevelMappable for SimpleTypeDefinition {
    fn map_from_top_level_xml(
        context: &mut MappingContext,
        self_ref: Ref<Self>,
        simple_type: Node,
        schema: Node,
    ) {
        Self::map_from_xml(context, simple_type, schema, Some(self_ref), None);
    }
}
