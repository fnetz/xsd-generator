use std::borrow::Cow;

use dt_xsd::{RefNamed, SchemaComponentTable, particle::MaxOccurs};
use heck::{ToLowerCamelCase, ToUpperCamelCase};
use swc_common::{
    Span,
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
};
use swc_ecma_ast::{
    Decl, Expr, Ident, IdentName, Module, Program, Stmt, TsArrayType, TsEntityName,
    TsInterfaceBody, TsInterfaceDecl, TsPropertySignature, TsQualifiedName, TsType,
    TsTypeAliasDecl, TsTypeAnn, TsTypeElement, TsTypeRef, TsUnionType,
};

use crate::ist::{
    self, Compositor, ExternalKind, Member, Quant, TypeBinding, TypeIndex, builder::IstBuilder,
};

struct TypescriptGenerator<'a> {
    ist: &'a IstBuilder,
    components: &'a SchemaComponentTable,
    comments: SingleThreadedComments,
}

impl<'a> TypescriptGenerator<'a> {
    fn new(ist: &'a IstBuilder, components: &'a SchemaComponentTable) -> Self {
        Self {
            ist,
            components,
            comments: SingleThreadedComments::default(),
        }
    }
}

fn namespace_to_import_name(namespace: Option<&str>) -> String {
    if let Some(namespace) = namespace {
        namespace
            .replace(":", "_")
            .replace("/", "_")
            .to_lower_camel_case()
    } else {
        "noNamespace".to_string()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BuiltinSource {
    Primitive,
    HelperType,
}

fn get_builtin_source_name(name: dt_xsd::xstypes::QName) -> (BuiltinSource, &'static str) {
    use BuiltinSource::*;
    let (source, name) = match name.local_name.as_ref() {
        "boolean" => (Primitive, "boolean"),
        "double" => (Primitive, "number"),
        "float" => (Primitive, "number"),
        "long" => (Primitive, "number"),
        "int" => (Primitive, "number"),
        "short" => (Primitive, "number"),
        "byte" => (Primitive, "number"),
        "unsignedLong" => (Primitive, "number"),
        "unsignedInt" => (Primitive, "number"),
        "unsignedShort" => (Primitive, "number"),
        "unsignedByte" => (Primitive, "number"),
        "string" => (Primitive, "string"),
        "anyType" => (HelperType, "AnyType"),
        "anySimpleType" => (HelperType, "AnySimpleType"),
        "anyAtomicType" => (HelperType, "AnyAtomicType"),
        "error" => (HelperType, "Error"),
        "decimal" => (HelperType, "Decimal"),
        "dateTime" => (HelperType, "DateTime"),
        "duration" => (HelperType, "Duration"),
        "time" => (HelperType, "Time"),
        "date" => (HelperType, "Date"),
        "gMonth" => (HelperType, "GMonth"),
        "gMonthDay" => (HelperType, "GMonthDay"),
        "gDay" => (HelperType, "GDay"),
        "gYear" => (HelperType, "GYear"),
        "gYearMonth" => (HelperType, "GYearMonth"),
        "hexBinary" => (HelperType, "HexBinary"),
        "base64Binary" => (HelperType, "Base64Binary"),
        "anyURI" => (HelperType, "AnyURI"),
        "QName" => (HelperType, "QName"),
        "NOTATION" => (HelperType, "Notation"),
        "normalizedString" => (HelperType, "NormalizedString"),
        "token" => (HelperType, "Token"),
        "language" => (HelperType, "Language"),
        "NMTOKEN" => (HelperType, "NmToken"),
        "NMTOKENS" => (HelperType, "NmTokens"),
        "Name" => (HelperType, "Name"),
        "NCName" => (HelperType, "NcName"),
        "ID" => (HelperType, "Id"),
        "IDREF" => (HelperType, "IdRef"),
        "IDREFS" => (HelperType, "IdRefs"),
        "ENTITY" => (HelperType, "Entity"),
        "ENTITIES" => (HelperType, "Entities"),
        "integer" => (HelperType, "Integer"),
        "nonPositiveInteger" => (HelperType, "NonPositiveInteger"),
        "negativeInteger" => (HelperType, "NegativeInteger"),
        "nonNegativeInteger" => (HelperType, "NonNegativeInteger"),
        "positiveInteger" => (HelperType, "PositiveInteger"),
        "yearMonthDuration" => (HelperType, "YearMonthDuration"),
        "dayTimeDuration" => (HelperType, "DayTimeDuration"),
        "dateTimeStamp" => (HelperType, "DateTimeStamp"),
        _ => panic!("Unknown builtin type: {:?}", name.local_name),
    };
    (source, name)
}

impl TypescriptGenerator<'_> {
    /// Creates a new dummy span used for indexing comments.
    ///
    /// The current implementation of `dummy_with_cmt` returns a span with `lo == hi`, so using
    /// `lo()` for the comment position should be ok.
    ///
    /// `dummy_with_cmt` must be called withing a `GLOBALS.set(..., || { ... })` to make the GLOBAL
    /// thread local available, otherwise it panics.
    fn new_dummy_comment_span() -> Span {
        Span::dummy_with_cmt()
    }

    fn new_ident<'a>(name: impl Into<Cow<'a, str>>) -> Ident {
        // using `Cow` as bridge between `&str`/`String` and `Atom`
        let name: Cow<_> = name.into();
        Ident::new_no_ctxt(name.into(), Span::default())
    }

    fn leading_comment<'a>(&self, kind: CommentKind, comment: impl Into<Cow<'a, str>>) -> Span {
        let comment = comment.into();

        let pos = Self::new_dummy_comment_span();
        self.comments.add_leading(
            pos.lo(),
            Comment {
                kind,
                // Using a new span here, since `pos` is used for the item that is being commented
                span: Span::default(),
                text: comment.into(),
            },
        );
        pos
    }

    fn binding_name(binding: &TypeBinding) -> Cow<str> {
        // Mostly for development purposes, in practice, the name should never be None.
        binding
            .name
            .as_ref()
            .map(|name| name.name.to_upper_camel_case().into())
            .unwrap_or_else(|| match binding.type_ {
                ist::Type::Composite(_) => "UnnamedComposite".into(),
                ist::Type::Enum(_) => "UnnamedEnum".into(),
            })
    }

    fn gen_type_ref(&self, type_ref: &ist::TypeRef) -> TsTypeRef {
        let type_name = match type_ref {
            ist::TypeRef::Builtin(type_definition) => {
                let name = type_definition
                    .name(self.components)
                    .expect("Builtin type should have a name");
                let (source, name) = get_builtin_source_name(name);

                match source {
                    BuiltinSource::Primitive => {
                        TsEntityName::Ident(Ident::new_no_ctxt(name.into(), Span::default()))
                    }
                    BuiltinSource::HelperType => {
                        TsEntityName::TsQualifiedName(Box::new(TsQualifiedName {
                            span: Span::default(),
                            left: TsEntityName::Ident(Ident::new_no_ctxt(
                                "ts_builtins".into(),
                                Span::default(),
                            )),
                            right: IdentName::new(name.into(), Span::default()),
                        }))
                    }
                }
            }
            ist::TypeRef::Internal(type_index) => {
                let name = Self::binding_name(&self.ist.types[type_index]);
                TsEntityName::Ident(Self::new_ident(name))
            }
            ist::TypeRef::External(qname, kind) => {
                let import_name = namespace_to_import_name(qname.namespace_name());
                let kind_name = match kind {
                    ExternalKind::TypeDefinition => "Type",
                    ExternalKind::AttributeDeclaration => "Attr",
                    ExternalKind::ElementDeclaration => "Elem",
                    ExternalKind::AttributeGroupDefinition => "AttrGroup",
                    ExternalKind::ModelGroupDefinition => "ModelGroup",
                    ExternalKind::NotationDeclaration => "Notation",
                    ExternalKind::IdentityConstraintDefinition => "IdentityConstraint",
                };

                let name = TsEntityName::TsQualifiedName(Box::new(TsQualifiedName {
                    span: Span::default(),
                    left: TsEntityName::Ident(Ident::new_no_ctxt(
                        import_name.into(),
                        Span::default(),
                    )),
                    right: IdentName::new(kind_name.to_string().into(), Span::default()),
                }));

                let symbol_name = qname.local_name().to_upper_camel_case();

                TsEntityName::TsQualifiedName(Box::new(TsQualifiedName {
                    span: Span::default(),
                    left: name,
                    right: IdentName::new(symbol_name.into(), Span::default()),
                }))
            }
        };

        TsTypeRef {
            span: Span::default(),
            type_name,
            type_params: None,
        }
    }

    fn undefined_type() -> TsType {
        TsTypeRef {
            span: Span::default(),
            type_name: TsEntityName::Ident(Self::new_ident("undefined")),
            type_params: None,
        }
        .into()
    }

    fn quantified_type(&self, quant: &Quant, type_: TsType) -> TsType {
        // TODO: Tuple type for special cases
        match quant.into_min_max() {
            (0, MaxOccurs::Count(1)) => TsType::TsUnionOrIntersectionType(
                TsUnionType {
                    span: Span::default(),
                    types: vec![Box::new(type_), Box::new(Self::undefined_type())],
                }
                .into(),
            ),
            (1, MaxOccurs::Count(1)) => type_,
            _ => TsArrayType {
                span: Span::default(),
                elem_type: Box::new(type_),
            }
            .into(),
        }
    }

    fn property_for_structure_field(&self, field: &Member) -> TsTypeElement {
        let type_ = self.gen_type_ref(&field.type_).into();

        // For optional fields (i.e. occurrence 0..1), we want to use `?` directly in the property
        // signature instead of the type.
        let optional = field.quant == Quant::zero_or_one();
        let type_ = if !optional {
            self.quantified_type(&field.quant, type_)
        } else {
            type_
        };

        TsPropertySignature {
            span: Span::default(),
            readonly: false,
            key: Box::new(Expr::Ident(Ident::new_no_ctxt(
                field.name.as_ref().map_or_else(
                    || "unnamedField".into(),
                    |name| name.name.to_lower_camel_case().into(),
                ),
                Span::default(),
            ))),
            computed: false,
            optional,
            type_ann: Some(Box::new(TsTypeAnn {
                span: Span::default(),
                type_ann: Box::new(type_),
            })),
        }
        .into()
    }

    fn statement_for_structure(
        &self,
        index: TypeIndex,
        binding: &TypeBinding,
        structure: &ist::CompositeType,
    ) -> Stmt {
        if structure.is_thin() {
            return self.statement_for_thin_struct(index, binding, structure);
        }

        let ts_interface = TsInterfaceDecl {
            span: self.leading_comment(
                CommentKind::Line,
                format!("{index:?} {:?}", binding.visibility),
            ),
            id: Self::new_ident(Self::binding_name(binding)),
            declare: false,
            type_params: None,
            extends: Vec::new(),
            body: TsInterfaceBody {
                span: Span::default(),
                body: structure
                    .members
                    .iter()
                    .map(|field| self.property_for_structure_field(field))
                    .collect(),
            },
        };
        Decl::TsInterface(Box::new(ts_interface)).into()
    }

    fn statement_for_union(
        &self,
        index: TypeIndex,
        binding: &TypeBinding,
        union_: &ist::CompositeType,
    ) -> Stmt {
        let ts_type = TsTypeAliasDecl {
            span: self.leading_comment(
                CommentKind::Line,
                format!("{index:?} {:?}", binding.visibility),
            ),
            declare: false,
            id: Self::new_ident(Self::binding_name(binding)),
            type_params: None,
            type_ann: Box::new(TsType::TsUnionOrIntersectionType(
                TsUnionType {
                    span: Span::default(),
                    types: union_
                        .members
                        .iter()
                        .map(|variant| TsType::TsTypeRef(self.gen_type_ref(&variant.type_)))
                        .map(Box::new)
                        .collect(),
                }
                .into(),
            )),
        };
        Decl::TsTypeAlias(Box::new(ts_type)).into()
    }

    fn statement_for_thin_struct(
        &self,
        index: TypeIndex,
        binding: &TypeBinding,
        structure: &ist::CompositeType,
    ) -> Stmt {
        let single_field = structure.members.first().unwrap();

        let type_ref = self.gen_type_ref(&single_field.type_);
        let ts_type = TsTypeAliasDecl {
            span: self.leading_comment(
                CommentKind::Line,
                format!("{index:?} {:?}", binding.visibility),
            ),
            declare: false,
            id: Self::new_ident(Self::binding_name(binding)),
            type_params: None,
            type_ann: Box::new(self.quantified_type(&single_field.quant, type_ref.into())),
        };
        Decl::TsTypeAlias(Box::new(ts_type)).into()
    }

    fn generate(&self) -> Program {
        let mut items = Vec::new();

        // Keep an (arbitrary) fixed order of the types to prevent shifting between runs
        let mut types = self
            .ist
            .types
            .iter()
            .map(|(k, v)| (*k, v))
            .collect::<Vec<_>>();
        types.sort_by_key(|(k, _)| k.sort_key());

        for (index, binding) in types {
            if !binding.should_emit() {
                continue;
            }

            let stmt = match &binding.type_ {
                ist::Type::Composite(comp) => match comp.compositor {
                    Compositor::Structure => self.statement_for_structure(index, binding, comp),
                    Compositor::Union => self.statement_for_union(index, binding, comp),
                },
                ist::Type::Enum(_enum_type) => todo!(),
            };
            items.push(stmt.into())
        }

        Program::Module(Module {
            body: items,
            shebang: None,
            span: self.leading_comment(
                CommentKind::Line,
                concat!(
                    " Generated by ",
                    env!("CARGO_PKG_NAME"),
                    " ",
                    env!("CARGO_PKG_VERSION")
                ),
            ),
        })
    }
}

pub fn generate_v2(ist: &IstBuilder, components: &SchemaComponentTable) -> String {
    // See [`TypescriptGenerator::new_dummy_comment_span`] for why this is needed.
    swc_common::GLOBALS.set(&Default::default(), || {
        let ctx = TypescriptGenerator::new(ist, components);
        let program = ctx.generate();
        swc_ecma_codegen::to_code_with_comments(Some(&ctx.comments), &program)
    })
}
