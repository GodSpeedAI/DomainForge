from pathlib import Path


def fix_unit_new(content):
    marker = "Unit::new("
    output = []
    cursor = 0

    while True:
        start = content.find(marker, cursor)
        if start == -1:
            output.append(content[cursor:])
            break

        output.append(content[cursor:start])
        depth = 0
        end = None
        index = start + len("Unit::new")

        while index < len(content):
            char = content[index]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    end = index + 1
                    break
            index += 1

        if end is None:
            output.append(content[start:])
            break

        call = content[start:end]
        if content.startswith(".unwrap()", end) or content.startswith(".expect(", end):
            output.append(call)
        else:
            output.append(f"{call}.unwrap()")
        cursor = end

    return "".join(output)


def main():
    test_dir = Path("sea-core/tests")
    for test_file in test_dir.glob("**/*.rs"):
        content = test_file.read_text()
        fixed = fix_unit_new(content)
        if fixed != content:
            test_file.write_text(fixed)
            print(f"Fixed {test_file}")


if __name__ == "__main__":
    main()
