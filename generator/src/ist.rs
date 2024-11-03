pub type Name = String;

pub struct TypeRef(usize);

pub struct StructureType {
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: Option<Name>,
    pub type_: FieldType,
    pub documentation: Option<String>,
}

pub struct FieldType {
    pub name: Name,
    pub is_optional: bool,
}

pub struct EnumType {
    pub name: Name,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub name: Name,
    pub documentation: Option<String>,
}

pub struct TaggedUnionType {
    pub name: Name,
    pub variants: Vec<TaggedUnionVariant>,
}

pub struct TaggedUnionVariant {
    pub name: Name,
    pub type_: FieldType,
    pub documentation: Option<String>,
}

pub enum Type {
    Builtin(Name),
    Structure(StructureType),
    Enum(EnumType),
    TaggedUnion(TaggedUnionType),
}

pub struct TypeBinding {
    pub name: Option<Name>,
    pub type_: Type,
    pub global: bool,
    pub documentation: Option<String>,
}
