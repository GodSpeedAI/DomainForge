import pytest
from sea_dsl import Role, Relation
import uuid


def test_role_creation():
    role = Role("Approver")
    assert role.name == "Approver"
    assert role.namespace is None  # Default namespace hidden
    assert role.id is not None

    # Check ID format (should be UUID-like or namespaced)
    assert role.id is not None
    # ID should be a UUID string
    uuid.UUID(role.id)


def test_role_with_namespace():
    role = Role("Viewer", namespace="governance")
    assert role.name == "Viewer"
    assert role.namespace == "governance"
    # Verify ID is a valid UUID
    uuid.UUID(role.id)


def test_role_attributes():
    role = Role("Admin")
    role.set_attribute("level", 5)
    role.set_attribute("active", True)

    assert role.get_attribute("level") == 5
    assert role.get_attribute("active") is True

    with pytest.raises(KeyError):
        role.get_attribute("nonexistent")


def test_relation_creation():
    # Roles use concept IDs (namespaced)
    subject = Role("Payer")
    object = Role("Payee")
    flow_uuid = str(uuid.uuid4())

    relation = Relation(
        name="Payment",
        subject_role_id=subject.id,
        predicate="pays",
        object_role_id=object.id,
        via_flow_id=flow_uuid,
    )

    assert relation.name == "Payment"
    assert relation.predicate == "pays"
    assert relation.subject_role_id == subject.id
    assert relation.object_role_id == object.id
    assert relation.via_flow_id == flow_uuid
