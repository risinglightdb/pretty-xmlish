use crate::{LinedBuffer, Pretty, PrettyConfig};

impl<'a> Pretty<'a> {
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
        let mut builder = String::with_capacity(self.ol_len_ascii());
        self.ol_build_str_ascii(&mut builder);
        builder
    }

    pub(crate) fn ol_len_ascii(&self) -> usize {
        use Pretty::*;
        match self {
            Text(s) => s.chars().count(),
            Record(name, m) => {
                let mem: usize = m
                    .iter()
                    .map(|(k, v)| k.chars().count() + ": ".len() + v.ol_len_ascii())
                    .sum();
                let mid = (m.len() - 1) * ", ".len();
                let beg = " { ".len() + " }".len() + name.chars().count();
                mem + mid + beg
            }
            Array(v) => {
                let mem: usize = v.iter().map(Self::ol_len_ascii).sum();
                let mid = (v.len() - 1) * ", ".len();
                let beg = "[ ".len() + " ]".len();
                mem + mid + beg
            }
        }
    }
}

impl PrettyConfig {
    pub fn ascii(&self, out: &mut String, pretty: &Pretty) {
        let boundaries = "| ".len() + " |".len();
        let width = self.interesting_ascii(0, pretty, 0);
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
        dat.line_ascii(pretty, 0);
        dat.pusheen();

        Self::horizon(dat.out, total_len);
    }

    pub(crate) fn interesting_ascii(
        &self,
        base_indent: usize,
        pretty: &Pretty,
        additional: usize,
    ) -> usize {
        let first_line_base = base_indent + additional;
        let len = pretty.ol_len_ascii() + first_line_base;
        if len <= self.width {
            len
        } else {
            let next_indent = base_indent + self.indent;
            use Pretty::*;
            match pretty {
                Text(s) => s.chars().count() + first_line_base,
                Array(v) => v
                    .iter()
                    .map(|p| self.interesting_ascii(next_indent, p, 0) + ",".len())
                    .max()
                    .unwrap_or(first_line_base + "[".len()),
                Record(name, m) => {
                    let header = name.chars().count() + first_line_base + " {".len();
                    m.iter()
                        .map(|(k, v)| {
                            self.interesting_ascii(next_indent, v, k.chars().count() + ": ".len())
                        })
                        .chain(Some(header).into_iter())
                        .max()
                        .unwrap()
                }
            }
        }
    }
}

impl<'a> LinedBuffer<'a> {
    pub(crate) fn line_ascii(&mut self, pretty: &Pretty, indent: usize) {
        use Pretty::*;
        let self_indent_len = indent * self.config.indent;
        let indent = indent + 1;
        let indent_len = indent * self.config.indent;

        let ol_len = pretty.ol_len_ascii();
        if ol_len + indent_len <= self.width {
            pretty.ol_build_str_ascii(self.out);
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
                        self.line_ascii(e, indent);
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
                        self.line_ascii(v, indent);
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
