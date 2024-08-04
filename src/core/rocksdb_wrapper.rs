use rocksdb::{DB, Options, WriteBatch, Transaction, TransactionDB, TransactionOptions};
use std::path::Path;
use anyhow::Result;

pub struct RocksDBWrapper {
    db: TransactionDB,
}

impl RocksDBWrapper {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = TransactionDB::open(&opts, path)?;
        Ok(Self { db })
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?)
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.db.delete(key)?;
        Ok(())
    }

    pub fn transaction(&self) -> Transaction<TransactionDB> {
        self.db.transaction()
    }
}