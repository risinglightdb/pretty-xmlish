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

    /// Potential optimizations: instead of using a mutable String,
    /// use a Write trait with monadic error reporting.
    fn ol_build_str(&self, builder: &mut String) {
        use Pretty::*;
        match self {
            Text(s) => builder.push_str(s),
            Record(name, m) => {
                builder.push_str(name);
                builder.push_str(" { ");
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        builder.push_str(", ");
                    }
                    builder.push_str(k);
                    builder.push_str(": ");
                    v.ol_build_str(builder);
                }
                builder.push_str(" }");
            }
            Array(v) => {
                builder.push_str("[ ");
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        builder.push_str(", ");
                    }
                    e.ol_build_str(builder);
                }
                builder.push_str(" ]");
            }
        }
    }

    fn ol_len(&self) -> usize {
        use Pretty::*;
        match self {
            Text(s) => s.len(),
            Record(name, m) => {
                let mem: usize = m
                    .iter()
                    .map(|(k, v)| k.len() + ": ".len() + v.ol_len())
                    .sum();
                let mid = (m.len() - 1) * ", ".len();
                let beg = " { ".len() + " }".len() + name.len();
                mem + mid + beg
            }
            Array(v) => {
                let mem: usize = v.iter().map(Self::ol_len).sum();
                let mid = (v.len() - 1) * ", ".len();
                let beg = "[ ".len() + " ]".len();
                mem + mid + beg
            }
        }
    }
}

impl<'a, T: Into<Str<'a>>> From<T> for Pretty<'a> {
    fn from(s: T) -> Self {
        Pretty::Text(s.into())
    }
}

#[derive(Clone)]
pub struct PrettyConfig {
    pub indent: usize,
    /// Preferred width of the output
    pub width: usize,
}

impl PrettyConfig {
    fn interesting(&self, base_indent: usize, pretty: &Pretty) -> usize {
        let len = pretty.ol_len() + base_indent;
        if len <= self.width {
            len
        } else {
            let new_indent = base_indent + self.indent;
            use Pretty::*;
            match pretty {
                Text(s) => s.len() + base_indent,
                Array(v) => v
                    .iter()
                    .map(|p| self.interesting(new_indent, p) + ",".len())
                    .max()
                    .unwrap_or(base_indent + "[".len()),
                Record(name, m) => {
                    let header = name.len() + base_indent + " {".len();
                    m.iter()
                        .map(|(k, v)| k.len() + ": ".len() + self.interesting(new_indent, v))
                        .chain(vec![header].into_iter())
                        .max()
                        .unwrap()
                }
            }
        }
    }
}

impl Default for PrettyConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            width: 120,
        }
    }
}

#[test]
fn main() {
    println!("Hello, world!");
}
