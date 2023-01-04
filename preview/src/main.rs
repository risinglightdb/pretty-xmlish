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
        "Sofia".into(),
    ]);
    let editors = Pretty::Array(vec![
        "Emacs".into(),
        "Vim".into(),
        "Sublime Text".into(),
        // https://www.w3.org/TR/xml-entity-names/025.html
        "Atom\u{02514}".into(),
    ]);
    let xml = XmlNode {
        name: "Info".into(),
        fields: btreemap! {
            "Songs".into() => songs,
            "Editors".into() => editors,
        },
        children: Default::default(),
    };
    // let xml = XmlNode {
    //     name: "Outer info".into(),
    //     fields: btreemap! {
    //         "Demo field".into() => "233".into(),
    //     },
    //     children: vec![Pretty::Record(xml.clone()), Pretty::Record(xml)],
    // };
    let pretty = Pretty::Array(vec![pretty.clone(), Pretty::Record(xml), pretty]);
    let pretty = Pretty::Array(vec![pretty.clone(), pretty]);
    let mut out = String::new();
    config.unicode(&mut out, &pretty);
    out.push('\n');
    config.ascii(&mut out, &pretty);
    println!("{}", out);
}
