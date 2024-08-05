use usearch::{Index, IndexOptions, MetricKind, ScalarKind, new_index};
use anyhow::{Result, Context};

pub struct UsearchWrapper {
    index: Index,
}

impl UsearchWrapper {
    pub fn new(dimensions: usize, metric: MetricKind, quantization: ScalarKind) -> Result<Self> {
        let options = IndexOptions {
            dimensions,
            metric,
            quantization,
            connectivity: 0, // zero for auto
            expansion_add: 0, // zero for auto
            expansion_search: 0, // zero for auto
            multi: false,
        };

        let index = new_index(&options).context("Failed to create new index")?;
        Ok(Self { index })
    }

    pub fn reserve(&self, capacity: usize) -> Result<()> {
        self.index.reserve(capacity).context("Failed to reserve capacity")
    }

    pub fn add(&self, id: u64, vector: &[f32]) -> Result<()> {
        self.index.add(id, vector).context("Failed to add vector to index")
    }

    pub fn search(&self, vector: &[f32], k: usize) -> Result<Vec<(u64, f32)>> {
        let results = self.index.search(vector, k).context("Failed to perform search")?;
        Ok(results.keys.into_iter().zip(results.distances).collect())
    }

    pub fn save(&self, path: &str) -> Result<()> {
        self.index.save(path).context("Failed to save index")
    }

    pub fn load(&self,path: &str) -> Result<()> {
        self.index.load(path).context("Failed to save index")
    }

    pub fn view(&self,path: &str) -> Result<()> {
        self.index.view(path).context("Failed to save index")
    }

    pub fn capacity(&self) -> usize {
        self.index.capacity()
    }

    pub fn connectivity(&self) -> usize {
        self.index.connectivity()
    }

    pub fn dimensions(&self) -> usize {
        self.index.dimensions()
    }

    pub fn size(&self) -> usize {
        self.index.size()
    }
}