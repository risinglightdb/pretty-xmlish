use std::fmt::{Formatter, Result};

use crate::{Pretty, PrettyConfig};

pub fn delegate_fmt<'a>(me: &Pretty<'a>, f: &mut Formatter<'_>, mut buffer: String) -> Result {
    let mut config = PrettyConfig {
        need_boundaries: false,
        ..PrettyConfig::default()
    };
    config.unicode(&mut buffer, &me);
    f.write_str(&buffer)
}
