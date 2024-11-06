use crate::xstypes::QName;
use roxmltree::Node;

pub trait ActualValue<'a> {
    fn convert(src: &'a str, parent: Node) -> Self;
}

impl<'a> ActualValue<'a> for &'a str {
    fn convert(src: &'a str, _parent: Node) -> Self {
        src
    }
}

impl ActualValue<'_> for String {
    fn convert(src: &'_ str, _parent: Node) -> Self {
        src.to_string()
    }
}

impl ActualValue<'_> for QName {
    fn convert(src: &'_ str, parent: Node) -> Self {
        QName::parse(src, parent).unwrap()
    }
}

impl<'a, T: ActualValue<'a>> ActualValue<'a> for Vec<T> {
    fn convert(src: &'a str, _parent: Node) -> Self {
        // NOTE: This assumes a list with whiteSpace="collapse"
        // TODO: split_ascii_whitespace includes U+000C FORM FEED, which is not considered
        // whitespace by the spec.
        src.split_ascii_whitespace()
            .map(|a| ActualValue::convert(a, _parent))
            .collect()
    }
}

impl ActualValue<'_> for bool {
    fn convert(src: &str, _parent: Node) -> Self {
        match src {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => panic!("Invalid value for boolean: {src}"),
        }
    }
}

impl ActualValue<'_> for u64 {
    fn convert(src: &str, _parent: Node) -> Self {
        src.parse().unwrap()
    }
}

pub fn actual_value<'a, T: ActualValue<'a>>(x: &'a str, parent: Node) -> T {
    T::convert(x, parent)
}

pub fn normalized_value(x: &str) -> &str {
    x // TODO
}
