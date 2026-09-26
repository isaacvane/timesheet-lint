mod model;
mod parser;
mod print;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

enum Source {
    File(String),
    Stdin,
}

struct Args {
    source: Source,
    json: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut path = None;
    let mut json = false;
    let mut stdin = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            "--stdin" => stdin = true,
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

    if stdin {
        if path.is_some() {
            return Err("cannot pass both a file path and --stdin".to_string());
        }
        return Ok(Args { source: Source::Stdin, json });
    }

    let path = path.ok_or_else(|| "missing timesheet file path (or pass --stdin)".to_string())?;
    Ok(Args { source: Source::File(path), json })
}

fn print_usage() {
    eprintln!("usage: timesheet [--json] (<file> | --stdin)");
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

    let contents = match &args.source {
        Source::File(path) => match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: could not read '{}': {}", path, e);
                process::exit(2);
            }
        },
        Source::Stdin => {
            let mut buf = String::new();
            match io::stdin().read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(e) => {
                    eprintln!("error: could not read stdin: {}", e);
                    process::exit(2);
                }
            }
        }
    };

    let label = match &args.source {
        Source::File(path) => path.as_str(),
        Source::Stdin => "<stdin>",
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
                eprintln!("{} problem(s) found in '{}':", errors.len(), label);
                for e in &errors {
                    eprintln!("  line {}: {}", e.line, e.message);
                }
            }
            process::exit(1);
        }
    }
}
