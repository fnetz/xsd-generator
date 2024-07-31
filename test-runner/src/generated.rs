//!Generated by dt-xsd-generator 0.1.0
#![allow(dead_code)]
pub struct TypeType(String);
pub struct HrefType(dt_builtins::AnyURI);
pub struct RoleType(dt_builtins::AnyURI);
pub struct ArcroleType(dt_builtins::AnyURI);
pub struct TitleAttrType(String);
pub struct ShowType(String);
pub struct ActuateType(String);
pub struct LabelType(String);
pub struct FromType(String);
pub struct ToType(String);
pub struct Simple {
    pub wildcard: Vec<()>,
}
pub enum Extended {
    Title(TitleEltType),
    Resource(ResourceType),
    Locator(LocatorType),
    Arc(ArcType),
}
pub struct TitleEltType {
    pub wildcard: Vec<()>,
}
pub struct ResourceType {
    pub wildcard: Vec<()>,
}
pub struct LocatorType {
    pub title: Vec<TitleEltType>,
}
pub struct ArcType {
    pub title: Vec<TitleEltType>,
}
pub struct Status(String);
pub struct BugUri(dt_builtins::AnyURI);
pub struct TestOutcome(String);
pub struct ExpectedOutcomeInner(String);
pub enum ExpectedOutcome {
    TestOutcome(TestOutcome),
    Unnamed(ExpectedOutcomeInner),
}
pub struct VersionInfo(Vec<VersionToken>);
pub struct KnownXsdVersion(String);
pub struct Xsd10Editions(String);
pub struct XmlSubstrate(String);
pub struct UnicodeVersions(String);
pub struct RuntimeSchemaError(String);
pub struct XpathInCta(String);
pub struct XdmFiltering(String);
pub enum KnownToken {
    KnownXsdVersion(KnownXsdVersion),
    Xsd10Editions(Xsd10Editions),
    XmlSubstrate(XmlSubstrate),
    UnicodeVersions(UnicodeVersions),
    RuntimeSchemaError(RuntimeSchemaError),
    XpathInCta(XpathInCta),
    XdmFiltering(XdmFiltering),
}
pub enum VersionToken {
    KnownToken(KnownToken),
    Decimal(dt_builtins::Decimal),
    Nmtoken(dt_builtins::NmToken),
}
pub struct StatusEntry {
    pub annotation: Vec<Annotation>,
    pub status: Status,
    pub date: dt_builtins::Date,
    pub bugzilla: Option<BugUri>,
}
pub struct Ref {
    pub annotation: Vec<Annotation>,
    pub r#type: Option<TypeType>,
    pub href: Option<HrefType>,
}
pub struct Role(String);
pub struct SchemaDocumentRef {
    pub annotation: Vec<Annotation>,
    pub role: Option<Role>,
}
pub struct TestSuite {
    pub annotation: Vec<Annotation>,
    pub test_set_ref: Vec<Ref>,
    pub name: dt_builtins::Name,
    pub release_date: dt_builtins::Date,
    pub schema_version: String,
    pub version: Option<VersionInfo>,
}
pub struct TestSet {
    pub annotation: Vec<Annotation>,
    pub test_group: Vec<TestGroup>,
    pub contributor: String,
    pub name: dt_builtins::Name,
    pub version: Option<VersionInfo>,
}
pub struct TestGroup {
    pub annotation: Vec<Annotation>,
    pub documentation_reference: Vec<Ref>,
    pub schema_test: Option<SchemaTest>,
    pub instance_test: Vec<InstanceTest>,
    pub name: dt_builtins::Name,
    pub version: Option<VersionInfo>,
}
pub struct SchemaTest {
    pub annotation: Vec<Annotation>,
    pub schema_document: Vec<SchemaDocumentRef>,
    pub expected: Vec<Expected>,
    pub current: Option<StatusEntry>,
    pub prior: Vec<StatusEntry>,
    pub name: dt_builtins::Name,
    pub version: Option<VersionInfo>,
}
pub struct InstanceTest {
    pub annotation: Vec<Annotation>,
    pub instance_document: Ref,
    pub expected: Vec<Expected>,
    pub current: Option<StatusEntry>,
    pub prior: Vec<StatusEntry>,
    pub name: dt_builtins::Name,
    pub version: Option<VersionInfo>,
}
pub struct Expected {
    pub validity: ExpectedOutcome,
    pub version: Option<VersionInfo>,
}
pub struct PublicationPermission(String);
pub struct TestSuiteResults {
    pub annotation: Vec<Annotation>,
    pub test_result: Vec<TestResult>,
    pub suite: dt_builtins::Name,
    pub processor: String,
    pub submit_date: dt_builtins::Date,
    pub publication_permission: Option<PublicationPermission>,
}
pub struct TestResult {
    pub annotation: Vec<Annotation>,
    pub validity: TestOutcome,
    pub set: dt_builtins::Name,
    pub group: dt_builtins::Name,
    pub test: dt_builtins::Name,
    pub normalized_load: Option<dt_builtins::Decimal>,
}
pub enum Annotation {
    Appinfo(Appinfo),
    Documentation(Documentation),
}
pub struct Appinfo {
    pub wildcard: (),
    pub source: Option<dt_builtins::AnyURI>,
}
pub struct LangInner(String);
pub enum Lang {
    Language(dt_builtins::Language),
    Unnamed(LangInner),
}
pub struct Documentation {
    pub wildcard: (),
    pub source: Option<dt_builtins::AnyURI>,
    pub lang: Option<Lang>,
}
