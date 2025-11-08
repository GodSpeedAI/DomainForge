#!/usr/bin/env python3
import re
from pathlib import Path


def fix_file(filepath):
    content = filepath.read_text()
    original = content

    # Fix: Unit::new(...) -> Unit::new(...).unwrap()
    # But only if not already followed by .unwrap() or .expect()
    content = re.sub(
        r"(Unit::new\([^)]+\))(?!\.(?:unwrap|expect))", r"\1.unwrap()", content
    )

    # Fix broken patterns like Decimal::from(1).unwrap() back to Decimal::from(1)
    content = re.sub(
        r"Decimal::from\((\d+)\)\.unwrap\(\)", r"Decimal::from(\1)", content
    )

    # Fix broken patterns like Decimal::new(x, y).unwrap() back to Decimal::new(x, y)
    content = re.sub(
        r"Decimal::new\(([^)]+)\)\.unwrap\(\)", r"Decimal::new(\1)", content
    )

    if content != original:
        filepath.write_text(content)
        return True
    return False


# Fix all test files
test_dir = Path("sea-core/tests")
fixed_count = 0
for test_file in test_dir.glob("**/*.rs"):
    if fix_file(test_file):
        print(f"Fixed: {test_file}")
        fixed_count += 1

print(f"\nTotal files fixed: {fixed_count}")
