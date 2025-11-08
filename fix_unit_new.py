import re
import sys
from pathlib import Path

def fix_unit_new(content):
    # Pattern to match Unit::new(...) that doesn't already have .unwrap() or .expect()
    pattern = r'Unit::new\(([^)]+(?:\([^)]*\))?[^)]*)\)(?!\.(?:unwrap|expect))'
    
    def replacer(match):
        return f'Unit::new({match.group(1)}).unwrap()'
    
    return re.sub(pattern, replacer, content)

# Find all test files
test_dir = Path('sea-core/tests')
for test_file in test_dir.glob('**/*.rs'):
    content = test_file.read_text()
    fixed = fix_unit_new(content)
    if fixed != content:
        test_file.write_text(fixed)
        print(f'Fixed {test_file}')
