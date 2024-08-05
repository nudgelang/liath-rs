use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::core::RocksDBWrapper;
use crate::vector::UsearchWrapper;
use anyhow::{Result, Context};
use usearch::{MetricKind, ScalarKind};

#[derive(Clone)]
pub struct Namespace {
    pub db: Arc<RocksDBWrapper>,
    pub vector_db: Arc<UsearchWrapper>,
}

impl Namespace {
    pub fn new(db: RocksDBWrapper, vector_db: UsearchWrapper) -> Self {
        Self { 
            db: Arc::new(db), 
            vector_db: Arc::new(vector_db) 
        }
    }
}

pub struct NamespaceManager {
    namespaces: Arc<RwLock<HashMap<String, Namespace>>>,
}

impl NamespaceManager {
    pub fn new() -> Self {
        Self {
            namespaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_namespace(&self, name: &str, dimensions: usize, metric: MetricKind, scalar: ScalarKind) -> Result<()> {
        let mut namespaces = self.namespaces.write().unwrap();
        if namespaces.contains_key(name) {
            return Err(anyhow::anyhow!("Namespace '{}' already exists", name));
        }
        
        let db = RocksDBWrapper::new(format!("data/{}", name))
            .context(format!("Failed to create RocksDB for namespace '{}'", name))?;
        let vector_db = UsearchWrapper::new(dimensions, metric, scalar)
            .context(format!("Failed to create UsearchWrapper for namespace '{}'", name))?;
        
        namespaces.insert(name.to_string(), Namespace::new(db, vector_db));
        Ok(())
    }

    pub fn get_namespace(&self, name: &str) -> Result<Namespace> {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Namespace '{}' not found", name))
    }

    pub fn delete_namespace(&self, name: &str) -> Result<()> {
        let mut namespaces = self.namespaces.write().unwrap();
        namespaces.remove(name)
            .ok_or_else(|| anyhow::anyhow!("Namespace '{}' not found", name))?;
        // TODO: Consider adding cleanup logic here (e.g., deleting files)
        Ok(())
    }

    pub fn list_namespaces(&self) -> Vec<String> {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.keys().cloned().collect()
    }

    pub fn namespace_exists(&self, name: &str) -> bool {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_manager() {
        let manager = NamespaceManager::new();

        // Create a namespace
        assert!(manager.create_namespace("test1", 128, MetricKind::Cos, ScalarKind::F32).is_ok());

        // Check if namespace exists
        assert!(manager.namespace_exists("test1"));
        assert!(!manager.namespace_exists("nonexistent"));

        // Try to create a duplicate namespace
        assert!(manager.create_namespace("test1", 128, MetricKind::Cos, ScalarKind::F32).is_err());

        // Get a namespace
        let namespace = manager.get_namespace("test1");
        assert!(namespace.is_ok());

        // List namespaces
        let namespaces = manager.list_namespaces();
        assert_eq!(namespaces, vec!["test1"]);

        // Delete a namespace
        assert!(manager.delete_namespace("test1").is_ok());

        // Try to get a deleted namespace
        assert!(manager.get_namespace("test1").is_err());

        // Try to delete a non-existent namespace
        assert!(manager.delete_namespace("nonexistent").is_err());
    }
}