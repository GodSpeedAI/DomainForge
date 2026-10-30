# units — Dimensions, Units, and the Unit Registry

## Overview
`Dimension`/`Unit`/`UnitRegistry` implement linear unit conversion over
`rust_decimal::Decimal`, plus a process-wide default registry pre-populated with
builtin units. `try_unit_from_string`/`unit_from_string` are the string-symbol
lookup pair used throughout the parser and primitives.

## Interface
- `[mod.rs:Dimension]` (`+parse`, `FromStr`, `Display`) — `Mass`, `Length`, `Volume`, `Currency`, `Time`, `Temperature`, `Count`, `Custom(String)`.
- `[mod.rs:Unit]` (`+new/new_base/with_base/symbol/name/dimension/base_factor/base_unit`, `Display`) — `symbol`, `name`, `dimension`, `base_factor`, `base_unit`.
- `[mod.rs:UnitConversion]` — trait with `convert_to_base`/`convert_from_base`; implemented for `Unit` as `value * base_factor` / `value / base_factor`.
- `[mod.rs:UnitError]` (`#[non_exhaustive]`) — `UnitNotFound`, `IncompatibleDimensions`, `ConversionNotDefined`, `ZeroBaseFactor`, `DuplicateUnit`.
- `[mod.rs:UnitRegistry]` (`+new/register/register_dimension/register_base/get_unit/units/base_units/convert/global/register_from_json`) — dimension-keyed unit store with builtin defaults.
- `[mod.rs:get_default_registry]` — alias for `UnitRegistry::global`.
- `[mod.rs:try_unit_from_string]` — fallible lookup; `UnitError::UnitNotFound` for unknown symbols.
- `[mod.rs:unit_from_string]` — infallible helper; fabricates a `Dimension::Custom(symbol)` unit for unknown symbols.

## Invariants
- `[mod.rs:UnitRegistry::register]` — rejects a duplicate symbol, a zero `base_factor` (division by zero in `convert_from_base`), a `Dimension::Temperature` unit (the model is purely linear and cannot express affine °C/°F offsets), and a `base_unit` that resolves to an already-registered unit of a *different* dimension. It deliberately does not require `base_unit` to already be registered.
- `[mod.rs:UnitRegistry::register_dimension]` — a documented no-op; it must not insert an empty-string placeholder into `base_units`. `register_base` is the only source of truth for a dimension's base unit.
- `[mod.rs:UnitRegistry::global]` — a process-wide `RwLock<UnitRegistry>` singleton; custom unit registrations persist for the process lifetime and are visible to every later parse in the same process (documented cross-request leak risk for long-lived hosts).
- `[mod.rs:unit_from_string]` — a fabricated unit's dimension is `Custom(symbol)`, not `Count`; two distinct unregistered symbols therefore report `IncompatibleDimensions` on conversion while a symbol converted against itself still works.
- `[mod.rs:UnitRegistry::convert]` — conversion between incompatible dimensions returns `UnitError::IncompatibleDimensions`; between two different `Currency` symbols it returns `UnitError::ConversionNotDefined` (no FX rates are modelled). Same-dimension units are asserted in debug builds to agree on `base_unit`.

## Side Effects
- `[mod.rs:UnitRegistry::global]` — lazily initializes and mutates a process-global `OnceLock<RwLock<UnitRegistry>>`; every `try_unit_from_string` takes a read lock on it, and `register_from_json` mutates it through the caller's write guard.

## Verification
- Run: `cargo test -p domainforge-core --features cli`
