use crate::{LinedBuffer, Pretty, PrettyConfig};

// https://www.w3.org/TR/xml-entity-names/025.html
const UP_RIGHT: char = '\u{2514}';
const UP_RIGHT_DOWN: char = '\u{251C}';
const LEFT_RIGHT: char = '\u{2500}';
const UP_DOWN: char = '\u{2502}';

impl<'a> Pretty<'a> {
    pub(crate) fn ol_len_unicode(&self) -> usize {
        use Pretty::*;
        match self {
            Text(s) => s.chars().count(),
            Array(v) => {
                let mem: usize = v.iter().map(Self::ol_len_ascii).sum();
                let mid = (v.len() - 1) * ", ".len();
                let beg = "[ ".len() + " ]".len();
                mem + mid + beg
            }
            _ => todo!()
        }
    }
}
