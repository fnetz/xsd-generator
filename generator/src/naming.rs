pub fn convert<F: NamingConvention, T: NamingConvention>(source: &str) -> String {
    T::concatenate(&F::split(source))
}

pub trait NamingConvention {
    fn split(name: &str) -> Vec<&str>;
    fn concatenate(words: &[&str]) -> String;
}

pub struct SnakeCase;

impl NamingConvention for SnakeCase {
    fn split(name: &str) -> Vec<&str> {
        name.split('_').collect()
    }

    fn concatenate(words: &[&str]) -> String {
        words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("_")
    }
}

pub struct CamelCase;

impl NamingConvention for CamelCase {
    fn split(name: &str) -> Vec<&str> {
        if name.is_empty() {
            vec![]
        } else if name.len() == 1 {
            vec![name]
        } else {
            let mut remaining = name;
            let mut words = vec![];
            while let Some(upper) = remaining[1..].find(|c: char| c.is_uppercase()) {
                let upper = upper + 1;
                words.push(&remaining[..upper]);
                remaining = &remaining[upper..];
                if remaining.len() < 2 {
                    break;
                }
            }
            if !remaining.is_empty() {
                words.push(remaining);
            }
            words
        }
    }

    fn concatenate(words: &[&str]) -> String {
        if words.is_empty() {
            String::new()
        } else {
            let mut result = String::new();
            result.push_str(&words[0].to_lowercase());
            for word in &words[1..] {
                let mut chars = word.chars();
                chars
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .for_each(|c| result.push(c));
                chars
                    .flat_map(|c| c.to_lowercase())
                    .for_each(|c| result.push(c));
            }
            result
        }
    }
}

pub struct PascalCase;

impl NamingConvention for PascalCase {
    fn split(name: &str) -> Vec<&str> {
        // For now, the splitting logic is the same as camelCase.
        CamelCase::split(name)
    }

    fn concatenate(words: &[&str]) -> String {
        let mut result = String::new();
        for word in words {
            let mut chars = word.chars();
            chars
                .next()
                .unwrap()
                .to_uppercase()
                .for_each(|c| result.push(c));
            chars
                .flat_map(|c| c.to_lowercase())
                .for_each(|c| result.push(c));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_splits_correctly() {
        let source = "helloWorld";
        let words = CamelCase::split(source);
        assert_eq!(words, vec!["hello", "World"]);
    }

    #[test]
    fn camel_case_split_noop() {
        let source = "hello";
        let words = CamelCase::split(source);
        assert_eq!(words, vec!["hello"]);
    }

    #[test]
    fn camel_case_split_single_letter() {
        let source = "i";
        let words = CamelCase::split(source);
        assert_eq!(words, vec!["i"]);

        let source = "inI";
        let words = CamelCase::split(source);
        assert_eq!(words, vec!["in", "I"]);
    }

    #[test]
    fn camel_case_concatenates_correctly() {
        let name = CamelCase::concatenate(&["HElLo", "wOrLd"]);
        assert_eq!(name, "helloWorld")
    }

    #[test]
    fn snake_case_roundtrip() {
        let source = "hello_world";
        let name = SnakeCase::split(source);
        assert_eq!(name, vec!["hello", "world"]);
        let concat = SnakeCase::concatenate(&name);
        assert_eq!(concat, source);
    }

    #[test]
    fn pascal_case_concat_works() {
        let name = PascalCase::concatenate(&["HElLo", "wOrLd"]);
        assert_eq!(name, "HelloWorld")
    }
}
