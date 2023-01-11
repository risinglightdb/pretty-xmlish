use crate::{LinedBuffer, Pretty, PrettyConfig, XmlNode};

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

    /// Parameter 'additional' is added to the first line **only**.
    pub(crate) fn interesting_ascii(
        &self,
        base_indent: usize,
        pretty: &Pretty,
        additional: usize,
    ) -> usize {
        let first_line_base = base_indent + additional;
        let len = pretty.ol_len() + first_line_base;
        if !pretty.has_children() && len <= self.width {
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
                    .unwrap_or(first_line_base + "[]".len()),
                Record(xml) => {
                    let header = xml.name.chars().count() + first_line_base + " {".len();
                    let children = (xml.children.iter().enumerate()).map(|(i, p)| {
                        let at_the_end = if i < xml.children.len() - 1 {
                            ",".len()
                        } else {
                            0
                        };
                        self.interesting_ascii(next_indent, p, 0) + at_the_end
                    });
                    (xml.fields.iter())
                        .map(|(k, v)| {
                            self.interesting_ascii(next_indent, v, k.chars().count() + ": ".len())
                        })
                        .chain(Some(header).into_iter())
                        .chain(children)
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

        let ol_len = pretty.ol_len();
        if !pretty.has_children() && ol_len + indent_len < self.width {
            pretty.ol_build_str_ascii(self.out);
            self.already_occupied += ol_len;
        } else {
            match pretty {
                Text(s) => self.push(s),
                Array(v) => {
                    self.push("[");
                    if v.is_empty() {
                        self.push("]");
                        return;
                    }
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
                Record(xml) => self.line_ascii_xml(xml, indent_len, indent, self_indent_len),
            }
        }
    }

    fn line_ascii_xml(
        &mut self,
        xml: &XmlNode,
        indent_len: usize,
        indent: usize,
        self_indent_len: usize,
    ) {
        self.push(&xml.name);
        self.push(" {");
        self.pusheen();
        for (i, (k, v)) in xml.fields.iter().enumerate() {
            self.begin_line();
            self.pip(indent_len);
            self.push(k);
            self.push(": ");
            self.line_ascii(v, indent);
            if i < xml.fields.len() - 1 {
                self.push(",");
            }
            self.pusheen();
        }
        self.begin_line();
        self.pip(self_indent_len);
        self.push("}");
        for child in xml.children.iter() {
            self.pusheen();
            self.begin_line();
            self.pip(indent_len);
            self.line_ascii(child, indent);
        }
    }
}
