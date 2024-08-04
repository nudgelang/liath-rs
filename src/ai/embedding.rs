use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::Result;

pub struct EmbeddingWrapper {
    model: TextEmbedding,
}

impl EmbeddingWrapper {
    pub fn new(model_name: EmbeddingModel) -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name,
            show_download_progress: true,
            ..Default::default()
        })?;
        Ok(Self { model })
    }

    pub fn generate(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(&[text], None)?;
        Ok(embeddings[0].clone())
    }
}