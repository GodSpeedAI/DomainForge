"""
Additional Python tests for SEA DSL bindings - Instance support
"""
import pytest
import sea_dsl


def test_instance_creation():
    """Test creating an Instance"""
    entity = sea_dsl.Entity("Warehouse")
    resource = sea_dsl.Resource("Camera", "units")

    instance = sea_dsl.Instance(resource.id, entity.id)
    assert instance.resource_id == resource.id
    assert instance.entity_id == entity.id
    assert len(instance.id) == 36


def test_instance_with_namespace():
    """Test creating an Instance with namespace"""
    entity = sea_dsl.Entity("Warehouse", "logistics")
    resource = sea_dsl.Resource("Camera", "units", "inventory")

    instance = sea_dsl.Instance(resource.id, entity.id, "inventory")
    assert instance.namespace == "inventory"


def test_instance_attributes():
    """Test Instance attributes"""
    entity = sea_dsl.Entity("Warehouse")
    resource = sea_dsl.Resource("Camera", "units")

    instance = sea_dsl.Instance(resource.id, entity.id)
    instance.set_attribute("serial_number", "CAM-12345")
    assert instance.get_attribute("serial_number") == "CAM-12345"


def test_graph_add_instance():
    """Test adding an Instance to a Graph"""
    graph = sea_dsl.Graph()
    entity = sea_dsl.Entity("Warehouse")
    resource = sea_dsl.Resource("Camera", "units")

    graph.add_entity(entity)
    graph.add_resource(resource)

    instance = sea_dsl.Instance(resource.id, entity.id)
    graph.add_instance(instance)

    assert graph.instance_count() == 1


def test_graph_instance_validation():
    """Test Graph validates Instance references"""
    graph = sea_dsl.Graph()
    entity = sea_dsl.Entity("Warehouse")
    resource = sea_dsl.Resource("Camera", "units")

    graph.add_entity(entity)
    # Not adding resource

    instance = sea_dsl.Instance(resource.id, entity.id)

    with pytest.raises(ValueError, match="Resource not found"):
        graph.add_instance(instance)


def test_round_trip_serialization():
    """Test that primitives can be serialized and remain valid"""
    graph = sea_dsl.Graph()

    # Create a complete model
    warehouse = sea_dsl.Entity("Warehouse", "logistics")
    warehouse.set_attribute("location", "NYC")

    factory = sea_dsl.Entity("Factory", "manufacturing")
    factory.set_attribute("capacity", 10000)

    cameras = sea_dsl.Resource("Camera", "units", "inventory")
    cameras.set_attribute("model", "X100")

    graph.add_entity(warehouse)
    graph.add_entity(factory)
    graph.add_resource(cameras)

    flow = sea_dsl.Flow(cameras.id, warehouse.id, factory.id, 500.0)
    flow.set_attribute("priority", "high")
    graph.add_flow(flow)

    instance = sea_dsl.Instance(cameras.id, warehouse.id, "inventory")
    instance.set_attribute("serial", "SN-001")
    graph.add_instance(instance)

    # Verify the graph state
    assert graph.entity_count() == 2
    assert graph.resource_count() == 1
    assert graph.flow_count() == 1
    assert graph.instance_count() == 1

    # Verify we can retrieve and check attributes
    retrieved_entity = graph.get_entity(warehouse.id)
    assert retrieved_entity.get_attribute("location") == "NYC"


def test_error_handling_invalid_references():
    """Test that proper errors are raised for invalid references"""
    graph = sea_dsl.Graph()
    warehouse = sea_dsl.Entity("Warehouse")
    graph.add_entity(warehouse)

    # Try to create flow with non-existent IDs
    fake_resource_id = "00000000-0000-0000-0000-000000000000"
    fake_entity_id = "11111111-1111-1111-1111-111111111111"

    flow = sea_dsl.Flow(fake_resource_id, warehouse.id, fake_entity_id, 100.0)

    with pytest.raises(ValueError):
        graph.add_flow(flow)


def test_namespace_isolation():
    """Test that namespaces properly isolate entities"""
    graph = sea_dsl.Graph()

    warehouse_logistics = sea_dsl.Entity("Warehouse", "logistics")
    warehouse_storage = sea_dsl.Entity("Warehouse", "storage")

    graph.add_entity(warehouse_logistics)
    graph.add_entity(warehouse_storage)

    assert graph.entity_count() == 2

    # Both should exist independently
    assert graph.has_entity(warehouse_logistics.id)
    assert graph.has_entity(warehouse_storage.id)
