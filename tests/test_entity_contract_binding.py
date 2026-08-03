import json

import domainforge


def test_python_graph_exposes_typed_entity_contract_json():
    graph = domainforge.Graph.parse(
        '@namespace "binding"\nentity "Item" { key item_id: string value: int }'
    )
    entity_id = graph.find_entity_by_name("Item")

    contract = json.loads(graph.entity_contract_json(entity_id))
    assert contract["key_field"] == "item_id"
    assert [field["name"] for field in contract["fields"]] == ["item_id", "value"]


def test_python_graph_keeps_bodyless_entity_contract_absent():
    graph = domainforge.Graph.parse('entity "Legacy"')
    assert graph.entity_contract_json(graph.find_entity_by_name("Legacy")) is None
