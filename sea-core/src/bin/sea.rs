use sea_core::parser::{parse_to_graph_with_options, ParseOptions};
use sea_core::{import_kg_turtle, Graph, NamespaceRegistry};
use sea_core::kg_import::ImportError;
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    let command = &args[1];
    match command.as_str() {
        "validate" => {
            let target = match args.get(2) {
                Some(value) => PathBuf::from(value),
                None => {
                    print_usage();
                    std::process::exit(2);
                }
            };
            if let Err(err) = run_validate(&target) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
            if let Err(err) = run_validate(&target) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
        "registry" => {
            // subcommands: list, resolve
            let sub = match args.get(2).map(String::as_str) {
                Some(value) => value,
                None => {
                    print_registry_usage();
                    std::process::exit(2);
                }
            };
            match sub {
                "list" => {
                    let (fail_on_ambiguity, path) = parse_registry_list_args(&args);
                    let path_ref = path.as_deref().unwrap_or_else(|| Path::new("."));
                    if let Err(err) = registry_list(path_ref, fail_on_ambiguity) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
                "resolve" => {
                    let (fail_on_ambiguity, file) = parse_registry_resolve_args(&args);
                    if let Err(err) = registry_resolve(&file, fail_on_ambiguity) {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                }
                _ => {
                    print_registry_usage();
                    std::process::exit(2);
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
                "sbvr" => match sea_core::SbvrModel::from_xmi(&source) {
                    Ok(model) => match model.to_graph() {
                        Ok(graph) => {
                            println!(
                                "Imported SBVR to Graph: entities={} resources={} flows={}",
                                graph.entity_count(),
                                graph.resource_count(),
                                graph.flow_count()
                            );
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
                },
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
                                Err(err_rdf) => match err_rdf {
                                    ImportError::ShaclValidation(msg) => {
                                        eprintln!("{}", msg);
                                        std::process::exit(1);
                                    }
                                    other => {
                                        let err_rdf_msg = other.to_string();
                                        match import_kg_turtle(&source) {
                                            Ok(graph) => {
                                                println!("Imported KG (Turtle) to Graph: entities={} resources={} flows={}", graph.entity_count(), graph.resource_count(), graph.flow_count());
                                                std::process::exit(0);
                                            }
                                            Err(err_turtle) => {
                                                eprintln!("Failed to import KG as RDF/XML: {}; as Turtle: {}", err_rdf_msg, err_turtle);
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                },
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
                                        Err(err_rdf) => match err_rdf {
                                            ImportError::ShaclValidation(msg) => {
                                                eprintln!("{}", msg);
                                                std::process::exit(1);
                                            }
                                            other => {
                                                let err_rdf_msg = other.to_string();
                                                eprintln!("Failed to import KG as Turtle: {}; and as RDF/XML: {}", err_turtle, err_rdf_msg);
                                                std::process::exit(1);
                                            }
                                        },
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
            print_usage();
            std::process::exit(2);
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  sea validate <file-or-directory>");
    eprintln!("  sea registry <subcommand> [args]");
}

fn run_validate(target: &Path) -> Result<(), String> {
    if target.is_dir() {
        validate_directory(target)
    } else {
        validate_file(target)
    }
}

fn validate_file(path: &Path) -> Result<(), String> {
    let source = read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
    let registry = NamespaceRegistry::discover(path).map_err(|e| e.to_string())?;
    let default_namespace = registry
        .as_ref()
        .and_then(|reg| reg.namespace_for(path).map(|ns| ns.to_string()));
    let options = ParseOptions { default_namespace };
    let graph = parse_to_graph_with_options(&source, &options)
        .map_err(|e| format!("Parse failed for {}: {}", path.display(), e))?;
    report_validation(graph)
}

fn validate_directory(path: &Path) -> Result<(), String> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| format!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| {
            format!(
                "No .sea-registry.toml found for {}. Run inside a workspace with a registry file.",
                path.display()
            )
        })?;

    let files = registry
        .resolve_files()
        .map_err(|e| format!("Failed to expand registry: {}", e))?;

    if files.is_empty() {
        return Err(format!(
            "Registry at '{}' did not match any .sea files",
            registry.root().display()
        ));
    }

    let mut graph = Graph::new();
    for binding in files {
        let source = read_to_string(&binding.path)
            .map_err(|e| format!("Failed to read {}: {}", binding.path.display(), e))?;
        let options = ParseOptions {
            default_namespace: Some(binding.namespace.clone()),
        };
        let file_graph = parse_to_graph_with_options(&source, &options)
            .map_err(|e| format!("Parse failed for {}: {}", binding.path.display(), e))?;
        graph
            .extend(file_graph)
            .map_err(|e| format!("Failed to merge {}: {}", binding.path.display(), e))?;
    }

    report_validation(graph)
}

fn report_validation(graph: Graph) -> Result<(), String> {
    let result = graph.validate();
    if result.error_count > 0 {
        println!("Validation failed: {} errors", result.error_count);
        for v in result.violations {
            println!(
                "- [{}] {}: {}",
                match v.severity {
                    sea_core::policy::Severity::Error => "ERROR",
                    sea_core::policy::Severity::Warning => "WARN",
                    sea_core::policy::Severity::Info => "INFO",
                },
                v.policy_name,
                v.message
            );
        }
        Err("Validation errors detected".into())
    } else {
        println!(
            "Validation succeeded: {} violations total",
            result.violations.len()
        );
        Ok(())
    }
}

fn print_registry_usage() {
    eprintln!("Usage:");
    eprintln!("  sea registry list [--fail-on-ambiguity] [<path>]");
    eprintln!("  sea registry resolve [--fail-on-ambiguity] <file>");
}

fn parse_registry_list_args(args: &[String]) -> (bool, Option<PathBuf>) {
    let mut fail_on_ambiguity = false;
    let mut path: Option<PathBuf> = None;

    for arg in args.iter().skip(3) {
        if arg == "--fail-on-ambiguity" {
            fail_on_ambiguity = true;
            continue;
        }

        if arg.starts_with("--") {
            eprintln!("Unknown registry flag: {}", arg);
            print_registry_usage();
            std::process::exit(2);
        }

        if path.is_some() {
            eprintln!("Unexpected positional argument: {}", arg);
            print_registry_usage();
            std::process::exit(2);
        }

        path = Some(PathBuf::from(arg));
    }

    (fail_on_ambiguity, path)
}

fn parse_registry_resolve_args(args: &[String]) -> (bool, PathBuf) {
    let mut fail_on_ambiguity = false;
    let mut file: Option<PathBuf> = None;

    for arg in args.iter().skip(3) {
        if arg == "--fail-on-ambiguity" {
            fail_on_ambiguity = true;
            continue;
        }

        if arg.starts_with("--") {
            eprintln!("Unknown registry flag: {}", arg);
            print_registry_usage();
            std::process::exit(2);
        }

        if file.is_some() {
            eprintln!("Unexpected positional argument: {}", arg);
            print_registry_usage();
            std::process::exit(2);
        }

        file = Some(PathBuf::from(arg));
    }

    match file {
        Some(path) => (fail_on_ambiguity, path),
        None => {
            eprintln!("Usage: sea registry resolve [--fail-on-ambiguity] <file>");
            print_registry_usage();
            std::process::exit(2);
        }
    }
}

fn registry_list(path: &Path, fail_on_ambiguity: bool) -> Result<(), String> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| format!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| format!("No .sea-registry.toml found for {}", path.display()))?;

    let files = registry
        .resolve_files_with_options(fail_on_ambiguity)
        .map_err(|e| format!("Failed to expand registry: {}", e))?;
    for binding in files {
        println!("{} => {}", binding.path.display(), binding.namespace);
    }
    Ok(())
}

fn registry_resolve(path: &Path, fail_on_ambiguity: bool) -> Result<(), String> {
    let registry = NamespaceRegistry::discover(path)
        .map_err(|e| format!("Failed to load registry near {}: {}", path.display(), e))?
        .ok_or_else(|| format!("No .sea-registry.toml found for {}", path.display()))?;

    let ns = match registry.namespace_for_with_options(path, fail_on_ambiguity) {
        Ok(s) => s.to_string(),
        Err(e) => return Err(e.to_string()),
    };
    println!("{} => {}", path.display(), ns);
    Ok(())
}
