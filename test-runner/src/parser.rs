use super::generated::*;
use roxmltree::Node;

const XLINK: &str = "http://www.w3.org/1999/xlink";

impl Annotation {
    pub fn from_xml(_node: Node) -> Self {
        // TODO this is generated wrong, should be list of Annotation
        let result = Annotation::Documentation(Documentation {
            wildcard: (),
            source: None,
            lang: None,
        });
        result
    }
}

impl Ref {
    pub fn from_xml(node: Node) -> Self {
        let mut result = Ref {
            annotation: vec![],
            r#type: node
                .attribute((XLINK, "type"))
                .map(|t| TypeType(t.into()))
                .or_else(|| Some(TypeType("locator".into()))),
            // TODO: optional with default value should not generate Option<...>
            href: node
                .attribute((XLINK, "href"))
                .map(|h| HrefType(dt_builtins::AnyURI(h.into()))),
        };
        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "annotation" => result.annotation.push(Annotation::from_xml(child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}

impl VersionInfo {
    pub fn from_xml(value: &str) -> Self {
        VersionInfo(
            value
                .trim()
                .split_whitespace()
                .map(|s| VersionToken::Nmtoken(dt_builtins::NmToken(s.into())))
                .collect(),
        )
    }
}

impl TestSuite {
    pub fn from_xml(node: Node) -> Self {
        let mut result = TestSuite {
            annotation: vec![],
            test_set_ref: vec![],
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            release_date: dt_builtins::Date(node.attribute("releaseDate").unwrap().into()),
            schema_version: node.attribute("schemaVersion").unwrap().into(),
            version: node.attribute("version").map(VersionInfo::from_xml),
        };
        #[derive(Copy, Clone, Debug)]
        enum State {
            Annotation,
            TestSetRef,
        }
        let mut state = State::Annotation;
        for child in node.children().filter(|n| n.is_element()) {
            match (state, child.tag_name().name()) {
                (State::Annotation, "annotation") => {
                    result.annotation.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::TestSetRef, "testSetRef") => {
                    state = State::TestSetRef;
                    result.test_set_ref.push(Ref::from_xml(child))
                }
                _ => unimplemented!(
                    "unexpected tag: {} (in state {state:?})",
                    child.tag_name().name()
                ),
            }
        }
        result
    }
}

impl TestOutcome {
    pub fn from_xml(value: &str) -> Option<Self> {
        match value {
            "valid" => Some(TestOutcome(value.into())),
            "invalid" => Some(TestOutcome(value.into())),
            "notKnown" => Some(TestOutcome(value.into())),
            "runtime-schema-error" => Some(TestOutcome(value.into())),
            _ => None,
        }
    }
}

impl ExpectedOutcome {
    pub fn from_xml(value: &str) -> Self {
        if let Some(outcome) = TestOutcome::from_xml(value) {
            ExpectedOutcome::TestOutcome(outcome)
        } else {
            ExpectedOutcome::Unnamed(match value {
                "implementation-defined" => ExpectedOutcomeInner(value.into()),
                "implementation-dependent" => ExpectedOutcomeInner(value.into()),
                "indeterminate" => ExpectedOutcomeInner(value.into()),
                "invalid-latent" => ExpectedOutcomeInner(value.into()),
                _ => unimplemented!("unexpected value: {}", value),
            })
        }
    }
}

impl Expected {
    pub fn from_xml(node: Node) -> Self {
        let result = Expected {
            validity: ExpectedOutcome::from_xml(node.attribute("validity").unwrap()),
            version: node.attribute("version").map(VersionInfo::from_xml),
        };
        result
    }
}

impl StatusEntry {
    pub fn from_xml(node: Node) -> Self {
        let mut result = StatusEntry {
            annotation: vec![],
            status: Status(node.attribute("status").unwrap().into()),
            date: dt_builtins::Date(node.attribute("date").unwrap().into()),
            bugzilla: node
                .attribute("bugzilla")
                .map(|x| BugUri(dt_builtins::AnyURI(x.into()))),
        };
        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "annotation" => result.annotation.push(Annotation::from_xml(child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}

impl SchemaDocumentRef {
    pub fn from_xml(node: Node) -> Self {
        let mut result = SchemaDocumentRef {
            annotation: vec![],
            role: node.attribute("role").map(|r| Role(r.into())),
            r#type: node
                .attribute((XLINK, "type"))
                .map(|t| TypeType(t.into()))
                .or_else(|| Some(TypeType("locator".into()))),
            href: node
                .attribute((XLINK, "href"))
                .map(|h| HrefType(dt_builtins::AnyURI(h.into()))),
        };
        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "annotation" => result.annotation.push(Annotation::from_xml(child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}

impl SchemaTest {
    pub fn from_xml(node: Node) -> Self {
        let mut result = SchemaTest {
            annotation: vec![],
            schema_document: vec![],
            expected: vec![],
            current: None,
            prior: vec![],
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            version: node.attribute("version").map(VersionInfo::from_xml),
        };
        #[derive(Copy, Clone, Debug)]
        enum State {
            Annotation,
            SchemaDocument,
            Expected,
            Current,
            Prior,
        }
        let mut state = State::Annotation;
        for child in node.children().filter(|n| n.is_element()) {
            match (state, child.tag_name().name()) {
                (State::Annotation, "annotation") => {
                    result.annotation.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::SchemaDocument, "schemaDocument") => {
                    state = State::SchemaDocument;
                    result
                        .schema_document
                        .push(SchemaDocumentRef::from_xml(child))
                }
                (State::SchemaDocument | State::Expected, "expected") => {
                    state = State::Expected;
                    result.expected.push(Expected::from_xml(child))
                }
                (State::SchemaDocument | State::Expected, "current") => {
                    state = State::Current;
                    result.current = Some(StatusEntry::from_xml(child))
                }
                (
                    State::SchemaDocument | State::Expected | State::Current | State::Prior,
                    "prior",
                ) => {
                    state = State::Prior;
                    result.prior.push(StatusEntry::from_xml(child))
                }
                _ => unimplemented!(
                    "unexpected tag: {} (in state {state:?})",
                    child.tag_name().name()
                ),
            }
        }
        result
    }
}

impl InstanceTest {
    pub fn from_xml(node: Node) -> Self {
        let mut annotation = vec![];
        let mut instance_document = None;
        let mut expected = vec![];
        let mut current = None;
        let mut prior = vec![];

        #[derive(Copy, Clone, Debug)]
        enum State {
            Annotation,
            InstanceDocument,
            Expected,
            Current,
            Prior,
        }
        let mut state = State::Annotation;
        for child in node.children().filter(|n| n.is_element()) {
            match (state, child.tag_name().name()) {
                (State::Annotation, "annotation") => annotation.push(Annotation::from_xml(child)),
                (State::Annotation, "instanceDocument") => {
                    state = State::InstanceDocument;
                    instance_document = Some(Ref::from_xml(child));
                }
                (State::InstanceDocument | State::Expected, "expected") => {
                    state = State::Expected;
                    expected.push(Expected::from_xml(child));
                }
                (State::InstanceDocument | State::Expected, "current") => {
                    state = State::Current;
                    current = Some(StatusEntry::from_xml(child))
                }
                (
                    State::InstanceDocument | State::Expected | State::Current | State::Prior,
                    "prior",
                ) => {
                    state = State::Prior;
                    prior.push(StatusEntry::from_xml(child));
                }
                _ => unimplemented!(
                    "unexpected tag: {} (in state {state:?})",
                    child.tag_name().name()
                ),
            }
        }

        let result = InstanceTest {
            annotation,
            instance_document: instance_document.unwrap(),
            expected,
            current,
            prior,
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            version: node.attribute("version").map(VersionInfo::from_xml),
        };
        result
    }
}

impl TestGroup {
    pub fn from_xml(node: Node) -> Self {
        let mut result = TestGroup {
            annotation: vec![],
            documentation_reference: vec![],
            schema_test: None,
            instance_test: vec![],
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            version: node.attribute("version").map(VersionInfo::from_xml),
        };
        #[derive(Copy, Clone, Debug)]
        enum State {
            Annotation,
            DocumentationReference,
            SchemaTest,
            InstanceTest,
        }
        let mut state = State::Annotation;
        for child in node.children().filter(|n| n.is_element()) {
            match (state, child.tag_name().name()) {
                (State::Annotation, "annotation") => {
                    result.annotation.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::DocumentationReference, "documentationReference") => {
                    state = State::DocumentationReference;
                    result.documentation_reference.push(Ref::from_xml(child))
                }
                (State::Annotation | State::DocumentationReference, "schemaTest") => {
                    state = State::SchemaTest;
                    result.schema_test = Some(SchemaTest::from_xml(child))
                }
                (
                    State::Annotation
                    | State::DocumentationReference
                    | State::SchemaTest
                    | State::InstanceTest,
                    "instanceTest",
                ) => {
                    state = State::InstanceTest;
                    result.instance_test.push(InstanceTest::from_xml(child))
                }
                _ => unimplemented!(
                    "unexpected tag: {} (in state {state:?})",
                    child.tag_name().name()
                ),
            }
        }
        result
    }
}

impl TestSet {
    pub fn from_xml(node: Node) -> Self {
        let mut result = TestSet {
            annotation: vec![],
            test_group: vec![],
            contributor: node.attribute("contributor").unwrap().into(),
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            version: node.attribute("version").map(VersionInfo::from_xml),
        };
        #[derive(Copy, Clone, Debug)]
        enum State {
            Annotation,
            TestGroup,
        }
        let mut state = State::Annotation;
        for child in node.children().filter(|n| n.is_element()) {
            match (state, child.tag_name().name()) {
                (State::Annotation, "annotation") => {
                    result.annotation.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::TestGroup, "testGroup") => {
                    state = State::TestGroup;
                    result.test_group.push(TestGroup::from_xml(child))
                }
                _ => unimplemented!(
                    "unexpected tag: {} (in state {state:?})",
                    child.tag_name().name()
                ),
            }
        }
        result
    }
}
