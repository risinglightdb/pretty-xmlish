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
    // BatchNestedLoopJoin { type: Inner, predicate: ($0 = ($3 + $4)), output_indices: all }
    //   BatchExchange { order: [], dist: Single }
    //     BatchScan { table: t1, columns: [v1, v2, v3] }
    //   BatchExchange { order: [], dist: Single }
    //     BatchScan { table: t2, columns: [v1, v2, v3] }
    let pretty = Pretty::simple_record(
        "BatchNestedLoopJoin",
        btreemap! {
            "type" => "Inner".into(),
            "predicate" => "($0 = ($3 + $4))".into(),
            "output_indices" => "all".into(),
        },
        vec![
            Pretty::simple_record(
                "BatchExchange",
                btreemap! {
                    "order" => Pretty::Array(vec![]),
                    "dist" => "Single".into(),
                },
                vec![Pretty::simple_record(
                    "BatchScan",
                    btreemap! {
                        "table" => "t1".into(),
                        "columns" => Pretty::list_of_strings(&["v1", "v2", "v3"])
                    },
                    vec![],
                )],
            ),
            Pretty::simple_record(
                "BatchExchange",
                btreemap! {
                    "order" => Pretty::Array(vec![]),
                    "dist" => "Single".into(),
                },
                vec![Pretty::simple_record(
                    "BatchScan",
                    btreemap! {
                        "table" => "t2".into(),
                        "columns" => Pretty::list_of_strings(&["v1", "v2", "v3"]),
                    },
                    vec![],
                )],
            ),
        ],
    );
    let mut out = String::new();
    config.unicode(&mut out, &pretty);
    // out.push('\n');
    // config.ascii(&mut out, &pretty);
    println!("{}", out);
}
