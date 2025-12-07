use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub allowed_features: HashSet<String>,
}

impl Profile {
    pub fn new(name: &str, description: &str, features: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            allowed_features: features.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn is_feature_allowed(&self, feature: &str) -> bool {
        self.allowed_features.contains(feature)
    }
}

pub struct ProfileRegistry {
    profiles: Vec<Profile>,
}

impl ProfileRegistry {
    fn new() -> Self {
        let mut profiles = Vec::new();

        // Default profile - everything enabled
        profiles.push(Profile::new(
            "default",
            "Standard SEA DSL profile with all core features",
            &["core", "cloud", "data"],
        ));

        // Cloud profile
        profiles.push(Profile::new(
            "cloud",
            "Cloud infrastructure modeling",
            &["core", "cloud"],
        ));

        // Data profile
        profiles.push(Profile::new(
            "data",
            "Data modeling and governance",
            &["core", "data"],
        ));

        Self { profiles }
    }

    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    pub fn list_names(&self) -> Vec<&str> {
        self.profiles.iter().map(|p| p.name.as_str()).collect()
    }

    pub fn global() -> &'static ProfileRegistry {
        static REGISTRY: OnceLock<ProfileRegistry> = OnceLock::new();
        REGISTRY.get_or_init(ProfileRegistry::new)
    }
}
