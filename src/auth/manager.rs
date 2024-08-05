use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};

pub struct AuthManager {
    user_permissions: HashMap<String, HashSet<String>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            user_permissions: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user_id: &str, permissions: Vec<String>) {
        self.user_permissions.insert(user_id.to_string(), permissions.into_iter().collect());
    }

    pub fn is_authorized(&self, user_id: &str, permission: &str) -> bool {
        self.user_permissions
            .get(user_id)
            .map(|permissions| permissions.contains(permission))
            .unwrap_or(false)
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<()> {
        self.user_permissions.remove(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        Ok(())
    }

    pub fn update_permissions(&mut self, user_id: &str, permissions: Vec<String>) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .clear();
        self.user_permissions.get_mut(user_id).unwrap().extend(permissions);
        Ok(())
    }

    pub fn add_permission(&mut self, user_id: &str, permission: String) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .insert(permission);
        Ok(())
    }

    pub fn remove_permission(&mut self, user_id: &str, permission: &str) -> Result<()> {
        self.user_permissions.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .remove(permission);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager() {
        let mut auth_manager = AuthManager::new();
        
        auth_manager.add_user("user1", vec!["select".to_string(), "insert".to_string()]);
        assert!(auth_manager.is_authorized("user1", "select"));
        assert!(auth_manager.is_authorized("user1", "insert"));
        assert!(!auth_manager.is_authorized("user1", "delete"));
        
        auth_manager.update_permissions("user1", vec!["select".to_string(), "delete".to_string()]).unwrap();
        assert!(auth_manager.is_authorized("user1", "select"));
        assert!(!auth_manager.is_authorized("user1", "insert"));
        assert!(auth_manager.is_authorized("user1", "delete"));
        
        auth_manager.add_permission("user1", "update".to_string()).unwrap();
        assert!(auth_manager.is_authorized("user1", "update"));
        
        auth_manager.remove_permission("user1", "delete").unwrap();
        assert!(!auth_manager.is_authorized("user1", "delete"));
        
        auth_manager.remove_user("user1").unwrap();
        assert!(!auth_manager.is_authorized("user1", "select"));
    }
}