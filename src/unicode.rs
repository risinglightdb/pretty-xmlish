use crate::{LinedBuffer, Pretty, PrettyConfig};

// https://www.w3.org/TR/xml-entity-names/025.html
const UP_RIGHT: char = '\u{2514}';
const UP_RIGHT_DOWN: char = '\u{251C}';
const LEFT_RIGHT: char = '\u{2500}';
const UP_DOWN: char = '\u{2502}';

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
                    let header = xml.name.chars().count() + first_line_base + ":".len();
                    todo!()
                }
            }
        }
    }
}
