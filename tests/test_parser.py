import pytest
import sea_dsl


def test_parse_basic():
    source = '''
        Entity "Warehouse" in logistics
        Entity "Factory" in logistics
        Resource "Cameras" units
    '''
    
    graph = sea_dsl.Graph.parse(source)
    assert graph.entity_count() == 2
    assert graph.resource_count() == 1


def test_parse_with_flow():
    source = '''
        Entity "Warehouse"
        Entity "Factory"
        Resource "Cameras" units
        Flow "Cameras" from "Warehouse" to "Factory" quantity 100
    '''
    
    graph = sea_dsl.Graph.parse(source)
    assert graph.entity_count() == 2
    assert graph.resource_count() == 1
    assert graph.flow_count() == 1


def test_parse_complex():
    source = '''
        Entity "Supplier" in supply_chain
        Entity "Warehouse" in supply_chain
        Entity "Store" in retail
        
        Resource "Widgets" units
        Resource "Gadgets" units
        
        Flow "Widgets" from "Supplier" to "Warehouse" quantity 500
        Flow "Widgets" from "Warehouse" to "Store" quantity 300
        Flow "Gadgets" from "Supplier" to "Warehouse" quantity 200
    '''
    
    graph = sea_dsl.Graph.parse(source)
    assert graph.entity_count() == 3
    assert graph.resource_count() == 2
    assert graph.flow_count() == 3


def test_parse_invalid_syntax():
    source = '''
        Entity "Warehouse"
        Invalid syntax here
    '''
    
    with pytest.raises(ValueError, match="Parse error"):
        sea_dsl.Graph.parse(source)


def test_parse_empty():
    source = ""
    graph = sea_dsl.Graph.parse(source)
    assert graph.entity_count() == 0


def test_parsed_graph_query():
    source = '''
        Entity "Warehouse"
        Entity "Factory"
        Resource "Steel" tons
        Flow "Steel" from "Warehouse" to "Factory" quantity 50
    '''
    
    graph = sea_dsl.Graph.parse(source)
    
    warehouse_id = graph.find_entity_by_name("Warehouse")
    assert warehouse_id is not None
    
    flows = graph.flows_from(warehouse_id)
    assert len(flows) == 1
    assert flows[0].quantity == 50.0
