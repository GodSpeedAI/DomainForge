# Project Domain Code

Generate a complete, ready-to-use DDD/CQRS domain layer — folders and files,
all domain code generated — in Python, TypeScript, or Rust, covering
everything up to the **port / abstract-base boundary** (no infrastructure
adapters).

All three targets share one language-neutral Domain IR
(`projection/domain/ir.rs`); each renderer translates IR → language syntax
and nothing else. Output is byte-identical run-to-run for a fixed
`--created-at`.

## Commands

```bash
# Python package (src-layout; stdlib only)
domainforge project model.sea --format domain-python --output out_py \
  --created-at 2026-07-02T00:00:00+00:00

# TypeScript package (zero runtime deps; strict)
domainforge project model.sea --format domain-typescript --output out_ts \
  --created-at 2026-07-02T00:00:00+00:00

# Rust crate (zero deps)
domainforge project model.sea --format domain-rust --output out_rs \
  --created-at 2026-07-02T00:00:00+00:00
```

Validate the output with the native toolchain:

| Target | Validation |
| --- | --- |
| Python | `python -m compileall src tests && mypy --strict src && python -m unittest discover tests` |
| TypeScript | `tsc --noEmit` |
| Rust | `cargo check && cargo test` |

## Scaffolded file tree

Each target has one file per IR collection (`aggregates`, `commands`,
`events`, `entities`, `errors`, `roles`, `value_objects`), a `ports` module
(repository/bus/read-model interfaces), a `container` composition root, and a
smoke test — nothing else. Trees below are the actual output for the
`fixtures/projection_cell/basic/model.sea` fixture.

### Python (`domain-python`, 17 files)

```
out_py/
├── README.md
├── pyproject.toml
├── src/procurement_domain/
│   ├── __init__.py
│   ├── container.py
│   ├── domain/
│   │   ├── __init__.py
│   │   ├── aggregates.py
│   │   ├── commands.py
│   │   ├── entities.py
│   │   ├── errors.py
│   │   ├── events.py
│   │   ├── roles.py
│   │   └── value_objects.py
│   └── ports/
│       ├── __init__.py
│       ├── bus.py
│       ├── read_model.py
│       └── repositories.py
└── tests/
    └── test_domain_smoke.py
```

### TypeScript (`domain-typescript`, 16 files)

```
out_ts/
├── README.md
├── package.json
├── tsconfig.json
└── src/
    ├── index.ts
    ├── container.ts
    ├── smoke.ts
    ├── domain/
    │   ├── aggregates.ts
    │   ├── commands.ts
    │   ├── entities.ts
    │   ├── errors.ts
    │   ├── events.ts
    │   ├── roles.ts
    │   └── valueObjects.ts
    └── ports/
        ├── bus.ts
        ├── readModel.ts
        └── repositories.ts
```

### Rust (`domain-rust`, 13 files)

```
out_rs/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── aggregates.rs
│   ├── commands.rs
│   ├── container.rs
│   ├── entities.rs
│   ├── errors.rs
│   ├── events.rs
│   ├── ports.rs
│   ├── roles.rs
│   └── value_objects.rs
└── tests/
    └── smoke.rs
```

## SEA → DDD mapping (normative)

All semantics are decided once in the Domain IR; every renderer expresses the
same mapping.

| SEA element | Domain construct |
| --- | --- |
| `@namespace` | Package/module name (`<ns>_domain`) |
| `Entity` | DDD Entity (identity `id` + `name`; attributes from `instance of`) |
| `Resource` | Aggregate root + Repository port + the resource's unit on the shared `Quantity` value object |
| `Flow` (default) | Command `Transfer<Resource>From<From>To<To>` + Event `<Resource>TransferredFrom<From>To<To>` + aggregate method `transfer_to_<to>` |
| `Flow @cqrs { "kind": "command" }` | Command + aggregate method only (no event) |
| `Flow @cqrs { "kind": "event" }` | Event only (no command, no method) |
| `Policy` | Domain error `<Pascal(policy)>Violation` + a no-op guard hook `check_<policy>()` called by each method (the `@rationale` becomes the docstring; the expression is not compiled to code) |
| `Metric` | Query `get_<metric>()` on a `<Ns>ReadModel` port |
| `Pattern` | Value object `<Pascal>` wrapping a string, validated against the regex (raises `Invalid<Pascal>`) |
| `Role` | Enum `Role`; commands carry an `issued_by: Role` field |
| `Relation` | Doc-comment on the Role enum |
| `instance` | Example construction in the generated smoke test |
| Dependency injection | A `Container` composition-root holding every port |

## The `@cqrs` annotation switch

A flow's `@cqrs` annotation controls which CQRS constructs it mints:

```sea
// Default: command + event + aggregate method.
Flow "PurchaseOrder" from "Buyer" to "Supplier" quantity 1

// Command-side only (no event published).
Flow "Payment" @cqrs { "kind": "command" } from "Buyer" to "Supplier" quantity 100

// Event-side only (an externally-observed fact; no command, no method).
Flow "Invoice" @cqrs { "kind": "event" } from "Supplier" to "Buyer" quantity 1
```

## What you still write by hand

The generated package stops at the port boundary. To run the domain you
provide:

- **Policy predicate bodies** — each guard hook (`check_<policy>()`) is a
  no-op stub returning `Ok`/`None`; fill in the actual predicate from the
  `@rationale` docstring.
- **Infrastructure adapters** — concrete repository, event-bus, command-bus,
  and read-model implementations behind the generated ports.
- **The Rust pattern-VO regex** — the Rust renderer validates non-empty only
  (zero-dep constraint); add the `regex` crate to enforce the full pattern.

## Guardrails

- Generated packages have **zero runtime dependencies** (stdlib only) and
  build offline.
- All identifiers go through `projection/ids.rs`; all flow reads through
  `projection/flows.rs`.
- The fixture `fixtures/projection_cell/basic/model.sea` is never modified by
  these targets.
