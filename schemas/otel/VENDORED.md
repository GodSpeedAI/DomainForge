# OTel SemConv registry validation schema

`semconv_registry.schema.json` is a **DomainForge-authored** JSON Schema (Draft-07),
not a vendored upstream artifact. It describes the fixed subset of the
OpenTelemetry semantic-convention registry format that
`domainforge project --format otel-semconv` emits: groups of
`type: attribute_group` carrying attribute definitions, and groups of
`type: span` referencing them.

## Why not `weaver registry check`?

The plan (Task 6) allows substituting a JSON-Schema validation of the emitted
YAML for OpenTelemetry's official `weaver` checker when a `weaver` install is
too heavy for CI. `weaver` pulls a large Rust binary / container and its
registry schema has churned across releases; for the small, closed shape this
projection produces, a self-contained JSON-Schema gate is lighter and stable.

The `verify-otel` CI job therefore parses the emitted `registry/telemetry.yaml`
and validates it against this schema (PyYAML + `jsonschema`). Bumping to real
`weaver` later is a drop-in: point the CI step at `weaver registry check`
instead, keeping this schema as the fast local gate.
