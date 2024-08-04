use std::collections::HashMap;
use crate::query::parser::QueryType;
use anyhow::Result;

pub struct AuthManager {
    user_permissions: HashMap<String, Vec<QueryType>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            user_permissions: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user_id: &str, permissions: Vec<QueryType>) {
        self.user_permissions.insert(user_id.to_string(), permissions);
    }

    pub fn is_authorized(&self, user_id: &str, query_type: &QueryType) -> bool {
        self.user_permissions
            .get(user_id)
            .map(|permissions| permissions.contains(query_type))
            .unwrap_or(false)
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<()> {
        self.user_permissions.remove(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        Ok(())
    }

    pub fn update_permissions(&mut self, user_id: &str, permissions: Vec<QueryType>) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?
            .clone_from(&permissions);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager() {
        let mut auth_manager = AuthManager::new();
        
        auth_manager.add_user("user1", vec![QueryType::Select, QueryType::Insert]);
        assert!(auth_manager.is_authorized("user1", &QueryType::Select));
        assert!(auth_manager.is_authorized("user1", &QueryType::Insert));
        assert!(!auth_manager.is_authorized("user1", &QueryType::Delete));
        
        auth_manager.update_permissions("user1", vec![QueryType::Select, QueryType::Delete]).unwrap();
        assert!(auth_manager.is_authorized("user1", &QueryType::Select));
        assert!(!auth_manager.is_authorized("user1", &QueryType::Insert));
        assert!(auth_manager.is_authorized("user1", &QueryType::Delete));
        
        auth_manager.remove_user("user1").unwrap();
        assert!(!auth_manager.is_authorized("user1", &QueryType::Select));
    }
}