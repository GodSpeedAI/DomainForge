use std::env;
use std::fs::read_to_string;
use sea_core::parser::parse_to_graph;
use sea_core::import_kg_turtle;

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
        "import" => {
            if args.len() < 5 || args[2] != "--format" {
                eprintln!("Usage: sea import --format <sbvr|kg> <file>");
                std::process::exit(2);
            }
            let format = &args[3];
            let filepath = &args[4];
            let source = match read_to_string(filepath) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", filepath, e);
                    std::process::exit(1);
                }
            };

            match format.as_str() {
                "sbvr" => {
                    match sea_core::SbvrModel::from_xmi(&source) {
                        Ok(model) => match model.to_graph() {
                            Ok(graph) => {
                                println!("Imported SBVR to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                std::process::exit(0);
                            }
                            Err(e) => {
                                eprintln!("Failed to convert SBVR to Graph: {}", e);
                                std::process::exit(1);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to parse SBVR XMI: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                "kg" => {
                    // If the source looks like XML (starts with <), prefer RDF/XML import path
                    let src_trim = source.trim_start();
                    if src_trim.starts_with('<') {
                        #[cfg(feature = "shacl")]
                        {
                            match sea_core::import_kg_rdfxml(&source) {
                                Ok(graph) => {
                                    println!("Imported KG (RDF/XML) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                    std::process::exit(0);
                                }
                                Err(err_rdf) => {
                                    // If SHACL validation failed, treat as a hard error and do not fall back.
                                    if err_rdf.contains("SHACL validation failed") {
                                        eprintln!("{}", err_rdf);
                                        std::process::exit(1);
                                    }
                                    // Fallback: try Turtle if RDF/XML import failed for parsing/format reasons
                                    match import_kg_turtle(&source) {
                                        Ok(graph) => {
                                            println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                            std::process::exit(0);
                                        }
                                        Err(err_turtle) => {
                                            eprintln!("Failed to import KG as RDF/XML: {}; as Turtle: {}", err_rdf, err_turtle);
                                            std::process::exit(1);
                                        }
                                    }
                                }
                            }
                        }
                        #[cfg(not(feature = "shacl"))]
                        {
                            // Without shacl feature we can't parse RDF/XML reliably, so attempt Turtle
                            match import_kg_turtle(&source) {
                                Ok(graph) => {
                                    println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                    std::process::exit(0);
                                }
                                Err(err) => {
                                    eprintln!("Failed to import KG: {}", err);
                                    std::process::exit(1);
                                }
                            }
                        }
                    } else {
                        // Treat as Turtle by default
                        match import_kg_turtle(&source) {
                            Ok(graph) => {
                                println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                std::process::exit(0);
                            }
                            Err(err_turtle) => {
                                // If shacl feature enabled, attempt RDF/XML import as fallback
                                #[cfg(feature = "shacl")]
                                {
                                    match sea_core::import_kg_rdfxml(&source) {
                                        Ok(graph) => {
                                            println!("Imported KG (RDF/XML) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                            std::process::exit(0);
                                        }
                                        Err(err_rdf) => {
                                            eprintln!("Failed to import KG as Turtle: {}; and as RDF/XML: {}", err_turtle, err_rdf);
                                            std::process::exit(1);
                                        }
                                    }
                                }
                                #[cfg(not(feature = "shacl"))]
                                {
                                    eprintln!("Failed to import KG (Turtle): {}. To try RDF/XML, enable feature 'shacl' in the build.", err_turtle);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                }
                _ => {
                    eprintln!("Unsupported format: {}", format);
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(2);
        }
    }
}
