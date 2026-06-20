import pytest
import domainforge


def test_entity_creation():
    entity = domainforge.Entity("Warehouse A")
    assert entity.name == "Warehouse A"
    assert len(entity.id) == 36
    assert entity.namespace is None


def test_entity_with_namespace():
    entity = domainforge.Entity("Factory", "logistics")
    assert entity.name == "Factory"
    assert entity.namespace == "logistics"


def test_entity_attributes():
    entity = domainforge.Entity("Factory")
    entity.set_attribute("capacity", 5000)
    assert entity.get_attribute("capacity") == 5000
    
    entity.set_attribute("location", "New York")
    assert entity.get_attribute("location") == "New York"


def test_entity_attribute_not_found():
    entity = domainforge.Entity("Factory")
    with pytest.raises(KeyError):
        entity.get_attribute("nonexistent")


def test_resource_creation():
    resource = domainforge.Resource("Cameras", "units")
    assert resource.name == "Cameras"
    assert resource.unit == "units"
    assert len(resource.id) == 36
    assert resource.namespace is None


def test_resource_with_namespace():
    resource = domainforge.Resource("Steel", "tons", "materials")
    assert resource.name == "Steel"
    assert resource.unit == "tons"
    assert resource.namespace == "materials"


def test_resource_attributes():
    resource = domainforge.Resource("Steel", "tons")
    resource.set_attribute("grade", "A36")
    assert resource.get_attribute("grade") == "A36"


def test_flow_creation():
    warehouse = domainforge.Entity("Warehouse")
    factory = domainforge.Entity("Factory")
    cameras = domainforge.Resource("Cameras", "units")
    
    flow = domainforge.Flow(cameras.id, warehouse.id, factory.id, 100.0)
    assert flow.resource_id == cameras.id
    assert flow.from_id == warehouse.id
    assert flow.to_id == factory.id
    assert flow.quantity == 100.0


def test_flow_invalid_uuid():
    with pytest.raises(ValueError, match="Invalid.*UUID"):
        domainforge.Flow("invalid-uuid", "also-invalid", "still-invalid", 100.0)
