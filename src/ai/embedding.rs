use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::{Result, Context};

pub struct EmbeddingWrapper {
    model: TextEmbedding,
    dimension: usize,
}

impl EmbeddingWrapper {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(Default::default())
            .context("Failed to create TextEmbedding with default options")?;
        let dimension = Self::get_dimension(&model)?;
        Ok(Self { model, dimension })
    }

    pub fn with_options(options: InitOptions) -> Result<Self> {
        let model = TextEmbedding::try_new(options)
            .context("Failed to create TextEmbedding with custom options")?;
        let dimension = Self::get_dimension(&model)?;
        Ok(Self { model, dimension })
    }

    pub fn generate(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        self.model.embed(texts, None)
            .context("Failed to generate embeddings")
    }

    pub fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    fn get_dimension(model: &TextEmbedding) -> Result<usize> {
        let sample_text = "Sample text for dimension check";
        let sample_embedding = model.embed(vec![sample_text], None)
            .context("Failed to generate sample embedding")?;
        Ok(sample_embedding[0].len())
    }
}

impl Default for EmbeddingWrapper {
    fn default() -> Self {
        Self::new().expect("Failed to create default EmbeddingWrapper")
    }
}