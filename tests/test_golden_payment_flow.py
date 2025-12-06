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
    assert graph.role_count() == 2
    assert graph.relation_count() == 1

    roles = {role.name: role for role in graph.all_roles()}
    assert set(roles.keys()) == {"Payer", "Payee"}

    relations = graph.all_relations()
    assert len(relations) == 1

    relation = relations[0]
    assert relation.name == "Payment"
    assert relation.predicate == "pays"
    assert relation.subject_role_id == roles["Payer"].id
    assert relation.object_role_id == roles["Payee"].id
