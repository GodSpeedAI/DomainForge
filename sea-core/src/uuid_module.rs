use uuid::Uuid;

/// Generate a new UUID v7
pub fn generate_uuid_v7() -> Uuid {
    Uuid::now_v7()
}

/// Parse a UUID from string
pub fn parse_uuid(s: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(s)
}

/// Format UUID as string
pub fn format_uuid(id: &Uuid) -> String {
    id.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generation() {
        let id = generate_uuid_v7();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_uuid_parsing() {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = parse_uuid(id_str).unwrap();
        assert_eq!(format_uuid(&id), id_str);
    }

    #[test]
    fn test_uuid_determinism() {
        let id1 = generate_uuid_v7();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = generate_uuid_v7();
        assert!(id1 <= id2); // Allow equality for same millisecond
    }
}
