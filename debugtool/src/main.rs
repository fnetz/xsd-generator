use std::fmt;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use dt_xsd::ComplexTypeDefinition;
use dt_xsd::ConstrainingFacet;
use dt_xsd::ModelGroup;
use dt_xsd::Particle;
use dt_xsd::Ref;
use dt_xsd::RefNamed;
use dt_xsd::SchemaComponentTable;
use dt_xsd::SimpleTypeDefinition;
use dt_xsd::Term;
use dt_xsd::TypeDefinition;
use dt_xsd::complex_type_def::ContentTypeVariety;
use dt_xsd::complex_type_def::OpenContentMode;
use dt_xsd::model_group::Compositor;
use dt_xsd::particle::MaxOccurs;
use dt_xsd::shared::ScopeVariety;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum BuiltinOverwriteAction {
    Deny,
    Warn,
    Allow,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum RegisterBuiltins {
    Yes,
    No,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to read
    input: String,

    #[command(subcommand)]
    command: Commands,

    /// Allow a XML Document Type Definition (DTD) to occur
    #[clap(long)]
    pub allow_dtd: bool,

    /// Allow automatic downloading of imports over HTTP
    // #[clap(long)]
    // pub allow_http_imports: bool,

    /// The action to take when trying to overwrite a built-in type
    #[clap(long, default_value = "deny", value_enum)]
    pub builtin_overwrite: BuiltinOverwriteAction,

    /// Whether to register the builtin types and attributes
    #[clap(long, default_value = "yes", value_enum)]
    pub register_builtins: RegisterBuiltins,

    #[clap(long)]
    show_empty: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show all elements in the file
    ShowAll,

    /// Show a specific type
    ShowType { id: u32 },

    /// Show a specific element
    ShowElement,

    /// Show attribute
    ShowAttribute,
}

fn show_vec<T: fmt::Debug>(vec: &[T], name: &str, indent: usize, options: &Cli) {
    if !vec.is_empty() || options.show_empty {
        print_indent(indent);
        println!("{{{name}}}: {vec:?}");
    }
}

fn show_option<T: fmt::Debug>(option: &Option<T>, name: &str, indent: usize, options: &Cli) {
    if option.is_some() || options.show_empty {
        print_indent(indent);
        print!("{{{name}}}: ");
        if let Some(value) = option {
            println!("{:?}", value);
        } else {
            println!("nil");
        }
    }
}

fn print_indent(indent: usize) {
    print!("{:indent$}", "", indent = indent * 2);
}

fn print_attribute_declaration(
    attribute_declaration: Ref<dt_xsd::AttributeDeclaration>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{attribute_declaration:?}");

    let attribute_declaration = attribute_declaration.get(components);

    show_vec(
        &attribute_declaration.annotations,
        "annotations",
        indent,
        options,
    );

    print_indent(indent);
    println!("{{name}}: {:?}", attribute_declaration.name);

    show_option(
        &attribute_declaration.target_namespace,
        "target namespace",
        indent,
        options,
    );

    print_indent(indent);
    println!(
        "{{type definition}}: {:?}",
        attribute_declaration.type_definition
    );

    // scope
    print_indent(indent);
    println!("{{scope}}: {:?}", attribute_declaration.scope);

    // value constraint
    show_option(
        &attribute_declaration.value_constraint,
        "value constraint",
        indent,
        options,
    );

    // inheritable
    print_indent(indent);
    println!("{{inheritable}}: {:?}", attribute_declaration.inheritable);
}

fn print_attribute_use(
    attribute_use: Ref<dt_xsd::AttributeUse>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{attribute_use:?}");

    let attribute_use = attribute_use.get(components);

    show_vec(&attribute_use.annotations, "annotations", indent, options);

    print_indent(indent);
    println!("{{required}}: {:?}", attribute_use.required);

    print_indent(indent);
    print!("{{attribute declaration}}: ");
    if attribute_use
        .attribute_declaration
        .get(components)
        .scope
        .variety()
        == ScopeVariety::Local
    {
        print_attribute_declaration(
            attribute_use.attribute_declaration,
            components,
            indent + 1,
            options,
        );
    } else {
        println!("{:?}", attribute_use.attribute_declaration);
    }

    show_option(
        &attribute_use.value_constraint,
        "value constraint",
        indent,
        options,
    );

    print_indent(indent);
    println!("{{inheritable}}: {:?}", attribute_use.inheritable);
}

fn print_model_group(
    model_group: Ref<ModelGroup>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{model_group:?}");

    let model_group = model_group.get(components);
    // TODO: global/local?

    show_vec(&model_group.annotations, "annotations", indent, options);

    print_indent(indent);
    println!(
        "{{compositor}}: {}",
        match model_group.compositor {
            Compositor::All => "all",
            Compositor::Choice => "choice",
            Compositor::Sequence => "sequence",
        }
    );

    print_indent(indent);
    println!("{{particles}}: [");

    for particle in &model_group.particles {
        print_indent(indent + 1);
        print_particle(*particle, components, indent + 2, options);
    }

    print_indent(indent);
    println!("]");
}

fn print_term(term: Term, components: &SchemaComponentTable, indent: usize, options: &Cli) {
    match term {
        Term::ElementDeclaration(element) => {
            if element.get(components).scope.variety() == ScopeVariety::Local {
                print_element_declaration(element, components, indent, options);
            } else {
                println!("{element:?}");
            }
        }
        Term::ModelGroup(model_group) => {
            print_model_group(model_group, components, indent, options);
        }
        Term::Wildcard(wildcard) => {
            // print_wildcard(wildcard, components, indent, options);
            println!("{wildcard:?}");
        }
    }
}

fn print_particle(
    particle: Ref<Particle>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{particle:?}");
    let particle = particle.get(components);

    print_indent(indent);
    println!("{{min occurs}}: {:?}", particle.min_occurs);

    print_indent(indent);
    print!("{{max occurs}}: ");
    match particle.max_occurs {
        MaxOccurs::Unbounded => println!("unbounded"),
        MaxOccurs::Count(count) => println!("{count}"),
    }

    print_indent(indent);
    print!("{{term}}: ");
    print_term(particle.term, components, indent + 1, options);

    show_vec(
        particle.annotations(components),
        "annotations",
        indent,
        options,
    );
}

fn print_constraining_facet(
    facet_ref: Ref<ConstrainingFacet>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    let facet = facet_ref.get(components);

    println!("{facet_ref:?} ({})", facet.name());

    match facet {
        ConstrainingFacet::Length(length)
        | ConstrainingFacet::MinLength(length)
        | ConstrainingFacet::MaxLength(length) => {
            show_vec(&length.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", length);

            print_indent(indent);
            println!("{{fixed}}: {:?}", length.fixed);
        }
        ConstrainingFacet::Pattern(pattern) => {
            show_vec(&pattern.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", pattern.value);
        }
        ConstrainingFacet::Enumeration(enumeration) => {
            show_vec(&enumeration.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", enumeration.value);
        }
        ConstrainingFacet::WhiteSpace(white_space) => {
            show_vec(&white_space.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", white_space.value);

            print_indent(indent);
            println!("{{fixed}}: {:?}", white_space.fixed);
        }
        ConstrainingFacet::MaxInclusive(min_max)
        | ConstrainingFacet::MaxExclusive(min_max)
        | ConstrainingFacet::MinExclusive(min_max)
        | ConstrainingFacet::MinInclusive(min_max) => {
            show_vec(&min_max.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", min_max.value);

            print_indent(indent);
            println!("{{fixed}}: {:?}", min_max.fixed);
        }
        ConstrainingFacet::TotalDigits(total_digits) => {
            show_vec(&total_digits.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", total_digits.value);

            print_indent(indent);
            println!("{{fixed}}: {:?}", total_digits.fixed);
        }
        ConstrainingFacet::FractionDigits(fraction_digits) => {
            show_vec(&fraction_digits.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", fraction_digits.value);

            print_indent(indent);
            println!("{{fixed}}: {:?}", fraction_digits.fixed);
        }
        ConstrainingFacet::Assertions(assertions) => {
            show_vec(&assertions.annotations, "annotations", indent, options);

            print_indent(indent);
            println!("{{value}}: {:?}", assertions.value);
        }
        ConstrainingFacet::ExplicitTimezone(explicit_timezone) => {
            show_vec(
                &explicit_timezone.annotations,
                "annotations",
                indent,
                options,
            );

            print_indent(indent);
            println!("{{value}}: {:?}", explicit_timezone.value);

            print_indent(indent);
            println!("{{fixed}}: {:?}", explicit_timezone.fixed);
        }
    }
}

fn print_simple_type_definition(
    simple_type: Ref<SimpleTypeDefinition>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{simple_type:?}");

    let simple_type = simple_type.get(components);

    show_vec(&simple_type.annotations, "annotations", indent, options);

    show_option(&simple_type.name, "name", indent, options);

    show_option(
        &simple_type.target_namespace,
        "target namespace",
        indent,
        options,
    );

    show_vec(&simple_type.final_, "final", indent, options);

    show_option(&simple_type.context, "context", indent, options);

    print_indent(indent);
    println!(
        "{{base type definition}}: {:?}",
        simple_type.base_type_definition
    );

    if simple_type.facets.iter().count() > 0 || options.show_empty {
        print_indent(indent);
        println!("{{facets}}: [");
        for facet in simple_type.facets.iter().copied() {
            print_indent(indent + 1);
            print_constraining_facet(facet, components, indent + 2, options);
        }
        print_indent(indent);
        println!("]");
    }

    if !simple_type.fundamental_facets.is_empty() || options.show_empty {
        print_indent(indent);
        println!("{{fundamental facets}}: [");

        if let Some(ordered) = simple_type.fundamental_facets.ordered() {
            print_indent(indent + 1);
            println!("ordered: {ordered:?}");
        } else if options.show_empty {
            print_indent(indent + 1);
            println!("ordered: nil");
        }

        if let Some(bounded) = simple_type.fundamental_facets.bounded() {
            print_indent(indent + 1);
            println!("bounded: {bounded:?}");
        } else if options.show_empty {
            print_indent(indent + 1);
            println!("bounded: nil");
        }

        if let Some(cardinality) = simple_type.fundamental_facets.cardinality() {
            print_indent(indent + 1);
            println!("cardinality: {cardinality:?}");
        } else if options.show_empty {
            print_indent(indent + 1);
            println!("cardinality: nil");
        }

        if let Some(numeric) = simple_type.fundamental_facets.numeric() {
            print_indent(indent + 1);
            println!("numeric: {numeric:?}");
        } else if options.show_empty {
            print_indent(indent + 1);
            println!("numeric: nil");
        }

        print_indent(indent);
        println!("]");
    }

    show_option(&simple_type.variety, "variety", indent, options);

    show_option(
        &simple_type.primitive_type_definition,
        "primitive type definition",
        indent,
        options,
    );

    show_option(
        &simple_type.item_type_definition,
        "item type definition",
        indent,
        options,
    );

    show_option(
        &simple_type.member_type_definitions,
        "member type definitions",
        indent,
        options,
    );
}

fn print_complex_type_definition(
    complex_type: Ref<ComplexTypeDefinition>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{complex_type:?}");

    let complex_type = complex_type.get(components);

    show_vec(&complex_type.annotations, "annotations", indent, options);

    show_option(&complex_type.name, "name", indent, options);

    show_option(
        &complex_type.target_namespace,
        "target namespace",
        indent,
        options,
    );

    print_indent(indent);
    println!(
        "{{base type definition}}: {:?}",
        complex_type.base_type_definition
    );

    show_vec(&complex_type.final_, "final", indent, options);

    show_option(&complex_type.context, "context", indent, options);

    show_option(
        &complex_type.derivation_method,
        "derivation method",
        indent,
        options,
    );

    print_indent(indent);
    println!("{{abstract}}: {:?}", complex_type.abstract_);

    if !complex_type.attribute_uses.is_empty() || options.show_empty {
        print_indent(indent);
        println!("{{attribute uses}}: [");
        for attribute_use in &complex_type.attribute_uses {
            print_indent(indent + 1);
            print_attribute_use(*attribute_use, components, indent + 2, options);
        }
        print_indent(indent);
        println!("]");
    }

    show_option(
        &complex_type.attribute_wildcard,
        "attribute wildcard",
        indent,
        options,
    );

    print_indent(indent);
    println!("{{content type}}:");

    {
        let indent = indent + 1;

        print_indent(indent);
        println!(
            "{{variety}}: {}",
            match complex_type.content_type.variety() {
                ContentTypeVariety::Empty => "empty",
                ContentTypeVariety::Simple => "simple",
                ContentTypeVariety::ElementOnly => "element-only",
                ContentTypeVariety::Mixed => "mixed",
            }
        );

        if complex_type.content_type.particle().is_some() || options.show_empty {
            print_indent(indent);
            print!("{{particle}}: ");
            if let Some(particle) = complex_type.content_type.particle() {
                print_particle(particle, components, indent + 1, options);
            } else {
                println!("nil");
            }
        }

        let open_content = complex_type.content_type.open_content();
        if open_content.is_some() || options.show_empty {
            print_indent(indent);
            print!("{{open content}}:");
            if let Some(open_content) = open_content {
                println!();
                let indent = indent + 1;
                print_indent(indent);
                println!(
                    "{{mode}}: {:?}",
                    match open_content.mode {
                        OpenContentMode::Interleave => "interleave",
                        OpenContentMode::Suffix => "suffix",
                    }
                );

                print_indent(indent);
                println!("{{wildcard}}: {:?}", open_content.wildcard);
            } else {
                print_indent(indent);
                println!(" nil");
            }
        }

        show_option(
            &complex_type.content_type.simple_type_definition(),
            "simple type definition",
            indent,
            options,
        );
    }

    show_vec(
        &complex_type.prohibited_substitutions,
        "prohibited substitutions",
        indent,
        options,
    );

    show_vec(&complex_type.assertions, "assertions", indent, options);
}

fn print_element_declaration(
    element: Ref<dt_xsd::ElementDeclaration>,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    println!("{element:?}");

    let element = element.get(components);

    show_vec(&element.annotations, "annotations", indent, options);

    print_indent(indent);
    println!("{{name}}: {:?}", element.name);

    show_option(
        &element.target_namespace,
        "target namespace",
        indent,
        options,
    );

    print_indent(indent);
    print!("{{type definition}}: ");
    if let Some(name) = element.type_definition.name(components) {
        println!("{:?} ({})", element.type_definition, name);
    } else {
        print_type_definition(element.type_definition, components, indent + 1, options);
    }

    show_option(&element.type_table, "type table", indent, options);

    print_indent(indent);
    println!("{{scope}}: {:?}", element.scope);

    show_option(
        &element.value_constraint,
        "value constraint",
        indent,
        options,
    );

    print_indent(indent);
    println!("{{nillable}}: {:?}", element.nillable);

    show_vec(
        &element.identity_constraint_definitions,
        "identity-constraint definitions",
        indent,
        options,
    );

    show_vec(
        &element.substitution_group_affiliations,
        "substitution group affiliations",
        indent,
        options,
    );

    show_vec(
        &element.substitution_group_exclusions,
        "substitution group exclusions",
        indent,
        options,
    );

    show_vec(
        &element.disallowed_substitutions,
        "disallowed substitutions",
        indent,
        options,
    );

    print_indent(indent);
    println!("{{abstract}}: {:?}", element.abstract_);
}

fn print_type_definition(
    type_definition: TypeDefinition,
    components: &SchemaComponentTable,
    indent: usize,
    options: &Cli,
) {
    match type_definition {
        TypeDefinition::Simple(simple_type) => {
            print_simple_type_definition(simple_type, components, indent, options);
        }
        TypeDefinition::Complex(complex_type) => {
            print_complex_type_definition(complex_type, components, indent, options);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let xsd = std::fs::read_to_string(&cli.input).unwrap();
    let options = roxmltree::ParsingOptions {
        allow_dtd: cli.allow_dtd,
        ..Default::default()
    };
    let xsd = roxmltree::Document::parse_with_options(&xsd, options).unwrap();
    let (schema, components) = dt_xsd::read_schema(
        xsd,
        match cli.builtin_overwrite {
            BuiltinOverwriteAction::Deny => dt_xsd::BuiltinOverwriteAction::Deny,
            BuiltinOverwriteAction::Warn => dt_xsd::BuiltinOverwriteAction::Warn,
            BuiltinOverwriteAction::Allow => dt_xsd::BuiltinOverwriteAction::Allow,
        },
        match cli.register_builtins {
            RegisterBuiltins::Yes => dt_xsd::RegisterBuiltins::Yes,
            RegisterBuiltins::No => dt_xsd::RegisterBuiltins::No,
        },
        &[],
    )
    .unwrap();

    match cli.command {
        Commands::ShowAll => {
            for type_def in schema.type_definitions {
                print_type_definition(type_def, &components, 1, &cli);
            }
            for element in schema.element_declarations {
                print_element_declaration(element, &components, 1, &cli);
            }
            for attribute in schema.attribute_declarations {
                print_attribute_declaration(attribute, &components, 1, &cli);
            }
        }
        Commands::ShowType { id } => {
            // TODO: complex/simple
            let type_def = schema
                .type_definitions
                .iter()
                .find(|type_def| match type_def {
                    TypeDefinition::Simple(simple_type) => simple_type.inner().get() == id,
                    TypeDefinition::Complex(complex_type) => complex_type.inner().get() == id,
                })
                .unwrap();
            print_type_definition(*type_def, &components, 1, &cli);
        }
        _ => {
            println!("Not implemented");
        }
    }
}
