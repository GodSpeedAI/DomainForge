//! Opinionated GodSpeed cell-environment profiles: versioned data, not
//! renderer conditionals. `CellIr::from_ast` looks a profile up by id and
//! seeds every collection from it before overlaying explicit SEA
//! declarations (see `ir.rs`).

/// A pinned tool/runtime default (`mise install`-able name -> version).
#[derive(Debug, Clone, Copy)]
pub struct PinnedTool {
    pub name: &'static str,
    pub version: &'static str,
}

/// A default ecosystem package with no required version (Devbox resolves
/// `name@latest`).
#[derive(Debug, Clone, Copy)]
pub struct SystemDep {
    pub name: &'static str,
    pub version: Option<&'static str>,
}

/// The native dependency-closure contract a profile assumes by default.
#[derive(Debug, Clone, Copy)]
pub struct DependencyContract {
    pub ecosystem: &'static str,
    pub manifest: &'static str,
    pub lockfile: &'static str,
    pub install: &'static str,
}

#[derive(Debug, Clone)]
pub struct CellProfileData {
    pub id: &'static str,
    pub version: &'static str,
    pub system_dependencies: Vec<SystemDep>,
    pub runtimes: Vec<PinnedTool>,
    pub tools: Vec<PinnedTool>,
    pub dependency_contract: Option<DependencyContract>,
}

// `Vec::new()` is not const, so the profile bodies are built by functions
// rather than const items (the alternative — const arrays converted with
// `.to_vec()` at lookup time — reads worse for four profiles this small).

fn python_agent_v1() -> CellProfileData {
    CellProfileData {
        id: "python-agent-v1",
        version: "1.0.0",
        system_dependencies: vec![
            SystemDep {
                name: "git",
                version: None,
            },
            SystemDep {
                name: "ca-certificates",
                version: None,
            },
            SystemDep {
                name: "openssl",
                version: None,
            },
            SystemDep {
                name: "pkg-config",
                version: None,
            },
            SystemDep {
                name: "curl",
                version: None,
            },
        ],
        runtimes: vec![PinnedTool {
            name: "python",
            version: "3.13",
        }],
        tools: vec![
            PinnedTool {
                name: "uv",
                version: "0.9.8",
            },
            PinnedTool {
                name: "just",
                version: "1.43.0",
            },
        ],
        dependency_contract: Some(DependencyContract {
            ecosystem: "python",
            manifest: "pyproject.toml",
            lockfile: "uv.lock",
            install: "uv sync --frozen",
        }),
    }
}

fn typescript_agent_v1() -> CellProfileData {
    CellProfileData {
        id: "typescript-agent-v1",
        version: "1.0.0",
        system_dependencies: vec![
            SystemDep {
                name: "git",
                version: None,
            },
            SystemDep {
                name: "ca-certificates",
                version: None,
            },
            SystemDep {
                name: "openssl",
                version: None,
            },
            SystemDep {
                name: "curl",
                version: None,
            },
        ],
        runtimes: vec![PinnedTool {
            name: "node",
            version: "24",
        }],
        tools: vec![
            PinnedTool {
                name: "pnpm",
                version: "9.15.0",
            },
            PinnedTool {
                name: "just",
                version: "1.43.0",
            },
        ],
        dependency_contract: Some(DependencyContract {
            ecosystem: "typescript",
            manifest: "package.json",
            lockfile: "pnpm-lock.yaml",
            install: "pnpm install --frozen-lockfile",
        }),
    }
}

fn rust_agent_v1() -> CellProfileData {
    CellProfileData {
        id: "rust-agent-v1",
        version: "1.0.0",
        system_dependencies: vec![
            SystemDep {
                name: "git",
                version: None,
            },
            SystemDep {
                name: "ca-certificates",
                version: None,
            },
            SystemDep {
                name: "openssl",
                version: None,
            },
            SystemDep {
                name: "pkg-config",
                version: None,
            },
            SystemDep {
                name: "clang",
                version: None,
            },
            SystemDep {
                name: "lld",
                version: None,
            },
        ],
        runtimes: vec![PinnedTool {
            name: "rust",
            version: "1.92.0",
        }],
        tools: vec![
            PinnedTool {
                name: "cargo-nextest",
                version: "0.9.100",
            },
            PinnedTool {
                name: "just",
                version: "1.43.0",
            },
        ],
        dependency_contract: Some(DependencyContract {
            ecosystem: "rust",
            manifest: "Cargo.toml",
            lockfile: "Cargo.lock",
            install: "cargo fetch --locked",
        }),
    }
}

/// Deterministic union of python/typescript/rust: dedupe by (category, name);
/// on a version disagreement within the union the higher pinned version wins
/// (profile-data composition, not user-facing override merging).
fn polyglot_agent_v1() -> CellProfileData {
    fn merge_tools(mut acc: Vec<PinnedTool>, incoming: Vec<PinnedTool>) -> Vec<PinnedTool> {
        for tool in incoming {
            match acc.iter_mut().find(|t| t.name == tool.name) {
                Some(existing) if version_key(tool.version) > version_key(existing.version) => {
                    existing.version = tool.version;
                }
                Some(_) => {}
                None => acc.push(tool),
            }
        }
        acc
    }
    fn merge_deps(mut acc: Vec<SystemDep>, incoming: Vec<SystemDep>) -> Vec<SystemDep> {
        for dep in incoming {
            if !acc.iter().any(|d| d.name == dep.name) {
                acc.push(dep);
            }
        }
        acc
    }

    let (py, ts, rs) = (python_agent_v1(), typescript_agent_v1(), rust_agent_v1());
    let system_dependencies = merge_deps(
        merge_deps(py.system_dependencies, ts.system_dependencies),
        rs.system_dependencies,
    );
    let runtimes = merge_tools(merge_tools(py.runtimes, ts.runtimes), rs.runtimes);
    let tools = merge_tools(merge_tools(py.tools, ts.tools), rs.tools);

    CellProfileData {
        id: "polyglot-agent-v1",
        version: "1.0.0",
        system_dependencies,
        runtimes,
        tools,
        // A polyglot cell has no single default dependency contract: each
        // ecosystem's DependencySet must be declared explicitly in .sea.
        dependency_contract: None,
    }
}

/// Sort key for simple `major.minor.patch`-ish dotted-numeric version
/// strings; non-numeric segments (e.g. a pre-release suffix) sort as 0 so
/// the comparison never panics on unexpected input.
fn version_key(v: &str) -> Vec<u64> {
    v.split('.')
        .map(|seg| {
            seg.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .map(|digits| digits.parse::<u64>().unwrap_or(0))
        .collect()
}

/// Look up a profile by id. Returns `None` for unknown profiles; callers
/// turn that into `CELL001`.
pub fn lookup(profile_id: &str) -> Option<CellProfileData> {
    match profile_id {
        "python-agent-v1" => Some(python_agent_v1()),
        "typescript-agent-v1" => Some(typescript_agent_v1()),
        "rust-agent-v1" => Some(rust_agent_v1()),
        "polyglot-agent-v1" => Some(polyglot_agent_v1()),
        _ => None,
    }
}

pub const KNOWN_PROFILES: &[&str] = &[
    "python-agent-v1",
    "typescript-agent-v1",
    "rust-agent-v1",
    "polyglot-agent-v1",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_known_profiles_resolve() {
        for id in KNOWN_PROFILES {
            assert!(lookup(id).is_some(), "profile {id} should resolve");
        }
    }

    #[test]
    fn unknown_profile_is_none() {
        assert!(lookup("nonexistent-v1").is_none());
    }

    #[test]
    fn polyglot_union_has_no_duplicate_names() {
        let p = polyglot_agent_v1();
        let mut names: Vec<&str> = p.system_dependencies.iter().map(|d| d.name).collect();
        let before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            before,
            "system dependency union must be deduped"
        );

        let mut tool_names: Vec<&str> = p.tools.iter().map(|t| t.name).collect();
        let before = tool_names.len();
        tool_names.sort();
        tool_names.dedup();
        assert_eq!(tool_names.len(), before, "tool union must be deduped");
    }

    #[test]
    fn polyglot_union_includes_all_three_runtimes() {
        let p = polyglot_agent_v1();
        let names: Vec<&str> = p.runtimes.iter().map(|r| r.name).collect();
        assert!(names.contains(&"python"));
        assert!(names.contains(&"node"));
        assert!(names.contains(&"rust"));
    }
}
