import pytest
import domainforge


def test_graph_creation():
    graph = domainforge.Graph()
    assert graph.entity_count() == 0
    assert graph.resource_count() == 0
    assert graph.flow_count() == 0


def test_graph_add_entity():
    graph = domainforge.Graph()
    entity = domainforge.Entity("Warehouse")
    
    graph.add_entity(entity)
    assert graph.entity_count() == 1
    assert graph.has_entity(entity.id)


def test_graph_add_resource():
    graph = domainforge.Graph()
    resource = domainforge.Resource("Cameras", "units")
    
    graph.add_resource(resource)
    assert graph.resource_count() == 1
    assert graph.has_resource(resource.id)


def test_graph_add_flow():
    graph = domainforge.Graph()
    warehouse = domainforge.Entity("Warehouse")
    factory = domainforge.Entity("Factory")
    cameras = domainforge.Resource("Cameras", "units")
    
    graph.add_entity(warehouse)
    graph.add_entity(factory)
    graph.add_resource(cameras)
    
    flow = domainforge.Flow(cameras.id, warehouse.id, factory.id, 100.0)
    graph.add_flow(flow)
    
    assert graph.flow_count() == 1
    assert graph.has_flow(flow.id)


def test_graph_flow_validation():
    graph = domainforge.Graph()
    warehouse = domainforge.Entity("Warehouse")
    factory = domainforge.Entity("Factory")
    cameras = domainforge.Resource("Cameras", "units")
    
    graph.add_entity(warehouse)
    graph.add_resource(cameras)
    
    flow = domainforge.Flow(cameras.id, warehouse.id, factory.id, 100.0)
    
    with pytest.raises(ValueError, match="Target entity not found"):
        graph.add_flow(flow)


def test_graph_get_entity():
    graph = domainforge.Graph()
    entity = domainforge.Entity("Warehouse")
    graph.add_entity(entity)
    
    retrieved = graph.get_entity(entity.id)
    assert retrieved is not None
    assert retrieved.id == entity.id
    assert retrieved.name == entity.name


def test_graph_find_entity_by_name():
    graph = domainforge.Graph()
    entity = domainforge.Entity("Warehouse")
    graph.add_entity(entity)
    
    found_id = graph.find_entity_by_name("Warehouse")
    assert found_id == entity.id
    
    not_found = graph.find_entity_by_name("Nonexistent")
    assert not_found is None


def test_graph_flows_from():
    graph = domainforge.Graph()
    warehouse = domainforge.Entity("Warehouse")
    factory1 = domainforge.Entity("Factory1")
    factory2 = domainforge.Entity("Factory2")
    cameras = domainforge.Resource("Cameras", "units")
    
    graph.add_entity(warehouse)
    graph.add_entity(factory1)
    graph.add_entity(factory2)
    graph.add_resource(cameras)
    
    flow1 = domainforge.Flow(cameras.id, warehouse.id, factory1.id, 100.0)
    flow2 = domainforge.Flow(cameras.id, warehouse.id, factory2.id, 50.0)
    
    graph.add_flow(flow1)
    graph.add_flow(flow2)
    
    flows = graph.flows_from(warehouse.id)
    assert len(flows) == 2


def test_graph_all_methods():
    graph = domainforge.Graph()
    e1 = domainforge.Entity("E1")
    e2 = domainforge.Entity("E2")
    r1 = domainforge.Resource("R1", "units")
    
    graph.add_entity(e1)
    graph.add_entity(e2)
    graph.add_resource(r1)
    
    flow = domainforge.Flow(r1.id, e1.id, e2.id, 10.0)
    graph.add_flow(flow)
    
    all_entities = graph.all_entities()
    assert len(all_entities) == 2
    
    all_resources = graph.all_resources()
    assert len(all_resources) == 1
    
    all_flows = graph.all_flows()
    assert len(all_flows) == 1


def test_graph_exposes_parsed_declarations_in_source_order():
    source = '''
    @namespace "accessors"
    Dimension "Length"
    Unit "m" of "Length" factor 1 base "m"
    Entity "Warehouse"
    Pattern "WarehouseCode" matches "^[A-Z]+$"
    ConceptChange "warehouse_v2" @from_version v1.0.0 @to_version v2.0.0 @migration_policy mandatory @breaking_change true
    instance warehouse_1 of "Warehouse" { code: "WH" }
    Policy stock_positive per Constraint Obligation priority 3 as: 1 > 0
    Metric "stock_count" as: 1 @unit "items"
    Mapping "warehouse_calm" for calm { Entity "Warehouse" -> component { name: "warehouse" } }
    Projection "warehouse_kg" for kg { Entity "Warehouse" { label: "Warehouse" } }
    '''
    graph = domainforge.Graph.parse(source)

    assert [policy.name for policy in graph.all_policies()] == ["stock_positive"]
    assert graph.all_policies()[0].expression == "(1.0 > 0.0)"
    assert graph.all_metrics()[0].unit == "items"
    assert graph.all_mappings()[0].target_format == "CALM"
    assert graph.all_projections()[0].target_format == "KG"
    assert graph.all_dimensions()[0].name == "Length"
    assert graph.all_units()[0].base_factor == "1"
    assert graph.all_patterns()[0].regex == "^[A-Z]+$"
    assert graph.all_concept_changes()[0].namespace == "accessors"
    assert graph.all_entity_instances()[0].fields_json == '{"code":"WH"}'
