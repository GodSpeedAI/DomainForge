import textwrap

import sea_dsl


def test_registry_discover_and_resolve(tmp_path):
    # Skip this test if runtime package isn't built with python FFI enabled
    if not hasattr(sea_dsl, "NamespaceRegistry"):
        return
    base = tmp_path
    domains = base / "domains" / "logistics"
    domains.mkdir(parents=True)
    file_path = domains / "warehouse.sea"
    file_path.write_text('Entity "Warehouse"')

    registry_path = base / ".sea-registry.toml"
    registry_content = textwrap.dedent("""\
        version = 1\n
        [[namespaces]]\n
        namespace = "logistics"\n
        patterns = ["domains/logistics/**/*.sea"]\n
    """)
    registry_path.write_text(registry_content)

    reg = sea_dsl.NamespaceRegistry.from_file(str(registry_path))
    assert reg is not None
    files = reg.resolve_files()
    assert len(files) == 1
    assert files[0].path.endswith("warehouse.sea")
    assert files[0].namespace == "logistics"
    # fail_on_ambiguity for trivial case should still succeed (no ambiguity)
    files = reg.resolve_files(False)
    assert len(files) == 1


def test_registry_precedence_long_and_tie(tmp_path):
    # Skip if not installed with FFI bindings
    if not hasattr(__import__("sea_dsl"), "NamespaceRegistry"):
        return
    base = tmp_path
    domains = base / "domains" / "logistics"
    domains.mkdir(parents=True)
    file_path = domains / "warehouse.sea"
    file_path.write_text('Entity "Warehouse"')

    # long prefix precedence
    registry_path = base / ".sea-registry.toml"
    registry_content = (
        "version = 1\n"
        'default_namespace = "default"\n\n'
        '[[namespaces]]\nnamespace = "short"\npatterns = ["domains/**/*.sea"]\n\n'
        '[[namespaces]]\nnamespace = "long"\npatterns = ["domains/logistics/**/*.sea"]\n'
    )
    registry_path.write_text(registry_content)
    reg = sea_dsl.NamespaceRegistry.from_file(str(registry_path))
    assert reg.namespace_for(str(file_path)) == "long"

    # alphabetical tie-breaker
    registry_content = (
        "version = 1\n"
        'default_namespace = "default"\n\n'
        '[[namespaces]]\nnamespace = "logistics"\npatterns = ["domains/*/warehouse.sea"]\n\n'
        '[[namespaces]]\nnamespace = "finance"\npatterns = ["domains/*/warehouse.sea"]\n'
    )
    registry_path.write_text(registry_content)
    reg = sea_dsl.NamespaceRegistry.from_file(str(registry_path))
    assert reg.namespace_for(str(file_path)) == "finance"

    # When asking to fail on ambiguity, an exception should be raised
    try:
        reg.namespace_for(str(file_path), True)
        assert False, "Expected namespace_for to raise on ambiguity"
    except Exception:
        pass
    # resolve_files should raise when fail_on_ambiguity True
    try:
        reg.resolve_files(True)
        assert False, "Expected resolve_files to raise on ambiguity"
    except Exception:
        pass
