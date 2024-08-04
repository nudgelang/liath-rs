use usearch::Index;
use anyhow::Result;

pub struct UsearchWrapper {
    index: Index,
}

impl UsearchWrapper {
    pub fn new(dimensions: usize) -> Result<Self> {
        let index = Index::new(dimensions)?;
        Ok(Self { index })
    }

    pub fn add(&mut self, id: u64, vector: &[f32]) -> Result<()> {
        self.index.add(id, vector)?;
        Ok(())
    }

    pub fn search(&self, vector: &[f32], k: usize) -> Result<Vec<(u64, f32)>> {
        let results = self.index.search(vector, k)?;
        Ok(results.into_iter().map(|r| (r.key, r.distance)).collect())
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        self.index.save(path)?;
        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let index = Index::load(path)?;
        Ok(Self { index })
    }
}