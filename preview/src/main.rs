use pretty_xmlish::{Pretty, PrettyConfig};

fn main() {
    let mut config = PrettyConfig::default();
    for arg in std::env::args() {
        if let Some(n) = arg.parse().ok() {
            config.width = n;
        }
    }
    // BatchNestedLoopJoin { type: Inner, predicate: ($0 = ($3 + $4)), output_indices: all }
    //   BatchExchange { order: [], dist: Single }
    //     BatchScan { table: t1, columns: [v1, v2, v3] }
    //   BatchExchange { order: [], dist: Single }
    //     BatchScan { table: t2, columns: [v1, v2, v3] }
    let pretty = Pretty::simple_record(
        "BatchNestedLoopJoin",
        vec![
            ("type", "Inner".into()),
            ("predicate", "($0 = ($3 + $4))".into()),
            ("output_indices", "all".into()),
        ],
        vec![
            Pretty::simple_record(
                "BatchExchange",
                vec![("order", Pretty::Array(vec![])), ("dist", "Single".into())],
                vec![Pretty::childless_record(
                    "BatchScan",
                    vec![
                        ("table", "t1".into()),
                        ("columns", Pretty::list_of_strings(&["v1", "v2", "v3"])),
                    ],
                )],
            ),
            Pretty::simple_record(
                "BatchExchange",
                vec![("order", Pretty::Array(vec![])), ("dist", "Single".into())],
                vec![Pretty::childless_record(
                    "BatchScan",
                    vec![
                        ("table", "t2".into()),
                        ("columns", Pretty::list_of_strings(&["v1", "v4444444444444444444444444444444444444444444444447777777777777777777777777777772", "v3"])),
                    ],
                )],
            ),
        ],
    );
    let mut out = String::new();
    let w = config.unicode(&mut out, &pretty);
    out.push('\n');
    config.need_boundaries = false;
    config.unicode(&mut out, &pretty);
    // config.ascii(&mut out, &pretty);
    println!("{}\nActual width: {}", out, w);
}
