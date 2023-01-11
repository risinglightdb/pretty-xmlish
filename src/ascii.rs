use crate::{LinedBuffer, Pretty, PrettyConfig, XmlNode};

impl PrettyConfig {
    pub fn ascii(&self, out: &mut String, pretty: &Pretty) {
        let boundaries = "| ".len() + " |".len();
        let width = self.interesting_ascii(0, pretty, 0, 0);
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
        dat.line_ascii(pretty, 0, 0);
        dat.pusheen();

        Self::horizon(dat.out, total_len);
    }

    pub(crate) fn interesting_ascii(
        &self,
        base_indent: usize,
        pretty: &Pretty,
        start_add: usize,
        end_add: usize,
    ) -> usize {
        let first_line_base = base_indent + start_add + end_add;
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
                    .map(|p| self.interesting_ascii(next_indent, p, 0, ",".len()))
                    .max()
                    .unwrap_or(first_line_base + "[]".len() + end_add),
                Record(xml) => {
                    let header = xml.name.chars().count() + first_line_base + " {".len();
                    let children = (xml.children.iter().enumerate()).map(|(i, p)| {
                        let at_the_end = if i < xml.children.len() - 1 {
                            ",".len()
                        } else {
                            0
                        };
                        self.interesting_ascii(next_indent, p, 0, at_the_end)
                    });
                    (xml.fields.iter().enumerate())
                        .map(|(i, (k, v))| {
                            let end = if i < xml.fields.len() - 1 {
                                ",".len()
                            } else {
                                0
                            };
                            let start = k.chars().count() + ": ".len();
                            self.interesting_ascii(next_indent, v, start, end)
                        })
                        .chain(vec![header, "}".len() + end_add].into_iter())
                        .chain(children)
                        .max()
                        .unwrap()
                }
            }
        }
    }
}

impl<'a> LinedBuffer<'a> {
    pub(crate) fn line_ascii(
        &mut self,
        pretty: &Pretty,
        self_indent_len: usize,
        additional: usize,
    ) {
        use Pretty::*;
        let indent_len = self_indent_len + self.config.indent;

        let ol_len = pretty.ol_len();
        if !pretty.has_children() && ol_len + indent_len + additional < self.width {
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
                        self.line_ascii(e, indent_len, 0);
                        if i < v.len() - 1 {
                            self.push(",");
                        }
                        self.pusheen();
                    }
                    self.begin_line();
                    self.pip(self_indent_len);
                    self.push("]");
                }
                Record(xml) => self.line_ascii_xml(xml, indent_len, self_indent_len),
            }
        }
    }

    fn line_ascii_xml(&mut self, xml: &XmlNode, indent_len: usize, self_indent_len: usize) {
        self.push(&xml.name);
        self.push(" {");
        self.pusheen();
        for (i, (k, v)) in xml.fields.iter().enumerate() {
            self.begin_line();
            self.pip(indent_len);
            self.push(k);
            self.push(": ");
            self.line_ascii(v, indent_len, k.len() + ": ".len());
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
            self.line_ascii(child, indent_len, 0);
        }
    }
}
