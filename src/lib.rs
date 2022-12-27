use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{Debug, Display},
};

type Str<'a> = Cow<'a, str>;

pub mod ascii;
pub use ascii::*;
pub mod unicode;
pub use unicode::*;

/// Use `into`!!
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

        /// Potential improvements: instead of using a mutable String,
    /// use a Write trait with monadic error reporting.
    pub(crate) fn ol_build_str_ascii(&self, builder: &mut String) {
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
                    v.ol_build_str_ascii(builder);
                }
                builder.push_str(" }");
            }
            Array(v) => {
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
            Record(name, m) => {
                let mem: usize = m
                    .iter()
                    .map(|(k, v)| k.chars().count() + ": ".len() + v.ol_len())
                    .sum();
                let mid = (m.len() - 1) * ", ".len();
                let beg = " { ".len() + " }".len() + name.chars().count();
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
    /// Preferred width of the output, exlusive of the boundaries.
    pub width: usize,
}

impl PrettyConfig {
    pub fn horizon(out: &mut String, total_len: usize) {
        out.push_str("+");
        out.push_str("-".repeat(total_len - 2).as_str());
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
        self.already_occupied += s.len();
    }
    fn pip(&mut self, amount: usize) {
        self.push(" ".repeat(amount).as_str());
    }
    fn pusheen(&mut self) {
        self.pip(self.width - self.already_occupied);
        self.push(" |\n");
        self.already_occupied = 0;
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
