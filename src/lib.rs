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
    /// Preferred width of the output, exlusive of the boundaries.
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

    pub fn java(&self, out: &mut String, pretty: &Pretty) {
        let boundaries = "# ".len() + " #".len();
        let width = self.interesting(0, pretty);
        let total_len = width + boundaries;
        out.push_str("#".repeat(total_len).as_str());
        out.push_str("\n");
        fn line(pretty: &Pretty, indent: usize, dat: &mut Data, need_comma: bool) {
            use Pretty::*;
            dat.push("# ");
            let indent_len = indent * dat.config.indent;
            dat.pip(indent_len);
            let ol_len = pretty.ol_len();
            if ol_len + indent_len <= dat.width {
                pretty.ol_build_str(dat.out);
                dat.already_occupied += ol_len;
                dat.pusheen();
            } else {
                match pretty {
                    Text(s) => {
                        dat.push(s);
                        if need_comma {
                            dat.push(",");
                        }
                        dat.pusheen();
                    }
                    Array(v) => {
                        dat.push("[");
                        dat.pusheen();
                        for (i, e) in v.iter().enumerate() {
                            line(e, indent + 1, dat, i < v.len() - 1);
                        }
                    }
                    _ => todo!(),
                }
            }
        }
        let mut dat = Data {
            out,
            width,
            config: self,
            already_occupied: 0,
        };
        line(pretty, 0, &mut dat, false);
        out.push_str("#".repeat(total_len).as_str());
    }
}

struct Data<'a> {
    width: usize,
    /// Modify when out is also modified.
    pub already_occupied: usize,
    out: &'a mut String,
    config: &'a PrettyConfig,
}
impl<'a> Data<'a> {
    fn push(&mut self, s: &str) {
        self.out.push_str(s);
        self.already_occupied += s.len();
    }
    fn pip(&mut self, amount: usize) {
        self.push(" ".repeat(amount).as_str());
    }
    fn pusheen(&mut self) {
        self.pip(self.width - self.already_occupied);
        self.already_occupied = 0;
        self.push(" #\n");
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
