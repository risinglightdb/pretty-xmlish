use crate::{LinedBuffer, Pretty, PrettyConfig, Str};

/// https://www.w3.org/TR/xml-entity-names/025.html
/// These unicode characters are assumed to have length 1!
mod characters {
    pub const UP_RIGHT: char = '\u{2514}';
    pub const UP_RIGHT_DOWN: char = '\u{251C}';
    pub const LEFT_RIGHT: char = '\u{2500}';
    pub const UP_DOWN: char = '\u{2502}';
}

impl PrettyConfig {
    pub(crate) fn interesting_unicode(
        &self,
        base_indent: usize,
        pretty: &Pretty,
        additional: usize,
    ) -> usize {
        let first_line_base = base_indent + additional;
        let len = pretty.ol_len() + first_line_base;
        if len <= self.width {
            len
        } else {
            let next_indent = base_indent + self.indent;
            use Pretty::*;
            match pretty {
                Text(s) => s.chars().count() + first_line_base,
                Array(list) => list
                    .iter()
                    .map(|p| self.interesting_ascii(next_indent, p, 0) + ",".len())
                    .max()
                    .unwrap_or(first_line_base + "[".len()),
                Record(xml) => {
                    let header = xml.name.chars().count() + first_line_base;
                    let fields = (xml.fields.iter()).map(|(k, v)| {
                        self.interesting_unicode(next_indent, v, k.chars().count() + ": ".len())
                    });
                    (xml.children.iter())
                        .map(|p| self.interesting_unicode(next_indent, p, 0))
                        .chain(Some(header).into_iter())
                        .chain(fields)
                        .max()
                        .unwrap()
                }
            }
        }
    }
}

impl<'a> LinedBuffer<'a> {
    pub(crate) fn line_unicode(&mut self, pretty: &Pretty, prefix: Str) {
        use Pretty::*;
        let self_indent_len = prefix.len();
        let indent_len = self_indent_len + self.config.indent;
        let ol_len = pretty.ol_len();
        if ol_len + indent_len <= self.width {
            pretty.ol_build_str_ascii(self.out);
            self.already_occupied += ol_len;
        } else {
            match pretty {
                Text(s) => self.push(s),
                Array(list) => {
                    self.push("[");
                    let mut first = true;
                    for p in list {
                        if first {
                            first = false;
                        } else {
                            self.push(", ");
                        }
                        self.line_unicode(p, prefix.clone());
                    }
                    self.push("]");
                }
                Record(xml) => {
                    self.push(&xml.name);
                    self.pusheen();
                    let mut cont_prefix = prefix.clone();
                    {
                        let editor = cont_prefix.to_mut();
                        if self.config.indent > 0 {
                            editor.push(characters::UP_DOWN);
                        }
                    }
                    for (i, (k, v)) in xml.fields.iter().enumerate() {
                        self.begin_line();
                        self.push(&prefix);
                        self.push(k);
                        self.push(": ");
                        self.line_unicode(v, cont_prefix.clone());
                        self.pusheen();
                    }

                    todo!()
                }
            }
        }
    }
}
