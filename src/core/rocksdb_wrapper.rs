use rocksdb::{TransactionDB, Options, WriteOptions, Transaction};
use std::path::Path;
use anyhow::{Result, Context};

pub struct RocksDBWrapper {
    db: TransactionDB,
}

impl RocksDBWrapper {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        
        let txn_db_opts = rocksdb::TransactionDBOptions::default();
        
        let db = TransactionDB::open(&opts, &txn_db_opts, path)
            .context("Failed to open TransactionDB")?;
        
        Ok(Self { db })
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)
            .context("Failed to put value in DB")?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.db.get(key)
            .context("Failed to get value from DB")
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.db.delete(key)
            .context("Failed to delete value from DB")?;
        Ok(())
    }

    pub fn transaction(&self) -> Transaction<TransactionDB> {
        self.db.transaction()
    }

    // New method to perform a write operation within a transaction
    pub fn transaction_put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut tx = self.db.transaction();
        tx.put(key, value)
            .context("Failed to put value in transaction")?;
        tx.commit()
            .context("Failed to commit transaction")?;
        Ok(())
    }
}