#!/usr/bin/env python3
import re
import sys


def fix_entity_new(content):
    """Fix Entity::new(name, namespace) -> Entity::new_with_namespace(name, namespace)"""
    pattern = r"Entity::new\(([^,]+),\s*([^)]+)\)"
    replacement = r"Entity::new_with_namespace(\1, \2)"
    return re.sub(pattern, replacement, content)


def fix_resource_new(content):
    """Fix Resource::new(name, unit_str, namespace) -> Resource::new_with_namespace(name, unit_from_string(unit_str), namespace)"""
    pattern = r'Resource::new\(([^,]+),\s*"([^"]+)",\s*([^)]+)\)'
    replacement = r'Resource::new_with_namespace(\1, unit_from_string("\2"), \3)'
    return re.sub(pattern, replacement, content)


def add_unit_import(content):
    """Add unit_from_string import if not present"""
    if "unit_from_string" not in content:
        # Find the imports section and add it
        import_pattern = r"(use sea_core::primitives::\{[^}]+)\}"
        if re.search(import_pattern, content):
            content = re.sub(import_pattern, r"\1, unit_from_string}", content)
    return content


def main():
    if len(sys.argv) != 2:
        print("Usage: fix_api.py <filename>")
        sys.exit(1)

    filename = sys.argv[1]

    with open(filename, "r") as f:
        content = f.read()

    # Apply fixes
    content = fix_entity_new(content)
    content = fix_resource_new(content)
    content = add_unit_import(content)

    with open(filename, "w") as f:
        f.write(content)

    print(f"Fixed {filename}")


if __name__ == "__main__":
    main()
