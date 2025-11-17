use std::env;
use std::fs::read_to_string;
use sea_core::parser::parse_to_graph;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: sea validate <file>");
        std::process::exit(2);
    }

    let command = &args[1];
    match command.as_str() {
        "validate" => {
            let filepath = &args[2];
            let source = match read_to_string(filepath) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", filepath, e);
                    std::process::exit(1);
                }
            };
            match parse_to_graph(&source) {
                Ok(graph) => {
                    let result = graph.validate();
                    if result.error_count > 0 {
                        println!("Validation failed: {} errors", result.error_count);
                        for v in result.violations {
                            println!("- [{}] {}: {}", match v.severity {
                                sea_core::policy::Severity::Error => "ERROR",
                                sea_core::policy::Severity::Warning => "WARN",
                                sea_core::policy::Severity::Info => "INFO",
                            }, v.policy_name, v.message);
                        }
                        std::process::exit(1);
                    } else {
                        println!("Validation succeeded: {} violations total", result.violations.len());
                        std::process::exit(0);
                    }
                }
                Err(e) => {
                    eprintln!("Parse failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(2);
        }
    }
}
