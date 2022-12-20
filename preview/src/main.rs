use data_display::*;

fn main() {
    let config = PrettyConfig::default();
    let pretty = Pretty::Array(vec!["Lorem ipsum".into(), "2 < 1".into()]);
    let songs = Pretty::Array(vec![
        "Feel Good Inc.".into(),
        "Smooth".into(),
        "Ch-ch-ch-changes".into(),
        "Human after all".into(),
    ]);
    let pretty = Pretty::Array(vec![
        pretty.clone(),
        "Visual Studio Code".into(),
        songs,
        pretty,
    ]);
    let pretty = Pretty::Array(vec![pretty.clone(), pretty]);
    let mut out = String::new();
    config.java(&mut out, &pretty);
    println!("{}", out);
}
