use super::generated::*;
use dt_builtins::meta::SimpleType as _;
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
            annotations: vec![],
            r#type: node
                .attribute((XLINK, "type"))
                .map(|t| TypeType::from_string(t).unwrap())
                .or(Some(TypeType::Locator)),
            // TODO: optional with default value should not generate Option<...>
            href: node
                .attribute((XLINK, "href"))
                .map(|h| HrefType::from_string(h).unwrap()),
        };
        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "annotation" => result.annotations.push(Annotation::from_xml(child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}

impl TestSuite {
    pub fn from_xml(node: Node) -> Self {
        let mut result = TestSuite {
            annotations: vec![],
            test_set_refs: vec![],
            name: dt_builtins::Name::from_string(node.attribute("name").unwrap()).unwrap(),
            release_date: dt_builtins::Date::from_string(node.attribute("releaseDate").unwrap())
                .unwrap(),
            schema_version: node.attribute("schemaVersion").unwrap().into(),
            version: node
                .attribute("version")
                .map(VersionInfo::from_string)
                .transpose()
                .unwrap(),
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
                    result.annotations.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::TestSetRef, "testSetRef") => {
                    state = State::TestSetRef;
                    result.test_set_refs.push(Ref::from_xml(child))
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

impl Expected {
    pub fn from_xml(node: Node) -> Self {
        let result = Expected {
            validity: ExpectedOutcome::from_string(node.attribute("validity").unwrap()).unwrap(),
            version: node
                .attribute("version")
                .map(VersionInfo::from_string)
                .transpose()
                .unwrap(),
        };
        result
    }
}

impl StatusEntry {
    pub fn from_xml(node: Node) -> Self {
        let mut result = StatusEntry {
            annotations: vec![],
            status: Status::from_string(node.attribute("status").unwrap()).unwrap(),
            date: dt_builtins::Date::from_string(node.attribute("date").unwrap()).unwrap(),
            bugzilla: node
                .attribute("bugzilla")
                .map(|x| BugUri::from_string(x).unwrap()),
        };
        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "annotation" => result.annotations.push(Annotation::from_xml(child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}

impl SchemaDocumentRef {
    pub fn from_xml(node: Node) -> Self {
        let mut result = SchemaDocumentRef {
            annotations: vec![],
            role: node
                .attribute("role")
                .map(|r| Role::from_string(r).unwrap()),
            r#type: node
                .attribute((XLINK, "type"))
                .map(|t| TypeType::from_string(t).unwrap())
                .or(Some(TypeType::Locator)),
            href: node
                .attribute((XLINK, "href"))
                .map(|h| HrefType::from_string(h).unwrap()),
        };
        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "annotation" => result.annotations.push(Annotation::from_xml(child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}

impl SchemaTest {
    pub fn from_xml(node: Node) -> Self {
        let mut result = SchemaTest {
            annotations: vec![],
            schema_documents: vec![],
            expecteds: vec![],
            current: None,
            priors: vec![],
            name: dt_builtins::Name::from_string(node.attribute("name").unwrap()).unwrap(),
            version: node
                .attribute("version")
                .map(VersionInfo::from_string)
                .transpose()
                .unwrap(),
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
                    result.annotations.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::SchemaDocument, "schemaDocument") => {
                    state = State::SchemaDocument;
                    result
                        .schema_documents
                        .push(SchemaDocumentRef::from_xml(child))
                }
                (State::SchemaDocument | State::Expected, "expected") => {
                    state = State::Expected;
                    result.expecteds.push(Expected::from_xml(child))
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
                    result.priors.push(StatusEntry::from_xml(child))
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
        let mut annotations = vec![];
        let mut instance_document = None;
        let mut expecteds = vec![];
        let mut current = None;
        let mut priors = vec![];

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
                (State::Annotation, "annotation") => annotations.push(Annotation::from_xml(child)),
                (State::Annotation, "instanceDocument") => {
                    state = State::InstanceDocument;
                    instance_document = Some(Ref::from_xml(child));
                }
                (State::InstanceDocument | State::Expected, "expected") => {
                    state = State::Expected;
                    expecteds.push(Expected::from_xml(child));
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
                    priors.push(StatusEntry::from_xml(child));
                }
                _ => unimplemented!(
                    "unexpected tag: {} (in state {state:?})",
                    child.tag_name().name()
                ),
            }
        }

        let result = InstanceTest {
            annotations,
            instance_document: instance_document.unwrap(),
            expecteds,
            current,
            priors,
            name: dt_builtins::Name::from_string(node.attribute("name").unwrap()).unwrap(),
            version: node
                .attribute("version")
                .map(VersionInfo::from_string)
                .transpose()
                .unwrap(),
        };
        result
    }
}

impl TestGroup {
    pub fn from_xml(node: Node) -> Self {
        let mut result = TestGroup {
            annotations: vec![],
            documentation_references: vec![],
            schema_test: None,
            instance_tests: vec![],
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            version: node
                .attribute("version")
                .map(VersionInfo::from_string)
                .transpose()
                .unwrap(),
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
                    result.annotations.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::DocumentationReference, "documentationReference") => {
                    state = State::DocumentationReference;
                    result.documentation_references.push(Ref::from_xml(child))
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
                    result.instance_tests.push(InstanceTest::from_xml(child))
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
            annotations: vec![],
            test_groups: vec![],
            contributor: node.attribute("contributor").unwrap().into(),
            name: dt_builtins::Name::from_string(node.attribute("name").unwrap()).unwrap(),
            version: node
                .attribute("version")
                .map(VersionInfo::from_string)
                .transpose()
                .unwrap(),
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
                    result.annotations.push(Annotation::from_xml(child))
                }
                (State::Annotation | State::TestGroup, "testGroup") => {
                    state = State::TestGroup;
                    result.test_groups.push(TestGroup::from_xml(child))
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
