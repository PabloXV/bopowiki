use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("bopomofo_dict.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(
        &mut file,
        "static BOPOMOFO_DICT: phf::Map<&'static str, &'static str> = "
    )
    .unwrap();

    let mut builder = phf_codegen::Map::new();

    let csv = include_str!("data/tsi_dedup.csv");
    let mut seen = std::collections::HashSet::new();

    for line in csv.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            let word = parts[0];
            if !seen.contains(word) {
                seen.insert(word);
                builder.entry(word, &format!("\"{}\"", parts[2]));
            }
        }
    }

    writeln!(&mut file, "{}", builder.build()).unwrap();
    write!(&mut file, ";\n").unwrap();
}
