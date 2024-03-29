use crate::{LinedBuffer, Pretty, PrettyConfig, XmlNode};

impl PrettyConfig {
    pub fn ascii(&mut self, out: &mut String, pretty: &Pretty) {
        let (pretty, width) = self.interesting_ascii(0, pretty, 0, 0);
        self.width = width;
        let (pretty, width) = self.interesting_ascii(0, &pretty, 0, 0);
        let mut dat = LinedBuffer {
            out,
            width,
            config: self,
            already_occupied: 0,
        };
        self.horizon(dat.out, width);
        if self.need_boundaries {
            dat.out.push_str("\n");
        }
        dat.begin_line();
        dat.line_ascii(&pretty, 0);
        if self.need_boundaries {
            dat.pusheen();
        }
        self.horizon(dat.out, width);
    }

    pub(crate) fn interesting_ascii<'a>(
        &self,
        base_indent: usize,
        pretty: &'a Pretty<'a>,
        start_add: usize,
        end_add: usize,
    ) -> (Pretty<'a>, usize) {
        let first_line_base = base_indent + start_add + end_add;
        let ol_len = pretty.ol_len(self.reduced_spaces);
        let len = ol_len + first_line_base;
        if !pretty.has_children() && len <= self.width {
            return (Pretty::Linearized(pretty, ol_len), len);
        }
        let next_indent = base_indent + self.indent;
        use Pretty::*;
        match pretty {
            Text(s) => {
                let len = s.chars().count() + first_line_base;
                (s.as_ref().into(), len)
            }
            Array(v) => {
                let (v, lens): (Vec<_>, Vec<_>) = (v.iter())
                    .map(|p| self.interesting_ascii(next_indent, p, 0, ",".len()))
                    .unzip();
                let max =
                    (lens.into_iter().max()).unwrap_or(first_line_base + "[]".len() + end_add);
                (Array(v), max)
            }
            Record(xml) => {
                let header = xml.name.chars().count() + first_line_base + " {".len();
                let (children, c_lens): (Vec<_>, Vec<_>) = (xml.children.iter().enumerate())
                    .map(|(i, p)| {
                        let at_the_end = if i < xml.children.len() - 1 {
                            ",".len()
                        } else {
                            0
                        };
                        self.interesting_ascii(next_indent, p, 0, at_the_end)
                    })
                    .unzip();
                let (fields, f_lens): (Vec<_>, Vec<_>) = (xml.fields.iter().enumerate())
                    .map(|(i, (k, v))| {
                        let end = if i < xml.fields.len() - 1 {
                            ",".len()
                        } else {
                            0
                        };
                        let start = k.chars().count() + ": ".len();
                        let (f, len) = self.interesting_ascii(next_indent, v, start, end);
                        ((k.clone(), f), len)
                    })
                    .unzip();
                let fields_is_linear = len < self.width;
                let max = (c_lens.into_iter())
                    .chain(vec![header, "}".len() + end_add].into_iter())
                    .chain(if fields_is_linear {
                        vec![len].into_iter()
                    } else {
                        f_lens.into_iter()
                    })
                    .max()
                    .unwrap();
                let xml_node = XmlNode {
                    name: xml.name.clone(),
                    fields,
                    fields_is_linear,
                    children,
                };
                (Record(xml_node), max)
            }
            Linearized(_, l) => (pretty.clone(), *l),
        }
    }
}

impl<'a> LinedBuffer<'a> {
    pub(crate) fn line_ascii(&mut self, pretty: &Pretty, self_indent_len: usize) {
        let indent_len = self_indent_len + self.config.indent;
        use Pretty::*;
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
                    self.line_ascii(e, indent_len);
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
            Linearized(p, ol_len) => {
                p.ol_build_str_ascii(self.config.reduced_spaces, self.out);
                self.already_occupied += ol_len;
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
            self.line_ascii(v, indent_len);
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
            self.line_ascii(child, indent_len);
        }
    }
}
