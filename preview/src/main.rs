use data_display::*;
#[macro_use]
extern crate maplit;

fn main() {
    let mut config = PrettyConfig::default();
    for arg in std::env::args() {
        if let Some(n) = arg.parse().ok() {
            config.width = n;
        }
    }
    let pretty = Pretty::Array(vec!["Lorem ipsum".into(), "2 < 1".into()]);
    let songs = Pretty::Array(vec![
        "Feel Good Inc.".into(),
        "Smooth".into(),
        "Ch-ch-ch-changes".into(),
        "Human after all".into(),
    ]);
    let editors = Pretty::Array(vec![
        "Emacs".into(),
        "Vim".into(),
        "Sublime Text".into(),
        // https://www.w3.org/TR/xml-entity-names/025.html
        "Atom\u{02514}".into(),
    ]);
    let pretty = Pretty::Array(vec![
        pretty.clone(),
        Pretty::Record(
            "Info".into(),
            btreemap! {
                "Songs".into() => songs,
                "Editors".into() => editors,
            },
        ),
        pretty,
    ]);
    let pretty = Pretty::Array(vec![pretty.clone(), pretty]);
    let mut out = String::new();
    config.java(&mut out, &pretty);
    println!("{}", out);
}
