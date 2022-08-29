mod xsd;

fn main() {
    let xsd = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let xsd = roxmltree::Document::parse(&xsd).unwrap();
    let (schema, components) = xsd::read_schema(xsd);
    dbg!(&schema, &components);
}
