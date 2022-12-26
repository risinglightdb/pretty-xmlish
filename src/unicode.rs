use crate::{LinedBuffer, Pretty, PrettyConfig};

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
