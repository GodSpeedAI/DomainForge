# Create Custom Units

Goal: Define custom dimensions/units and validate quantities using them in SEA models.

- ## Prerequisites

- SEA CLI installed.

   - For developers building from source: `cargo install --path sea-core --features cli` (produces the `sea` binary).
   - For users installing from crates.io or binaries: use the official release package or `cargo install sea-core` (if published), or download the published binary from GitHub Releases.
- Familiarity with resources and flows in the SEA DSL.
- Optional: Python/TypeScript bindings if you want to assert units programmatically.

## Steps (be concise)

1. **Declare dimensions and units in DSL**

   ```sea
   Dimension "Currency"
   Unit "USD" of "Currency" factor 1 base "USD"
   Unit "EUR" of "Currency" factor 1.07 base "USD"
   ```

   - `factor` sets the multiplier relative to the base unit.
   - Always include `base` to avoid ambiguous conversions.

2. **Attach units to resources and flows**

   ```sea
   Resource "Money" unit "USD"
   Entity "Alice"
   Entity "Bob"
   Flow "Money" from "Alice" to "Bob" quantity 100
   ```

   - Resources reference the canonical unit.
   - Flows inherit the resource unit unless explicitly overridden.

3. **Convert units in policies or aggregations**

   ```sea
   Policy euro_cap as:
     forall f in flows where f.resource = "Money":
       f.quantity as "EUR" <= 10000 "EUR"
   ```

   - The `as` clause requests conversion using defined factors.

4. **Create the example file and validate with the CLI**

   Create `examples/custom_units.sea` (this file contains the model used by the following examples):

```bash
mkdir -p examples
cat > examples/custom_units.sea <<'SEA'
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"

Resource "Money" unit "USD"
Entity "Alice"
Entity "Bob"
Flow "Money" from "Alice" to "Bob" quantity 100
SEA
```

   Now validate the file with the CLI:

   ```bash
   sea validate --format human examples/custom_units.sea
   ```

   This validation will fail if a unit or dimension is missing, duplicated, or forms a circular base chain.

5. **Inspect units in Python (re-using examples/custom_units.sea)**

   ```python
   from sea_dsl import Graph

   graph = Graph.parse(open("examples/custom_units.sea").read())
   resource = graph.all_resources()[0]
   assert resource.unit == "USD"
   ```

6. **Inspect units in TypeScript (re-using examples/custom_units.sea)**

   ```ts
   import { Graph } from "@domainforge/sea";
   import { readFileSync } from "fs";

   const graph = Graph.parse(readFileSync("examples/custom_units.sea", "utf8"));
   console.log(graph.allResources()[0].unit);
   ```

## Edge Cases

- **Circular bases**: Avoid `USD` base `EUR` and `EUR` base `USD`; validation will fail with a descriptive error.
- **Precision**: Use decimal notation (`1.07`) to prevent rounding surprises; avoid locale-specific commas.
- **Namespace scoping**: Declare dimensions/units in the same namespace as consuming resources to prevent lookup failures.
- **Unit reuse**: Reusing unit names across dimensions is not allowed; pick unique names or namespace them explicitly.

## Testing and Validation Tips

- Add a small regression test under `tests/` that parses the DSL and asserts `graph.all_resources()[0].unit`.
- When exporting to CALM, confirm that quantities carry the correct `unit` field; run `jq '.models[0].flows[0].quantity' file.calm.json`.
- Use `sea project --format kg` and inspect the Turtle output to ensure unit annotations appear in the KG representation.

## Troubleshooting

- **Unknown unit error**: Define the unit before referencing it in resources/flows.
- **Factor mismatch**: Ensure factors are numeric; strings or missing values cause parse failures.
- **Conversion failures**: If policy evaluation fails due to missing conversion paths, verify the base chain is fully connected.

## Verification Checklist

- [ ] Dimension and unit declarations parse without errors.
- [ ] Resources reference valid units and flows inherit them correctly.
- [ ] Policy conversions using `as "<Unit>"` succeed with expected values.
- [ ] CALM/KG exports include the units and survive round-trip import.

## Links

- Tutorials: [First SEA Model](../tutorials/first-sea-model.md)
- Reference: [Grammar Spec](../reference/grammar-spec.md), [Primitives API](../reference/primitives-api.md)
