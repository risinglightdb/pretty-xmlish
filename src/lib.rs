use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{Debug, Display},
};

type Str<'a> = Cow<'a, str>;

#[derive(Clone)]
pub enum Pretty<'a> {
    Text(Str<'a>),
    Record(Str<'a>, BTreeMap<Str<'a>, Self>),
    Array(Vec<Self>),
}

impl<'a> Pretty<'a> {
    pub fn display(display: &impl Display) -> Self {
        display.to_string().into()
    }

    pub fn debug(debug: &impl Debug) -> Self {
        format!("{:?}", debug).into()
    }

    fn ol_len(&self) -> usize {
        use Pretty::*;
        match self {
            Text(s) => s.len(),
            Record(name, m) => {
                // 2 is for the ": " separator
                let mem: usize = m.iter().map(|(k, v)| k.len() + 2 + v.ol_len()).sum();
                let mid = (m.len() - 1) * 2; // 2 is for the ", " separator
                let beg = 3 + 2 + name.len(); // " { " and " }"
                mem + mid + beg
            }
            Array(v) => {
                let mem: usize = v.iter().map(Self::ol_len).sum();
                let mid = (v.len() - 1) * 2; // 2 is for the ", " separator
                let beg = 2 + 2; // "[ " and " ]"
                mem + mid + beg
            }
        }
    }
}

impl<'a> From<String> for Pretty<'a> {
    fn from(s: String) -> Self {
        Pretty::Text(Cow::Owned(s))
    }
}

impl<'a> From<&'a str> for Pretty<'a> {
    fn from(s: &'a str) -> Self {
        Pretty::Text(Cow::Borrowed(s))
    }
}

#[test]
fn main() {
    println!("Hello, world!");
}
