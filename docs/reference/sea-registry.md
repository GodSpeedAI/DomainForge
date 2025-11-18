# `.sea-registry.toml`

The SEA CLI now understands workspaces that organize models across multiple
logical namespaces. A workspace declares its layout in a `.sea-registry.toml`
file placed at the repository root. The registry assigns namespaces to groups of
files using glob patterns, ensuring that files without explicit `in <namespace>`
clauses still receive deterministic identifiers.

## File Format

```toml
version = 1
# Optional fallback used when a file does not match any explicit pattern.
default_namespace = "default"

[[namespaces]]
namespace = "logistics"
patterns = ["examples/namespaces/logistics/**/*.sea"]

[[namespaces]]
namespace = "finance"
patterns = ["examples/namespaces/finance/**/*.sea"]
```

* `version`: currently only `1` is supported.
* `default_namespace`: fallback applied when a file is outside every rule.
* `namespaces`: each entry declares the namespace label along with one or more
  glob `patterns` relative to the registry file.

The registry loader validates the file at startup. Invalid glob patterns, empty
rule sets, and files that match multiple namespaces cause descriptive errors so
broken configurations fail fast.

## CLI Workflow

1. Place `.sea-registry.toml` at the workspace root.
2. Organize SEA source files underneath the directories referenced by each
   namespace's glob patterns.
3. Run the CLI from any subdirectory:

```bash
cargo run --bin sea -- validate .
```

* If the CLI receives a directory, it loads the registry, expands all matching
  `.sea` files, assigns namespaces according to the glob that matched, and
  merges the resulting graphs before running validation.
* If the CLI receives a single file, it walks up the directory tree looking for
  `.sea-registry.toml` and uses the namespace rule that matches the file as the
  default namespace during parsing.

The repo ships with a registry and sample files under `examples/namespaces` that
illustrate the workflow.
