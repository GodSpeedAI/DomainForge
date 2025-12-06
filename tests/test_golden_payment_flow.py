from sea_dsl import Graph

PAYMENT_DSL = """
Role "Payer"
Role "Payee"

Resource "Money" units

Entity "Alice"
Entity "Bob"

Flow "Money" from "Alice" to "Bob" quantity 10

Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  via: flow "Money"
"""


def test_payment_flow_parity_from_dsl():
    graph = Graph.parse(PAYMENT_DSL)

    assert graph.entity_count() == 2
    assert graph.resource_count() == 1
    assert graph.flow_count() == 1

    flows = graph.all_flows()
    assert len(flows) == 1
    flow = flows[0]
    # Validate flow quantity and resource name (resource API supported by Python binding)
    assert flow.quantity == 10
    alice_id = graph.find_entity_by_name("Alice")
    bob_id = graph.find_entity_by_name("Bob")
    assert alice_id is not None
    assert bob_id is not None
    assert flow.from_id == alice_id
    assert flow.to_id == bob_id
    res = graph.get_resource(flow.resource_id)
    assert res is not None
    assert res.name == "Money"
