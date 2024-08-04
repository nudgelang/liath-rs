use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::core::RocksDBWrapper;
use crate::vector::UsearchWrapper;
use anyhow::Result;

pub struct Namespace {
    db: RocksDBWrapper,
    vector_db: UsearchWrapper,
}

impl Namespace {
    pub fn new(db: RocksDBWrapper, vector_db: UsearchWrapper) -> Self {
        Self { db, vector_db }
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

    pub fn create_namespace(&self, name: &str) -> Result<()> {
        let mut namespaces = self.namespaces.write().unwrap();
        if namespaces.contains_key(name) {
            return Err(anyhow::anyhow!("Namespace already exists"));
        }
        
        let db = RocksDBWrapper::new(format!("data/{}", name))?;
        let vector_db = UsearchWrapper::new(384)?; // Assuming 384 dimensions for embeddings
        
        namespaces.insert(name.to_string(), Namespace::new(db, vector_db));
        Ok(())
    }

    pub fn get_namespace(&self, name: &str) -> Option<Namespace> {
        let namespaces = self.namespaces.read().unwrap();
        namespaces.get(name).cloned()
    }

    pub fn delete_namespace(&self, name: &str) -> Result<()> {
        let mut namespaces = self.namespaces.write().unwrap();
        namespaces.remove(name).ok_or_else(|| anyhow::anyhow!("Namespace not found"))?;
        Ok(())
    }
}