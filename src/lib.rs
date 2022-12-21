use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{Debug, Display},
};

type Str<'a> = Cow<'a, str>;

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

    #[allow(dead_code)]
    /// For debugging purposes.
    fn ol_to_string(&self) -> String {
        let mut builder = String::with_capacity(self.ol_len());
        self.ol_build_str(&mut builder);
        builder
    }

    fn ol_len(&self) -> usize {
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
    fn interesting(&self, base_indent: usize, pretty: &Pretty, additional: usize) -> usize {
        let first_line_base = base_indent + additional;
        let len = pretty.ol_len() + first_line_base;
        if len <= self.width {
            len
        } else {
            let next_indent = base_indent + self.indent;
            use Pretty::*;
            match pretty {
                Text(s) => s.chars().count() + first_line_base,
                Array(v) => v
                    .iter()
                    .map(|p| self.interesting(next_indent, p, 0) + ",".len())
                    .max()
                    .unwrap_or(first_line_base + "[".len()),
                Record(name, m) => {
                    let header = name.chars().count() + first_line_base + " {".len();
                    m.iter()
                        .map(|(k, v)| {
                            self.interesting(next_indent, v, k.chars().count() + ": ".len())
                        })
                        .chain(Some(header).into_iter())
                        .max()
                        .unwrap()
                }
            }
        }
    }

    pub fn horizon(out: &mut String, total_len: usize) {
        out.push_str("+");
        out.push_str("-".repeat(total_len - 2).as_str());
        out.push_str("+");
    }
    pub fn java(&self, out: &mut String, pretty: &Pretty) {
        let boundaries = "| ".len() + " |".len();
        let width = self.interesting(0, pretty, 0);
        let total_len = width + boundaries;
        let mut dat = LinedBuffer {
            out,
            width,
            config: self,
            already_occupied: 0,
        };
        Self::horizon(dat.out, total_len);
        dat.out.push_str("\n");

        dat.begin_line();
        dat.line(pretty, 0);
        dat.pusheen();

        Self::horizon(dat.out, total_len);
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

    fn line(&mut self, pretty: &Pretty, indent: usize) {
        use Pretty::*;
        let self_indent_len = indent * self.config.indent;
        let indent = indent + 1;
        let indent_len = indent * self.config.indent;

        let ol_len = pretty.ol_len();
        if ol_len + indent_len <= self.width {
            pretty.ol_build_str(self.out);
            self.already_occupied += ol_len;
        } else {
            match pretty {
                Text(s) => self.push(s),
                Array(v) => {
                    self.push("[");
                    self.pusheen();
                    for (i, e) in v.iter().enumerate() {
                        self.begin_line();
                        self.pip(indent_len);
                        self.line(e, indent);
                        if i < v.len() - 1 {
                            self.push(",");
                        }
                        self.pusheen();
                    }
                    self.begin_line();
                    self.pip(self_indent_len);
                    self.push("]");
                }
                Record(name, m) => {
                    self.push(name);
                    self.push(" {");
                    self.pusheen();
                    for (i, (k, v)) in m.iter().enumerate() {
                        self.begin_line();
                        self.pip(indent_len);
                        self.push(k);
                        self.push(": ");
                        self.line(v, indent);
                        if i < m.len() - 1 {
                            self.push(",");
                        }
                        self.pusheen();
                    }
                    self.begin_line();
                    self.pip(self_indent_len);
                    self.push("}");
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
