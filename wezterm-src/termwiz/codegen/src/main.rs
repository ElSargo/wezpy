use std::io::Write;
use std::path::Path;

const VARIATION_SEQUENCES: &str = include_str!("../../data/emoji-variation-sequences.txt");

include!("../../src/emoji_presentation.rs");
include!("../../src/widechar_width.rs");

fn main() {
    println!("//! This file was generated by running:");
    println!("//! cd ../codegen ; cargo run > ../emoji_variation.rs");
    println!();
    emit_variation_map();
    emit_classify_table();
    generate_nerdfonts_data();
}

fn generate_nerdfonts_data() {
    let mut symbols: Vec<(String, char)> = vec![];

    for set in &["cod", "dev", "fa", "fae", "iec", "logos", "md", "oct", "ple", "pom", "seti", "weather"] {
        let url = format!("https://raw.githubusercontent.com/ryanoasis/nerd-fonts/master/bin/scripts/lib/i_{set}.sh");
        let filename = format!("/tmp/termwiz-data-fa-{set}.sh");

        if !Path::new(&filename).exists() {
            let body = reqwest::blocking::get(url).unwrap().text().unwrap();
            std::fs::write(&filename, &body).unwrap();
        }

        let data = std::fs::read_to_string(&filename).unwrap();

        for line in data.lines() {
            let fields: Vec<&str> = line.split(' ').collect();
            if fields.len() == 2 {
                if let Some(codepoint) = fields[0].strip_prefix("i='").and_then(|s| s.strip_suffix("'")) {
                    if let Some(name) = fields[1].strip_prefix("i_").and_then(|s| s.strip_suffix("=$i")) {
                        let codepoint = codepoint.chars().next().unwrap();
                        symbols.push((name.to_string(), codepoint));
                    }
                }
            }
        }
    }

    symbols.sort_by(|(a, _), (b, _)| human_sort::compare(a, b));

    let mut f = std::fs::File::create("../src/nerdfonts_data.rs").unwrap();

    writeln!(f, "//! Data mapping nerd font symbol names to their char codepoints").unwrap();
    writeln!(f, "//! This file was generated by running:").unwrap();
    writeln!(f, "//! cd ../codegen ; cargo run").unwrap();
    writeln!(f, "pub const NERD_FONT_GLYPHS: &[(&str, char)] = &[").unwrap();
    for (name, codepoint) in &symbols {
        writeln!(f, "    (\"{name}\", '{}'), // {codepoint}", codepoint.escape_unicode()).unwrap();
    }
    writeln!(f, "];").unwrap();
}

fn emit_classify_table() {
    let table = WcLookupTable::new();
    println!("use crate::widechar_width::{{WcLookupTable, WcWidth}};");
    println!();
    println!("pub const WCWIDTH_TABLE: WcLookupTable = WcLookupTable {{");
    println!("  table: [");

    for c in &table.table {
        println!("  WcWidth::{:?},", c);
    }

    println!("]}};");
}

/// Parses emoji-variation-sequences.txt, which is part of the UCD download
/// for a given version of the Unicode spec.
/// It defines which sequences can have explicit presentation selectors.
fn emit_variation_map() {
    let mut map = phf_codegen::Map::new();

    'next_line: for line in VARIATION_SEQUENCES.lines() {
        if let Some(lhs) = line.split('#').next() {
            if let Some(seq) = lhs.split(';').next() {
                let mut s = String::new();
                let mut last = None;
                for hex in seq.split_whitespace() {
                    match u32::from_str_radix(hex, 16) {
                        Ok(n) => {
                            let c = char::from_u32(n).unwrap();
                            s.push(c);
                            last.replace(c);
                        }
                        Err(_) => {
                            continue 'next_line;
                        }
                    }
                }

                if let Some(last) = last {
                    let first = if EMOJI_PRESENTATION.contains_u32(s.chars().next().unwrap() as u32) {
                        "Presentation::Emoji"
                    } else {
                        "Presentation::Text"
                    };
                    map.entry(
                        s,
                        &format!("({}, {})", first,
                        match last {
                            '\u{FE0F}' => "Presentation::Emoji",
                            '\u{FE0E}' => "Presentation::Text",
                            _ => unreachable!(),
                        }),
                    );
                }
            }
        }
    }

    println!("use crate::emoji::Presentation;");
    println!();
    println!(
        "pub static VARIATION_MAP: phf::Map<&'static str, (Presentation, Presentation)> = \n{};\n",
        map.build(),
    );
}