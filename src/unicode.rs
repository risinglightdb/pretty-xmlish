use std::iter::repeat;

use crate::{LinedBuffer, Pretty, PrettyConfig, XmlNode};

/// https://www.w3.org/TR/xml-entity-names/025.html
/// These unicode characters are assumed to have length 1!
mod characters {
    pub const UR: char = '\u{2514}';
    pub const DR: char = '\u{250C}';
    pub const URD: char = '\u{251C}';
    pub const LR: char = '\u{2500}';
    pub const UD: char = '\u{2502}';
}

impl PrettyConfig {
    pub fn unicode(&mut self, out: &mut String, pretty: &Pretty) -> usize {
        let (pretty, width) = self.interesting_unicode(0, pretty, 0);
        self.width = width;
        // The second time folds previous lines that can be wrapped with the extended width.
        let (pretty, width) = self.interesting_unicode(0, &pretty, 0);
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
        dat.line_unicode(&pretty, 0, Default::default(), "");
        if self.need_boundaries {
            dat.pusheen();
        }

        self.horizon(dat.out, width);
        width
    }

    pub(crate) fn interesting_unicode<'a>(
        &self,
        base_indent: usize,
        pretty: &'a Pretty<'a>,
        additional: usize,
    ) -> (Pretty<'a>, usize) {
        let first_line_base = base_indent + additional;
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
                if v.len() == 1 {
                    return self.interesting_unicode(base_indent, &v[0], additional);
                }
                let (v, lens): (Vec<_>, Vec<_>) = (v.iter())
                    .map(|p| self.interesting_unicode(next_indent, p, 0))
                    .unzip();
                let max = (lens.into_iter().max()).unwrap_or(first_line_base + "[]".len());
                (Array(v), max)
            }
            Record(xml) => {
                let header = xml.name.chars().count() + first_line_base;
                // Here, `len` does not include the children
                let fields_is_linear = xml.fields_is_linear
                    || (len < self.width && xml.fields.iter().all(|(_, t)| !t.has_children()));
                let (fields, f_lens): (Vec<_>, Vec<_>) = (xml.fields.iter())
                    .map(|(k, v)| {
                        let additional = k.chars().count() + ": ".len();
                        let (f, len) = self.interesting_unicode(next_indent, v, additional);
                        ((k.clone(), f), len)
                    })
                    .unzip();
                let (children, c_lens): (Vec<_>, Vec<_>) = (xml.children.iter())
                    .map(|p| self.interesting_unicode(next_indent, p, 0))
                    .unzip();
                let max = (c_lens.into_iter())
                    .chain(if fields_is_linear {
                        vec![len].into_iter()
                    } else {
                        f_lens.into_iter()
                    })
                    .chain(Some(header).into_iter())
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

    pub(crate) fn append_prefix(&self, editor: &str, start: char, fill: char) -> String {
        let indent = self.indent;
        let mut editor = editor.to_string();
        let mut remaining = indent;
        if remaining > 1 {
            editor.push(start);
            remaining -= 1;
        }
        editor.extend(repeat(fill).take(remaining - 1));
        if !self.reduced_spaces {
            editor.push(' ');
        }
        editor
    }
}

impl<'a> LinedBuffer<'a> {
    pub(crate) fn line_unicode(
        &mut self,
        pretty: &Pretty,
        indent_len: usize,
        prefix: &str,
        one_line_prefix: &str,
    ) {
        use Pretty::*;
        let indent_len = indent_len + self.config.indent;

        enum Cubical<'a> {
            Cartesian(&'a [Pretty<'a>]),
            DeMorgan(&'a XmlNode<'a>),
        }
        use Cubical::*;
        let regularity = match pretty {
            Text(s) => {
                self.push(one_line_prefix);
                self.push(s);
                return;
            }
            Linearized(p, ol_len) => {
                self.push(one_line_prefix);
                p.ol_build_str_ascii(self.config.reduced_spaces, self.out);
                self.already_occupied += ol_len;
                return;
            }
            Record(xml) => DeMorgan(xml),
            Array(list) => Cartesian(list),
        };
        use characters::*;
        let cont_prefix = self.config.append_prefix(prefix, UD, ' ');
        let last_cont_prefix = self.config.append_prefix(prefix, ' ', ' ');
        let fields_prefix = self.config.append_prefix(prefix, URD, LR);
        let last_field_prefix = self.config.append_prefix(prefix, UR, LR);
        let choose = |is_not_last_line: bool| {
            if is_not_last_line {
                (&cont_prefix, &fields_prefix)
            } else {
                (&last_cont_prefix, &last_field_prefix)
            }
        };
        match regularity {
            Cartesian(list) => {
                if list.is_empty() {
                    self.push("[]");
                    return;
                }
                use characters::*;
                let fst_field_prefix = self.config.append_prefix(prefix, DR, LR);
                self.pusheen();
                for (i, p) in list.iter().enumerate() {
                    self.begin_line();
                    let is_not_last_line = i < list.len() - 1;
                    let (cont_prefix, fields_prefix) = if i == 0 {
                        (&cont_prefix, &fst_field_prefix)
                    } else {
                        choose(is_not_last_line)
                    };
                    self.push(&fields_prefix);
                    self.line_unicode(p, indent_len, &cont_prefix, "");
                    if is_not_last_line {
                        self.pusheen();
                    }
                }
            }
            DeMorgan(xml) => self.line_unicode_xml(xml, choose, indent_len),
        }
    }

    fn line_unicode_xml<'b>(
        &mut self,
        xml: &XmlNode,
        choose: impl Fn(bool) -> (&'b String, &'b String),
        indent_len: usize,
    ) {
        let has_children = xml.has_children();
        if xml.fields_is_linear {
            let reduced_ws = self.config.reduced_spaces;
            xml.ol_build_str_ascii(reduced_ws, self.out);
            self.already_occupied += xml.ol_len(reduced_ws);
            self.pusheen();
        } else {
            self.push(&xml.name);
            self.pusheen();
            for (i, (k, v)) in xml.fields.iter().enumerate() {
                self.begin_line();
                let is_not_last_line = has_children || i < xml.fields.len() - 1;
                let (cont_prefix, fields_prefix) = choose(is_not_last_line);
                self.push(&fields_prefix);
                self.push(k);
                self.push(":");
                self.line_unicode(v, indent_len + k.len() + ": ".len(), &cont_prefix, " ");
                if is_not_last_line {
                    self.pusheen();
                }
            }
        }
        for (i, child) in xml.children.iter().enumerate() {
            self.begin_line();
            let is_not_last_line = i < xml.children.len() - 1;
            let (cont_prefix, fields_prefix) = choose(is_not_last_line);
            self.push(&fields_prefix);
            // let prefix = if self.config.reduced_spaces { "" } else { " " };
            self.line_unicode(child, indent_len, &cont_prefix, "");
            if is_not_last_line {
                self.pusheen();
            }
        }
    }
}
