use super::generated::*;
use roxmltree::Node;

impl Annotation {
    pub fn from_xml(_node: &Node) -> Self {
        todo!()
    }
}

impl Ref {
    pub fn from_xml(_node: &Node) -> Self {
        todo!()
    }
}

impl TestSuite {
    pub fn from_xml(node: &Node) -> Self {
        let mut result = TestSuite {
            annotation: vec![],
            test_set_ref: vec![],
            name: dt_builtins::Name(node.attribute("name").unwrap().into()),
            release_date: dt_builtins::Date(node.attribute("releaseDate").unwrap().into()),
            schema_version: node.attribute("schemaVersion").unwrap().into(),
            version: node.attribute("version").map(|v| VersionInfo(v.into())),
        };
        for child in node.children() {
            match child.tag_name().name() {
                "annotation" => result.annotation.push(Annotation::from_xml(&child)),
                "testSetRef" => result.test_set_ref.push(Ref::from_xml(&child)),
                _ => unimplemented!("unexpected tag: {}", child.tag_name().name()),
            }
        }
        result
    }
}
