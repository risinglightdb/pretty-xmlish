use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{Debug, Display},
    iter::repeat,
};

type Str<'a> = Cow<'a, str>;
/// This is recommended by `@rami3l` to supercede `Vec<Pretty>`.
/// Why not use this? Because Rust wouldn't let me!
/// https://github.com/rust-lang/rust/issues/23714
#[allow(dead_code)]
type Pretties<'a> = Cow<'a, [Pretty<'a>]>;

pub mod ascii;
pub use ascii::*;
pub mod unicode;
pub use unicode::*;

#[derive(Clone)]
pub struct XmlNode<'a> {
    pub name: Str<'a>,
    pub fields: BTreeMap<Str<'a>, Pretty<'a>>,
    pub children: Vec<Pretty<'a>>,
}

/// Use `into`!!
#[derive(Clone)]
pub enum Pretty<'a> {
    Text(Str<'a>),
    Record(XmlNode<'a>),
    Array(Vec<Self>),
}

impl<'a> Pretty<'a> {
    pub fn simple_record(
        name: &'a str,
        fields: BTreeMap<&'a str, Self>,
        children: Vec<Self>,
    ) -> Self {
        let name = name.into();
        let fields = fields.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Self::Record(XmlNode {
            name,
            fields,
            children,
        })
    }
    pub fn list_of_strings(list: &'a [&'a str]) -> Self {
        Self::Array(list.iter().map(|&s| s.into()).collect())
    }

    pub fn display(display: &impl Display) -> Self {
        display.to_string().into()
    }

    pub fn debug(debug: &impl Debug) -> Self {
        format!("{:?}", debug).into()
    }

    pub fn has_children(&self) -> bool {
        match self {
            Pretty::Record(xml) => !xml.children.is_empty(),
            _ => false,
        }
    }

    /// Potential improvements: instead of using a mutable String,
    /// use a Write trait with monadic error reporting.
    pub(crate) fn ol_build_str_ascii(&self, builder: &mut String) {
        use Pretty::*;
        match self {
            Text(s) => builder.push_str(s),
            Record(xml) => {
                builder.push_str(&xml.name);
                builder.push_str(" { ");
                for (i, (k, v)) in xml.fields.iter().enumerate() {
                    if i > 0 {
                        builder.push_str(", ");
                    }
                    builder.push_str(k);
                    builder.push_str(": ");
                    v.ol_build_str_ascii(builder);
                }
                builder.push_str(" }");
            }
            Array(v) => {
                if v.is_empty() {
                    builder.push_str("[]");
                    return;
                }
                builder.push_str("[ ");
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        builder.push_str(", ");
                    }
                    e.ol_build_str_ascii(builder);
                }
                builder.push_str(" ]");
            }
        }
    }

    #[allow(dead_code)]
    /// For debugging purposes.
    fn ol_to_string(&self) -> String {
        let mut builder = String::with_capacity(self.ol_len());
        self.ol_build_str_ascii(&mut builder);
        builder
    }

    pub(crate) fn ol_len(&self) -> usize {
        use Pretty::*;
        match self {
            Text(s) => s.chars().count(),
            Record(xml) => {
                let mem: usize = (xml.fields.iter())
                    .map(|(k, v)| k.chars().count() + ": ".len() + v.ol_len())
                    .sum();
                let mid = (xml.fields.len() - 1) * ", ".len();
                let beg = " { ".len() + " }".len() + xml.name.chars().count();
                mem + mid + beg
            }
            Array(v) => {
                if v.is_empty() {
                    return "[]".len();
                }
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
    /// Preferred width of the output, exlusive of the boundaries.
    pub width: usize,
}

impl PrettyConfig {
    pub fn horizon(out: &mut String, total_len: usize) {
        out.push_str("+");
        out.extend(repeat("-").take(total_len - 2));
        out.push_str("+");
    }
}

struct LinedBuffer<'a> {
    width: usize,
    /// Modify when out is also modified.
    pub already_occupied: usize,
    out: &'a mut String,
    config: &'a PrettyConfig,
}
impl<'a> LinedBuffer<'a> {
    fn begin_line(&mut self) {
        self.out.push_str("| ");
    }
    fn push(&mut self, s: &str) {
        self.out.push_str(s);
        self.already_occupied += s.chars().count();
    }
    fn pip(&mut self, amount: usize) {
        self.push(" ".repeat(amount).as_str());
    }
    fn pusheen(&mut self) {
        if self.width < self.already_occupied {
            println!(
                "width: {}, already_occupied: {}",
                self.width, self.already_occupied
            );
            panic!("already_occupied > width");
        }
        self.pip(self.width - self.already_occupied);
        self.push(" |\n");
        self.already_occupied = 0;
    }
}

impl Default for PrettyConfig {
    fn default() -> Self {
        Self {
            indent: 4,
            width: 120,
        }
    }
}
