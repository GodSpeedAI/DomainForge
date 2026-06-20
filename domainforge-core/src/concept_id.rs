use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(Uuid);

impl ConceptId {
    pub fn from_concept(namespace: &str, name: &str) -> Self {
        let namespace_uuid = Self::namespace_uuid();
        let data = format!("{}::{}", namespace, name);
        let uuid = Uuid::new_v5(&namespace_uuid, data.as_bytes());
        ConceptId(uuid)
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        ConceptId(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn from_legacy_uuid(uuid: Uuid) -> Self {
        ConceptId(uuid)
    }

    fn namespace_uuid() -> Uuid {
        Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap()
    }
}

impl std::fmt::Display for ConceptId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ConceptId {
    fn from(uuid: Uuid) -> Self {
        ConceptId(uuid)
    }
}

impl AsRef<Uuid> for ConceptId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_id_deterministic() {
        let id1 = ConceptId::from_concept("logistics", "Warehouse");
        let id2 = ConceptId::from_concept("logistics", "Warehouse");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_concept_id_different_namespace() {
        let id1 = ConceptId::from_concept("logistics", "Camera");
        let id2 = ConceptId::from_concept("finance", "Camera");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_concept_id_serialization() {
        let id = ConceptId::from_concept("test", "Entity");
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ConceptId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_concept_id_from_legacy_uuid() {
        let uuid = Uuid::new_v4();
        let concept_id = ConceptId::from_legacy_uuid(uuid);
        assert_eq!(*concept_id.as_uuid(), uuid);
    }

    #[test]
    fn test_concept_id_display() {
        let id = ConceptId::from_concept("test", "Entity");
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }
}
