from typing import Any, List, Optional

__version__: str

class Entity:
    """Represents a business entity (WHO) - actors, locations, or organizational units."""

    id: str
    name: str
    namespace: Optional[str]

    def __init__(self, name: str, namespace: Optional[str] = None) -> None:
        """Create a new Entity with the given name and optional namespace."""
        ...

    def set_attribute(self, key: str, value: Any) -> None:
        """Set a custom attribute on the entity."""
        ...

    def get_attribute(self, key: str) -> Any:
        """Get a custom attribute from the entity. Raises KeyError if not found."""
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Resource:
    """Represents a quantifiable resource (WHAT) measured in specific units."""

    id: str
    name: str
    unit: str
    namespace: Optional[str]

    def __init__(self, name: str, unit: str, namespace: Optional[str] = None) -> None:
        """Create a new Resource with the given name, unit, and optional namespace."""
        ...

    def set_attribute(self, key: str, value: Any) -> None:
        """Set a custom attribute on the resource."""
        ...

    def get_attribute(self, key: str) -> Any:
        """Get a custom attribute from the resource. Raises KeyError if not found."""
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

class Instance:
    """Represents a physical instance of a resource at a specific entity location."""

    id: str
    resource_id: str
    entity_id: str
    namespace: Optional[str]

    def __init__(self, resource_id: str, entity_id: str, namespace: Optional[str] = None) -> None:
        """Create a new Instance with the given resource ID, entity ID, and optional namespace."""
        ...

    def set_attribute(self, key: str, value: Any) -> None:
        """Set a custom attribute on the instance."""
        ...

    def get_attribute(self, key: str) -> Any:
        """Get a custom attribute from the instance. Raises KeyError if not found."""
        ...

    def __repr__(self) -> str: ...

class Flow:
    """Represents the movement of a resource from one entity to another."""

    id: str
    resource_id: str
    from_id: str
    to_id: str
    quantity: float
    namespace: Optional[str]

    def __init__(self, resource_id: str, from_id: str, to_id: str, quantity: float, namespace: Optional[str] = None) -> None:
        """Create a new Flow with the given resource, source entity, target entity, and quantity."""
        ...

    def set_attribute(self, key: str, value: Any) -> None:
        """Set a custom attribute on the flow."""
        ...

    def get_attribute(self, key: str) -> Any:
        """Get a custom attribute from the flow. Raises KeyError if not found."""
        ...

    def __repr__(self) -> str: ...

class Graph:
    """Container for entities, resources, and flows with validation and query capabilities."""

    def __init__(self) -> None:
        """Create a new empty Graph."""
        ...

    def add_entity(self, entity: Entity) -> None:
        """Add an entity to the graph. Raises ValueError if entity already exists."""
        ...

    def add_resource(self, resource: Resource) -> None:
        """Add a resource to the graph. Raises ValueError if resource already exists."""
        ...

    def add_flow(self, flow: Flow) -> None:
        """
        Add a flow to the graph. Raises ValueError if:
        - Flow already exists
        - Source or target entity not found
        - Resource not found
        """
        ...

    def add_instance(self, instance: Instance) -> None:
        """
        Add an instance to the graph. Raises ValueError if:
        - Instance already exists
        - Resource not found
        - Entity not found
        """
        ...

    def entity_count(self) -> int:
        """Return the number of entities in the graph."""
        ...

    def resource_count(self) -> int:
        """Return the number of resources in the graph."""
        ...

    def flow_count(self) -> int:
        """Return the number of flows in the graph."""
        ...

    def instance_count(self) -> int:
        """Return the number of instances in the graph."""
        ...

    def has_entity(self, id: str) -> bool:
        """Check if an entity with the given ID exists in the graph."""
        ...

    def has_resource(self, id: str) -> bool:
        """Check if a resource with the given ID exists in the graph."""
        ...

    def has_flow(self, id: str) -> bool:
        """Check if a flow with the given ID exists in the graph."""
        ...

    def get_entity(self, id: str) -> Optional[Entity]:
        """Get an entity by ID, or None if not found."""
        ...

    def get_resource(self, id: str) -> Optional[Resource]:
        """Get a resource by ID, or None if not found."""
        ...

    def get_flow(self, id: str) -> Optional[Flow]:
        """Get a flow by ID, or None if not found."""
        ...

    def find_entity_by_name(self, name: str) -> Optional[str]:
        """Find an entity by name and return its ID, or None if not found."""
        ...

    def find_resource_by_name(self, name: str) -> Optional[str]:
        """Find a resource by name and return its ID, or None if not found."""
        ...

    def flows_from(self, entity_id: str) -> List[Flow]:
        """Get all flows originating from the given entity."""
        ...

    def flows_to(self, entity_id: str) -> List[Flow]:
        """Get all flows going to the given entity."""
        ...

    def all_entities(self) -> List[Entity]:
        """Get all entities in the graph."""
        ...

    def all_resources(self) -> List[Resource]:
        """Get all resources in the graph."""
        ...

    def all_flows(self) -> List[Flow]:
        """Get all flows in the graph."""
        ...

    def all_instances(self) -> List[Instance]:
        """Get all instances in the graph."""
        ...

    @staticmethod
    def parse(source: str) -> 'Graph':
        """
        Parse a SEA DSL source string into a Graph.
        Raises ValueError if there are parse errors.
        """
        ...

    def __repr__(self) -> str: ...
