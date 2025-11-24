# Quantity Rendering Specification

## Overview

Quantity rendering in SEA DSL ensures that numerical values with units are displayed correctly according to the user's locale. This includes number formatting (decimal separators, grouping) and unit placement.

## QuantityFormatter

The `QuantityFormatter` struct in `sea-core/src/primitives/quantity.rs` handles the formatting logic. It uses `icu_decimal` for locale-aware number formatting.

### Usage

```rust
use sea_core::primitives::Quantity;
use sea_core::primitives::quantity::QuantityFormatter;
use sea_core::units::Dimension;
use icu_locid::locale;

let quantity = Quantity::new(1500.0, "USD".to_string(), Dimension::Currency);
let formatter = QuantityFormatter::new(locale!("en-US"));
assert_eq!(formatter.format(&quantity), "1,500 \"USD\"");
```

## Supported Locales

The formatter supports all locales provided by `icu_locid`. Common examples:

- `en-US`: 1,500.00
- `de-DE`: 1.500,00
- `fr-FR`: 1 500,00

## Implementation Details

- Uses `fixed_decimal` for high-precision decimal formatting.
- Currently formats the value as a string followed by the unit symbol in quotes.
