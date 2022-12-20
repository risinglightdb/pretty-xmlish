use data_display::*;

fn main() {
    let config = PrettyConfig::default();
    let pretty = Pretty::Text("Hello, world!".into());
    let mut out = String::new();
    config.java(&mut out, &pretty);
    println!("{}", out);
}
