pub type NCName = String;
pub type AnyURI = String;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QName(pub AnyURI, pub NCName);

pub type Sequence<T> = Vec<T>;
pub type Set<T> = Vec<T>; //std::collections::HashSet<T>;
