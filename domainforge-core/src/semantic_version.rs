use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err("Version must be in format major.minor.patch".to_string());
        }

        Ok(Self {
            major: parts[0]
                .parse()
                .map_err(|_| "Invalid major version".to_string())?,
            minor: parts[1]
                .parse()
                .map_err(|_| "Invalid minor version".to_string())?,
            patch: parts[2]
                .parse()
                .map_err(|_| "Invalid patch version".to_string())?,
        })
    }

    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    pub fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for SemanticVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_version_parsing() {
        let v = SemanticVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_semantic_version_parsing_invalid() {
        assert!(SemanticVersion::parse("1.2").is_err());
        assert!(SemanticVersion::parse("1.2.3.4").is_err());
        assert!(SemanticVersion::parse("a.b.c").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(2, 0, 0);
        assert!(v1 < v2);

        let v3 = SemanticVersion::new(1, 2, 0);
        let v4 = SemanticVersion::new(1, 1, 0);
        assert!(v3 > v4);

        let v5 = SemanticVersion::new(1, 1, 2);
        let v6 = SemanticVersion::new(1, 1, 1);
        assert!(v5 > v6);
    }

    #[test]
    fn test_version_bump_major() {
        let mut v = SemanticVersion::new(1, 2, 3);
        v.bump_major();
        assert_eq!(v, SemanticVersion::new(2, 0, 0));
    }

    #[test]
    fn test_version_bump_minor() {
        let mut v = SemanticVersion::new(1, 2, 3);
        v.bump_minor();
        assert_eq!(v, SemanticVersion::new(1, 3, 0));
    }

    #[test]
    fn test_version_bump_patch() {
        let mut v = SemanticVersion::new(1, 2, 3);
        v.bump_patch();
        assert_eq!(v, SemanticVersion::new(1, 2, 4));
    }

    #[test]
    fn test_version_display() {
        let v = SemanticVersion::new(1, 2, 3);
        assert_eq!(format!("{}", v), "1.2.3");
    }

    #[test]
    fn test_version_default() {
        let v = SemanticVersion::default();
        assert_eq!(v, SemanticVersion::new(1, 0, 0));
    }

    #[test]
    fn test_version_serialization() {
        let v = SemanticVersion::new(1, 2, 3);
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: SemanticVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(v, deserialized);
    }
}
