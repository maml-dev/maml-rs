use std::env;
use std::fs;

use maml::parser::parse_with_report;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.maml>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let src = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    match parse_with_report(filename, &src) {
        Some(value) => {
            println!("{:#?}", value);

            #[cfg(feature = "serde")]
            if let Ok(json) = serde_json::to_string_pretty(&value) {
                println!("\nAs JSON:\n{}", json);
            }
        }
        None => std::process::exit(1),
    }
}
