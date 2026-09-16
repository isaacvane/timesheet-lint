mod model;
mod parser;
mod print;

use std::env;
use std::fs;
use std::process;

struct Args {
    path: String,
    json: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut path = None;
    let mut json = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            other => {
                if path.is_some() {
                    return Err(format!("unexpected extra argument '{}'", other));
                }
                path = Some(other.to_string());
            }
        }
    }

    let path = path.ok_or_else(|| "missing timesheet file path".to_string())?;
    Ok(Args { path, json })
}

fn print_usage() {
    eprintln!("usage: timesheet [--json] <file>");
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("error: {}", msg);
            print_usage();
            process::exit(2);
        }
    };

    let contents = match fs::read_to_string(&args.path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: could not read '{}': {}", args.path, e);
            process::exit(2);
        }
    };

    match parser::parse(&contents) {
        Ok(sheet) => {
            if args.json {
                println!("{}", print::to_json(&sheet));
            } else {
                print!("{}", print::pretty_print(&sheet));
            }
        }
        Err(errors) => {
            if args.json {
                println!("{}", print::errors_to_json(&errors));
            } else {
                eprintln!("{} problem(s) found in '{}':", errors.len(), args.path);
                for e in &errors {
                    eprintln!("  line {}: {}", e.line, e.message);
                }
            }
            process::exit(1);
        }
    }
}
