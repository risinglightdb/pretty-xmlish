//! How to use this library? Very easy!
//!
//! ```rust
//! use pretty_xmlish::{Pretty, PrettyConfig};
//! use std::collections::BTreeMap;
//! // This class controls the expected width, indent size, and more.
//! let mut config = PrettyConfig::default();
//! // Other factory methods are available
//! let pretty = Pretty::simple_record("BatchNestedLoopJoin",
//!     BTreeMap::new(), // fields, if any
//!     vec![] // children, if any
//! );
//! let mut out = String::with_capacity(114514);
//! let w = config.unicode(&mut out, &pretty);
//! // w is the width of the output
//! // output is stored in `out`
//! ```

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    iter::repeat,
};

pub type Str<'a> = Cow<'a, str>;
/// This is recommended by `@rami3l` to supercede `Vec<Pretty>`.
/// Why not use this? Because Rust wouldn't let me!
/// https://github.com/rust-lang/rust/issues/23714
#[allow(dead_code)]
type Pretties<'a> = Cow<'a, [Pretty<'a>]>;
/// Good kids don't do this.
type BTreeMap<K, V> = Vec<(K, V)>;
/// Cow-str associative array
pub type CowAssocArr<'a> = BTreeMap<Str<'a>, Pretty<'a>>;
pub type StrAssocArr<'a> = BTreeMap<&'a str, Pretty<'a>>;

pub mod ascii;
pub mod unicode;

pub mod helper;

#[derive(Clone)]
pub struct XmlNode<'a> {
    pub name: Str<'a>,
    pub fields: CowAssocArr<'a>,
    /// Currently, if fields have `XmlNode` with children,
    /// they will not be considered during linearization.
    pub(crate) fields_is_linear: bool,
    pub children: Vec<Pretty<'a>>,
}

impl<'a> XmlNode<'a> {
    pub fn simple_record(
        name: impl Into<Str<'a>>,
        fields: StrAssocArr<'a>,
        children: Vec<Pretty<'a>>,
    ) -> Self {
        let name = name.into();
        let fields = fields.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Self::new(name, fields, children)
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty() || (self.fields.iter()).any(|(_, x)| x.has_children())
    }

    fn ol_build_str_ascii(&self, builder: &mut String) {
        builder.push_str(&self.name);
        if self.fields.is_empty() {
            return;
        }
        builder.push_str(" { ");
        for (i, (k, v)) in self.fields.iter().enumerate() {
            if i > 0 {
                builder.push_str(", ");
            }
            builder.push_str(k);
            builder.push_str(": ");
            v.ol_build_str_ascii(builder);
        }
        builder.push_str(" }");
    }

    fn ol_len(&self, reduced_ws: bool) -> usize {
        let mem: usize = (self.fields.iter())
            .map(|(k, v)| k.chars().count() + ": ".len() + v.ol_len(reduced_ws))
            .sum();
        let mid = self.fields.len().saturating_sub(1) * ", ".len();
        let begin_end = if self.fields.is_empty() {
            0
        } else {
            " {  }".len()
        } + self.name.chars().count();
        mem + mid + begin_end
    }

    pub fn new(name: Str<'a>, fields: CowAssocArr<'a>, children: Vec<Pretty<'a>>) -> Self {
        Self {
            name,
            fields,
            fields_is_linear: false,
            children,
        }
    }
}

/// Use `into`!!
#[derive(Clone)]
pub enum Pretty<'a> {
    Text(Str<'a>),
    Record(XmlNode<'a>),
    Array(Vec<Self>),
    Linearized(&'a Self, usize),
}

impl<'a> Pretty<'a> {
    pub fn simple_record(
        name: impl Into<Str<'a>>,
        fields: StrAssocArr<'a>,
        children: Vec<Self>,
    ) -> Self {
        Self::Record(XmlNode::simple_record(name, fields, children))
    }
    // Blame Rust for not having named arguments and default values.
    pub fn fieldless_record(name: impl Into<Str<'a>>, children: Vec<Self>) -> Self {
        Self::simple_record(name, Default::default(), children)
    }
    pub fn childless_record(name: impl Into<Str<'a>>, fields: StrAssocArr<'a>) -> Self {
        Self::simple_record(name, fields, Default::default())
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
        use Pretty::*;
        match self {
            Record(xml) => xml.has_children(),
            Array(v) => v.iter().any(Self::has_children),
            Text(..) => false,
            // Note: linearization happens only when children are absent
            Linearized(..) => false,
        }
    }

    /// Potential improvements: instead of using a mutable String,
    /// use a Write trait with monadic error reporting.
    pub(crate) fn ol_build_str_ascii(&self, builder: &mut String) {
        use Pretty::*;
        match self {
            Text(s) => builder.push_str(s),
            Record(xml) => xml.ol_build_str_ascii(builder),
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
            Linearized(p, _) => p.ol_build_str_ascii(builder),
        }
    }

    pub fn to_one_line_string(&self) -> String {
        let mut builder = String::with_capacity(self.ol_len(false));
        self.ol_build_str_ascii(&mut builder);
        builder
    }

    /// Does not include children of records.
    pub(crate) fn ol_len(&self, reduced_ws: bool) -> usize {
        use Pretty::*;
        match self {
            Text(s) => s.chars().count(),
            Record(xml) => xml.ol_len(reduced_ws),
            Array(v) => {
                if v.is_empty() {
                    return "[]".len();
                }
                let mem: usize = v.iter().map(|x| x.ol_len(reduced_ws)).sum();
                let mid = (v.len() - 1) * ", ".len();
                let beg = if reduced_ws { "[]".len() } else { "[  ]".len() };
                mem + mid + beg
            }
            Linearized(_, len) => *len,
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
    pub need_boundaries: bool,
    /// If true, then there will not be space before record name and enclosed
    /// in lists.
    pub reduced_spaces: bool,
}

impl PrettyConfig {
    pub fn horizon(&self, out: &mut String, width: usize) {
        if !self.need_boundaries {
            return;
        }
        out.push_str("+");
        out.extend(repeat("-").take(width + 2));
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
        if self.config.need_boundaries {
            self.out.push_str("| ");
        }
    }
    fn push(&mut self, s: &str) {
        self.out.push_str(s);
        self.already_occupied += s.chars().count();
    }
    fn pip(&mut self, amount: usize) {
        self.push(" ".repeat(amount).as_str());
    }
    fn pusheen(&mut self) {
        // if self.width < self.already_occupied {
        //     println!(
        //         "Bug!! w: {}, ao: {}",
        //         self.width, self.already_occupied
        //     );
        //     self.push(" |\n");
        // } else {
        let eol = if self.config.need_boundaries {
            self.pip(self.width - self.already_occupied);
            " |\n"
        } else {
            "\n"
        };
        self.push(eol);
        // }
        self.already_occupied = 0;
    }
}

impl Default for PrettyConfig {
    fn default() -> Self {
        Self {
            indent: 4,
            width: 120,
            need_boundaries: true,
            reduced_spaces: false,
        }
    }
}
