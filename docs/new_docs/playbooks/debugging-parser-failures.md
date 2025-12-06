# Debugging Parser Failures

When the parser fails to process a `.sea` file, follow these steps to diagnose the issue.

## 1. Read the Error Message

Pest provides detailed error locations.
```text
Error:   --> 3:12
  |
3 | entity Foo { type = "service"
  |            ^--- expected "}"
```
*Interpretation*: Missing closing brace.

## 2. Common Syntax Mistakes

- **Missing Quotes**: Strings must be double-quoted. `type = service` is wrong; `type = "service"` is right.
- **Trailing Commas**: SEA does not use commas between properties. Use newlines.
- **Keywords**: Ensure you aren't using a reserved keyword as an identifier.

## 3. Using the Pest Debugger

If you are modifying the grammar:

1. Go to [pest.rs](https://pest.rs/).
2. Paste the content of `sea-core/grammar/sea.pest` into the grammar box.
3. Paste your failing input into the input box.
4. Watch the rule matching visualization to see where it diverges.

## 4. Reporting Grammar Bugs

If valid syntax is rejected:
1. Create a minimal reproduction file (`repro.sea`).
2. Open an issue with the file content and the error output.
3. Tag with `area/parser`.
