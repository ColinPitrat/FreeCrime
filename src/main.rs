mod command;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "info" => {
            if args.len() < 3 {
                eprintln!("Usage: info <file>");
                return;
            }
            if let Err(e) = command::info::execute(&args[2]) {
                eprintln!("Error: {}", e);
            }
        }
        "extract" => {
            if args.len() < 4 {
                eprintln!("Usage: extract <file> <out>");
                return;
            }
            if let Err(e) = command::extract::execute(&args[2], &args[3]) {
                eprintln!("Error: {}", e);
            }
        }
        "overview" => {
            if args.len() < 3 {
                eprintln!("Usage: overview <cmp_file>");
                return;
            }
            if let Err(e) = command::overview::execute(&args[2]) {
                eprintln!("Error: {}", e);
            }
        }
        "display" => {
            if args.len() < 4 {
                eprintln!("Usage: display <cmp_file> <gry_file>");
                return;
            }
            if let Err(e) = command::display::execute(&args[2], &args[3]) {
                eprintln!("Error: {}", e);
            }
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("GTA Resource Tool");
    println!("Usage:");
    println!("  info <file>             Show information about a game file");
    println!("  extract <file> <out>    Extract content from a game file");
    println!("  overview <cmp>          Generate a top-down overview BMP of the map");
    println!("  display <cmp> <gry>     Interactive 3D map viewer (Bevy)");
}
