use std::iter::repeat;

use crate::{LinedBuffer, Pretty, PrettyConfig, Str};

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
    pub fn unicode(&self, out: &mut String, pretty: &Pretty) {
        let boundaries = "| ".len() + " |".len();
        let width = self.interesting_unicode(0, pretty, 0);
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
        dat.line_unicode(pretty, 0, Default::default());
        dat.pusheen();

        Self::horizon(dat.out, total_len);
    }

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

fn append_str(mut base: Str, action: impl FnOnce(&mut String)) -> Str {
    let x = base.to_mut();
    action(x);
    base
}

fn append_prefix(base: Str, indent: usize, start: char, fill: char) -> Str {
    append_str(base, |editor| {
        let mut remaining = indent;
        if remaining > 1 {
            editor.push(start);
            remaining -= 1;
        }
        editor.extend(repeat(fill).take(remaining - 1));
        editor.push(' ');
    })
}

impl<'a> LinedBuffer<'a> {
    pub(crate) fn line_unicode(&mut self, pretty: &Pretty, current_indent: usize, prefix: Str) {
        use Pretty::*;
        let current_indent = current_indent + 1;
        let indent_len = current_indent * self.config.indent;

        let ol_len = pretty.ol_len();
        if ol_len + indent_len <= self.width {
            pretty.ol_build_str_ascii(self.out);
            self.already_occupied += ol_len;
        } else {
            match pretty {
                Text(s) => self.push(s),
                Array(list) => {
                    self.push("[");
                    self.pusheen();
                    let el_prefix = append_prefix(prefix.clone(), self.config.indent, ' ', ' ');
                    for p in list {
                        self.begin_line();
                        self.push(&el_prefix);
                        self.line_unicode(p, current_indent, el_prefix.clone());
                        self.pusheen();
                    }
                    self.begin_line();
                    self.push(&prefix);
                    self.push("]");
                }
                Record(xml) => {
                    use characters::*;
                    self.push(&xml.name);
                    self.pusheen();
                    let idt = self.config.indent;
                    let cont_prefix = append_prefix(prefix.clone(), idt, UD, ' ');
                    let last_cont_prefix = append_prefix(prefix.clone(), idt, ' ', ' ');
                    let fields_prefix = append_prefix(prefix.clone(), idt, URD, LR);
                    let last_field_prefix = append_prefix(prefix, idt, UR, LR);
                    for (i, (k, v)) in xml.fields.iter().enumerate() {
                        self.begin_line();
                        let is_not_last_line = i < xml.fields.len() - 1;
                        let (cont_prefix, fields_prefix) = if is_not_last_line {
                            (&cont_prefix, &fields_prefix)
                        } else {
                            (&last_cont_prefix, &last_field_prefix)
                        };
                        self.push(&fields_prefix);
                        self.push(k);
                        self.push(": ");
                        self.line_unicode(v, current_indent, cont_prefix.clone());
                        if is_not_last_line {
                            self.pusheen();
                        }
                    }
                    // TODO: children
                }
            }
        }
    }
}
