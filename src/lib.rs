use std::collections::{BTreeMap};

#[derive(Clone)]
pub enum Pretty {
    String(String),
    Record(BTreeMap<String, Pretty>),
    Array(Vec<Pretty>),
}

#[test]
fn main() {
    println!("Hello, world!");
}
